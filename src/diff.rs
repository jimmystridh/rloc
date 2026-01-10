use crate::counter::{count_lines, FileStats};
use crate::walker::{walk_files, FileEntry, WalkerConfig};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct DiffStats {
    pub same: LanguageDiff,
    pub modified: LanguageDiff,
    pub added: LanguageDiff,
    pub removed: LanguageDiff,
}

#[derive(Debug, Clone, Default)]
pub struct LanguageDiff {
    pub files: u64,
    pub code: u64,
    pub comments: u64,
    pub blanks: u64,
}

impl LanguageDiff {
    pub fn add(&mut self, stats: &FileStats) {
        self.files += 1;
        self.code += stats.code;
        self.comments += stats.comments;
        self.blanks += stats.blanks;
    }

    #[allow(dead_code)]
    pub fn total(&self) -> u64 {
        self.code + self.comments + self.blanks
    }
}

#[derive(Debug)]
pub struct DiffResult {
    pub by_language: HashMap<String, DiffStats>,
    pub totals: DiffStats,
}

pub fn compute_diff(config1: &WalkerConfig, config2: &WalkerConfig, verbose: bool) -> DiffResult {
    let files1 = walk_files(config1);
    let files2 = walk_files(config2);

    let stats1 = collect_stats(&files1, verbose);
    let stats2 = collect_stats(&files2, verbose);

    let mut by_language: HashMap<String, DiffStats> = HashMap::new();
    let mut totals = DiffStats::default();

    // Process files from set 1
    for (path, (lang, stats)) in &stats1 {
        let entry = by_language.entry(lang.clone()).or_default();

        if let Some((_, stats2)) = stats2.get(path) {
            if stats.code == stats2.code
                && stats.comments == stats2.comments
                && stats.blanks == stats2.blanks
            {
                entry.same.add(stats);
                totals.same.add(stats);
            } else {
                entry.modified.add(stats);
                totals.modified.add(stats);
            }
        } else {
            entry.removed.add(stats);
            totals.removed.add(stats);
        }
    }

    // Process files only in set 2 (added)
    for (path, (lang, stats)) in &stats2 {
        if !stats1.contains_key(path) {
            let entry = by_language.entry(lang.clone()).or_default();
            entry.added.add(stats);
            totals.added.add(stats);
        }
    }

    DiffResult {
        by_language,
        totals,
    }
}

fn collect_stats(files: &[FileEntry], verbose: bool) -> HashMap<PathBuf, (String, FileStats)> {
    let mut result = HashMap::new();

    for entry in files {
        match count_lines(&entry.path, entry.language) {
            Ok(stats) if stats.total() > 0 => {
                let relative = entry
                    .path
                    .file_name()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| entry.path.clone());
                result.insert(relative, (entry.language.name.to_string(), stats));
            }
            Err(e) if verbose => {
                eprintln!("warning: {}: {}", entry.path.display(), e);
            }
            _ => {}
        }
    }

    result
}

pub fn render_diff(result: &DiffResult) {
    println!();
    println!(
        "{:<14} {:>10} {:>10} {:>10} {:>10}",
        "Language", "Same", "Modified", "Added", "Removed"
    );
    println!("{}", "─".repeat(58));

    let mut langs: Vec<_> = result.by_language.iter().collect();
    langs.sort_by(|a, b| {
        let total_a = a.1.same.code + a.1.modified.code + a.1.added.code + a.1.removed.code;
        let total_b = b.1.same.code + b.1.modified.code + b.1.added.code + b.1.removed.code;
        total_b.cmp(&total_a)
    });

    for (lang, stats) in langs {
        println!(
            "{:<14} {:>10} {:>10} {:>10} {:>10}",
            lang,
            format_diff_count(stats.same.code),
            format_diff_count(stats.modified.code),
            format_diff_count(stats.added.code),
            format_diff_count(stats.removed.code),
        );
    }

    println!("{}", "─".repeat(58));
    println!(
        "{:<14} {:>10} {:>10} {:>10} {:>10}",
        "SUM",
        format_diff_count(result.totals.same.code),
        format_diff_count(result.totals.modified.code),
        format_diff_count(result.totals.added.code),
        format_diff_count(result.totals.removed.code),
    );
}

fn format_diff_count(n: u64) -> String {
    if n == 0 {
        "-".to_string()
    } else {
        n.to_string()
    }
}
