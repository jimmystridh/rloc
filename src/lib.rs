//! # rloc
//!
//! A fast, modern library for counting lines of code and detecting programming languages.
//!
//! ## Quick Start
//!
//! ```no_run
//! use std::path::Path;
//!
//! // Get the top language for a directory
//! let top = rloc::top_language(Path::new(".")).unwrap();
//! println!("Top language: {} ({} lines of code)", top.name, top.code);
//!
//! // Fast mode: only look at extensions, don't read file contents
//! let top_fast = rloc::top_language_fast(Path::new(".")).unwrap();
//! println!("Top language (fast): {} ({} files)", top_fast.name, top_fast.files);
//!
//! // Full analysis with all languages
//! let analysis = rloc::analyze(Path::new(".")).unwrap();
//! for lang in &analysis.languages {
//!     println!("{}: {} files, {} code", lang.name, lang.files, lang.code);
//! }
//! ```
//!
//! ## Single File Detection
//!
//! ```no_run
//! use std::path::Path;
//!
//! if let Some(lang) = rloc::detect_language(Path::new("main.rs")) {
//!     println!("Detected: {}", lang.name);
//! }
//! ```

// Internal modules - exposed publicly for CLI binary
pub mod archive;
pub mod counter;
pub mod custom_langs;
mod languages;
pub mod stats;
pub mod walker;

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub mod diff;
#[cfg(feature = "cli")]
pub mod output;
#[cfg(feature = "cli")]
pub mod strip;

use dashmap::DashSet;
use rayon::prelude::*;
use std::path::Path;

pub use languages::{LANGUAGES, Language, detect_language, list_extensions, list_languages};

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Default)]
pub struct LanguageBreakdown {
    pub name: &'static str,
    pub files: u64,
    pub code: u64,
    pub comments: u64,
    pub blanks: u64,
}

impl LanguageBreakdown {
    pub fn total_lines(&self) -> u64 {
        self.code + self.comments + self.blanks
    }
}

#[derive(Debug, Clone, Default)]
pub struct Analysis {
    pub languages: Vec<LanguageBreakdown>,
    pub total_files: u64,
    pub total_code: u64,
    pub total_comments: u64,
    pub total_blanks: u64,
}

impl Analysis {
    pub fn top_language(&self) -> Option<&LanguageBreakdown> {
        self.languages.first()
    }

    pub fn total_lines(&self) -> u64 {
        self.total_code + self.total_comments + self.total_blanks
    }
}

#[derive(Debug, Clone, Default)]
pub struct AnalyzeConfig {
    pub paths: Vec<std::path::PathBuf>,
    pub exclude_dirs: Vec<String>,
    pub exclude_exts: Vec<String>,
    pub exclude_langs: Vec<String>,
    pub include_exts: Vec<String>,
    pub include_langs: Vec<String>,
    pub follow_symlinks: bool,
    pub hidden: bool,
    pub max_depth: Option<usize>,
    pub skip_gitignore: bool,
    pub max_file_size: Option<u64>,
    pub threads: Option<usize>,
}

impl AnalyzeConfig {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            paths: vec![path.as_ref().to_path_buf()],
            exclude_dirs: walker::WalkerConfig::default().exclude_dirs,
            ..Default::default()
        }
    }

    pub fn paths(mut self, paths: Vec<std::path::PathBuf>) -> Self {
        self.paths = paths;
        self
    }

    pub fn exclude_dirs(mut self, dirs: Vec<String>) -> Self {
        self.exclude_dirs = dirs;
        self
    }

    pub fn include_langs(mut self, langs: Vec<String>) -> Self {
        self.include_langs = langs;
        self
    }

    pub fn exclude_langs(mut self, langs: Vec<String>) -> Self {
        self.exclude_langs = langs;
        self
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    pub fn threads(mut self, threads: usize) -> Self {
        self.threads = Some(threads);
        self
    }
}

/// Get the top (most code) language in a directory.
///
/// This performs a full analysis, reading file contents to count lines of code,
/// comments, and blank lines.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
///
/// let top = rloc::top_language(Path::new(".")).unwrap();
/// println!("{}: {} lines of code", top.name, top.code);
/// ```
pub fn top_language(path: impl AsRef<Path>) -> Result<LanguageBreakdown> {
    let analysis = analyze(path)?;
    analysis.top_language().cloned().ok_or(Error::NoSourceFiles)
}

/// Get the top language quickly by only counting files (not reading contents).
///
/// This is much faster than `top_language()` but only gives file counts,
/// not lines of code.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
///
/// let top = rloc::top_language_fast(Path::new(".")).unwrap();
/// println!("{}: {} files", top.name, top.files);
/// ```
pub fn top_language_fast(path: impl AsRef<Path>) -> Result<LanguageBreakdown> {
    let analysis = analyze_fast(path)?;
    analysis.top_language().cloned().ok_or(Error::NoSourceFiles)
}

/// Analyze a directory and return full statistics for all languages.
///
/// This reads file contents to accurately count lines of code, comments,
/// and blank lines.
pub fn analyze(path: impl AsRef<Path>) -> Result<Analysis> {
    let config = AnalyzeConfig::new(path);
    analyze_with_config(config)
}

/// Fast analysis that only counts files by extension (no file reads).
///
/// This is useful when you only need to know the language distribution
/// by file count, not by lines of code.
pub fn analyze_fast(path: impl AsRef<Path>) -> Result<Analysis> {
    let config = AnalyzeConfig::new(path);
    analyze_fast_with_config(config)
}

