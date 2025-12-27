use crate::languages::{detect_language, get_language_ignore_case, Language};
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct WalkerConfig {
    pub paths: Vec<PathBuf>,
    pub list_file: Option<PathBuf>,
    pub exclude_dirs: Vec<String>,
    pub exclude_exts: Vec<String>,
    pub exclude_langs: Vec<String>,
    pub include_exts: Vec<String>,
    pub include_langs: Vec<String>,
    pub force_lang: HashMap<String, String>,
    pub match_dir: Option<Regex>,
    pub not_match_dir: Vec<Regex>,
    pub match_file: Option<Regex>,
    pub not_match_file: Vec<Regex>,
    pub include_content: Option<Regex>,
    pub exclude_content: Option<Regex>,
    pub vcs: Option<VcsMode>,
    pub follow_symlinks: bool,
    pub hidden: bool,
    pub fullpath: bool,
    pub max_depth: Option<usize>,
    pub skip_gitignore: bool,
    pub skip_uniqueness: bool,
    pub include_submodules: bool,
    pub max_file_size: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VcsMode {
    Auto,
    Git,
    None,
}

impl Default for WalkerConfig {
    fn default() -> Self {
        Self {
            paths: vec![PathBuf::from(".")],
            list_file: None,
            exclude_dirs: vec![
                ".git".into(),
                ".svn".into(),
                ".hg".into(),
                "node_modules".into(),
                "target".into(),
                "vendor".into(),
                "dist".into(),
                "build".into(),
                "__pycache__".into(),
                ".tox".into(),
                ".eggs".into(),
                "venv".into(),
                ".venv".into(),
                "env".into(),
                ".env".into(),
            ],
            exclude_exts: vec![],
            exclude_langs: vec![],
            include_exts: vec![],
            include_langs: vec![],
            force_lang: HashMap::new(),
            match_dir: None,
            not_match_dir: vec![],
            match_file: None,
            not_match_file: vec![],
            include_content: None,
            exclude_content: None,
            vcs: None,
            follow_symlinks: false,
            hidden: false,
            fullpath: false,
            max_depth: None,
            skip_gitignore: false,
            skip_uniqueness: false,
            include_submodules: false,
            max_file_size: None,
        }
    }
}

pub struct FileEntry {
    pub path: PathBuf,
    pub language: &'static Language,
}

pub fn walk_files(config: &WalkerConfig) -> Vec<FileEntry> {
    if let Some(ref list_file) = config.list_file {
        return walk_list_file(list_file, config);
    }

    if let Some(VcsMode::Git) = config.vcs {
        return walk_git_files(config);
    }

    if let Some(VcsMode::Auto) = config.vcs {
        if Path::new(".git").exists() {
            return walk_git_files(config);
        }
    }

    walk_filesystem(config)
}

fn walk_list_file(list_file: &Path, config: &WalkerConfig) -> Vec<FileEntry> {
    let content = match std::fs::read_to_string(list_file) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let files: Vec<PathBuf> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(PathBuf::from)
        .collect();

    filter_files(files, config)
}

fn walk_git_files(config: &WalkerConfig) -> Vec<FileEntry> {
    let mut args = vec!["ls-files", "--cached", "--others", "--exclude-standard"];
    if config.include_submodules {
        args.push("--recurse-submodules");
    }

    let output = Command::new("git")
        .args(&args)
        .output();

    let files = match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(PathBuf::from)
                .collect::<Vec<_>>()
        }
        _ => return walk_filesystem(config),
    };

    filter_files(files, config)
}