/// Analyze with custom configuration.
pub fn analyze_with_config(config: AnalyzeConfig) -> Result<Analysis> {
    if let Some(threads) = config.threads {
        if threads > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build_global()
                .ok();
        }
    }

    let walker_config = config_to_walker(&config);
    let files = walker::walk_files(&walker_config);

    if files.is_empty() {
        return Err(Error::NoSourceFiles);
    }

    let seen_hashes: DashSet<u64> = DashSet::new();

    let file_stats: Vec<_> = files
        .into_par_iter()
        .filter_map(|entry| {
            if let Ok(hash) = counter::compute_file_hash(&entry.path) {
                if !seen_hashes.insert(hash) {
                    return None;
                }
            }

            match counter::count_lines(&entry.path, entry.language) {
                Ok(stats) if stats.total() > 0 => Some(stats),
                _ => None,
            }
        })
        .collect();

    let summary = stats::Summary::from_file_stats(file_stats);
    Ok(summary_to_analysis(&summary))
}

/// Fast analysis with custom configuration (extension-only, no file reads).
pub fn analyze_fast_with_config(config: AnalyzeConfig) -> Result<Analysis> {
    let walker_config = config_to_walker(&config);
    let files = walker::walk_files(&walker_config);

    if files.is_empty() {
        return Err(Error::NoSourceFiles);
    }

    use ahash::AHashMap;
    let mut by_language: AHashMap<&'static str, u64> = AHashMap::new();

    for entry in &files {
        *by_language.entry(entry.language.name).or_insert(0) += 1;
    }

    let mut languages: Vec<_> = by_language
        .into_iter()
        .map(|(name, files)| LanguageBreakdown {
            name,
            files,
            code: 0,
            comments: 0,
            blanks: 0,
        })
        .collect();

    languages.sort_by(|a, b| b.files.cmp(&a.files));

    let total_files = languages.iter().map(|l| l.files).sum();

    Ok(Analysis {
        languages,
        total_files,
        total_code: 0,
        total_comments: 0,
        total_blanks: 0,
    })
}

fn config_to_walker(config: &AnalyzeConfig) -> walker::WalkerConfig {
    walker::WalkerConfig {
        paths: if config.paths.is_empty() {
            vec![std::path::PathBuf::from(".")]
        } else {
            config.paths.clone()
        },
        exclude_dirs: config.exclude_dirs.clone(),
        exclude_exts: config.exclude_exts.clone(),
        exclude_langs: config.exclude_langs.clone(),
        include_exts: config.include_exts.clone(),
        include_langs: config.include_langs.clone(),
        follow_symlinks: config.follow_symlinks,
        hidden: config.hidden,
        max_depth: config.max_depth,
        skip_gitignore: config.skip_gitignore,
        max_file_size: config.max_file_size,
        ..Default::default()
    }
}

fn summary_to_analysis(summary: &stats::Summary) -> Analysis {
    Analysis {
        languages: summary
            .languages
            .iter()
            .map(|l| LanguageBreakdown {
                name: languages::LANGUAGES
                    .get(&l.name)
                    .map(|lang| lang.name)
                    .unwrap_or_else(|| {
                        // For custom languages, we need to leak the string to get 'static
                        // This is acceptable since language names are bounded and reused
                        Box::leak(l.name.clone().into_boxed_str())
                    }),
                files: l.files,
                code: l.code,
                comments: l.comments,
                blanks: l.blanks,
            })
            .collect(),
        total_files: summary.total_files,
        total_code: summary.total_code,
        total_comments: summary.total_comments,
        total_blanks: summary.total_blanks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_analyze_simple() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("main.rs"),
            "fn main() {\n    println!(\"Hello\");\n}\n",
        )
        .unwrap();
        fs::write(
            temp.path().join("lib.rs"),
            "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
        )
        .unwrap();

        let analysis = analyze(temp.path()).unwrap();
        assert_eq!(analysis.total_files, 2);
        assert!(analysis.total_code > 0);

        let top = analysis.top_language().unwrap();
        assert_eq!(top.name, "Rust");
    }

    #[test]
    fn test_analyze_fast() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(temp.path().join("lib.rs"), "pub fn x() {}").unwrap();
        fs::write(temp.path().join("app.ts"), "const x = 1;").unwrap();

        let analysis = analyze_fast(temp.path()).unwrap();
        assert_eq!(analysis.total_files, 3);

        let top = analysis.top_language().unwrap();
        assert_eq!(top.name, "Rust");
        assert_eq!(top.files, 2);
    }

    #[test]
    fn test_top_language() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("main.py"),
            "print('hello')\nprint('world')\n",
        )
        .unwrap();
        fs::write(temp.path().join("lib.rs"), "fn x() {}\n").unwrap();

        let top = top_language(temp.path()).unwrap();
        assert_eq!(top.name, "Python");
    }

    #[test]
    fn test_detect_language() {
        let lang = detect_language(Path::new("test.rs")).unwrap();
        assert_eq!(lang.name, "Rust");

        let lang = detect_language(Path::new("test.ts")).unwrap();
        assert_eq!(lang.name, "TypeScript");

        assert!(detect_language(Path::new("test.unknown")).is_none());
    }

    #[test]
    fn test_no_source_files() {
        let temp = TempDir::new().unwrap();
        // Use an extension that's not recognized as any language
        fs::write(temp.path().join("readme.xyz"), "hello").unwrap();

        let result = analyze(temp.path());
        assert!(matches!(result, Err(Error::NoSourceFiles)));
    }
}