fn walk_filesystem(config: &WalkerConfig) -> Vec<FileEntry> {
    let mut files = Vec::new();

    for start_path in &config.paths {
        let mut builder = WalkBuilder::new(start_path);

        builder
            .hidden(!config.hidden)
            .follow_links(config.follow_symlinks)
            .git_ignore(!config.skip_gitignore)
            .git_global(!config.skip_gitignore)
            .git_exclude(!config.skip_gitignore);

        if let Some(depth) = config.max_depth {
            builder.max_depth(Some(depth));
        }

        let mut overrides = OverrideBuilder::new(start_path);

        for dir in &config.exclude_dirs {
            let _ = overrides.add(&format!("!**/{}/", dir));
            let _ = overrides.add(&format!("!{}/", dir));
        }

        if let Ok(ovr) = overrides.build() {
            builder.overrides(ovr);
        }

        for entry in builder.build().filter_map(Result::ok) {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                files.push(entry.into_path());
            }
        }
    }

    filter_files(files, config)
}

fn filter_files(files: Vec<PathBuf>, config: &WalkerConfig) -> Vec<FileEntry> {
    let include_langs_lower: Vec<String> = config.include_langs.iter()
        .map(|s| s.to_lowercase())
        .collect();
    let exclude_langs_lower: Vec<String> = config.exclude_langs.iter()
        .map(|s| s.to_lowercase())
        .collect();

    let max_bytes = config.max_file_size.map(|mb| mb * 1024 * 1024);

    files
        .into_iter()
        .filter(|path| {
            // Check file size first (if configured)
            if let Some(max) = max_bytes {
                if let Ok(meta) = path.metadata() {
                    if meta.len() > max {
                        return false;
                    }
                }
            }

            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if !config.include_exts.is_empty()
                    && !config.include_exts.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                        return false;
                    }
                if config.exclude_exts.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                    return false;
                }
            } else if !config.include_exts.is_empty() {
                return false;
            }

            if let Some(ref regex) = config.match_file {
                let name = if config.fullpath {
                    path.to_string_lossy()
                } else {
                    path.file_name()
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_default()
                };
                if !regex.is_match(&name) {
                    return false;
                }
            }

            for regex in &config.not_match_file {
                let name = if config.fullpath {
                    path.to_string_lossy()
                } else {
                    path.file_name()
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_default()
                };
                if regex.is_match(&name) {
                    return false;
                }
            }

            if let Some(ref regex) = config.match_dir {
                let dir = path.parent()
                    .map(|p| p.to_string_lossy())
                    .unwrap_or_default();
                if !regex.is_match(&dir) {
                    return false;
                }
            }

            for regex in &config.not_match_dir {
                let dir_name = if config.fullpath {
                    path.parent()
                        .map(|p| p.to_string_lossy())
                        .unwrap_or_default()
                } else {
                    path.parent()
                        .and_then(|p| p.file_name())
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_default()
                };
                if regex.is_match(&dir_name) {
                    return false;
                }
            }

            if config.include_content.is_some() || config.exclude_content.is_some() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Some(ref regex) = config.include_content {
                        if !regex.is_match(&content) {
                            return false;
                        }
                    }
                    if let Some(ref regex) = config.exclude_content {
                        if regex.is_match(&content) {
                            return false;
                        }
                    }
                } else {
                    return false;
                }
            }

            true
        })
        .filter_map(|path| {
            let language = if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if let Some(forced_lang) = config.force_lang.get(&ext.to_lowercase()) {
                    get_language_ignore_case(forced_lang)
                } else {
                    detect_language(&path)
                }
            } else {
                detect_language(&path)
            }?;

            if !include_langs_lower.is_empty()
                && !include_langs_lower.iter().any(|l| l.eq_ignore_ascii_case(language.name)) {
                    return None;
                }

            if exclude_langs_lower.iter().any(|l| l.eq_ignore_ascii_case(language.name)) {
                return None;
            }

            Some(FileEntry { path, language })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_excludes() {
        let config = WalkerConfig::default();
        assert!(config.exclude_dirs.contains(&"node_modules".to_string()));
        assert!(config.exclude_dirs.contains(&"target".to_string()));
        assert!(config.exclude_dirs.contains(&".git".to_string()));
    }
}
