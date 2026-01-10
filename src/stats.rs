use crate::counter::FileStats;
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Default, Serialize)]
pub struct LanguageStats {
    pub name: String,
    pub files: u64,
    pub code: u64,
    pub comments: u64,
    pub blanks: u64,
}

impl LanguageStats {
    pub fn total(&self) -> u64 {
        self.code + self.comments + self.blanks
    }

    pub fn add(&mut self, file_stats: &FileStats) {
        self.files += 1;
        self.code += file_stats.code;
        self.comments += file_stats.comments;
        self.blanks += file_stats.blanks;
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Summary {
    pub languages: Vec<LanguageStats>,
    pub total_files: u64,
    pub total_code: u64,
    pub total_comments: u64,
    pub total_blanks: u64,
    #[serde(skip)]
    pub elapsed: Option<Duration>,
    #[serde(skip)]
    pub file_stats: Vec<FileStats>,
}

impl Summary {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn total_lines(&self) -> u64 {
        self.total_code + self.total_comments + self.total_blanks
    }

    pub fn from_file_stats(stats: Vec<FileStats>) -> Self {
        let mut by_language: AHashMap<String, LanguageStats> = AHashMap::new();

        for file_stat in &stats {
            let entry = by_language
                .entry(file_stat.language.clone())
                .or_insert_with(|| LanguageStats {
                    name: file_stat.language.clone(),
                    ..Default::default()
                });
            entry.add(file_stat);
        }

        let mut languages: Vec<_> = by_language.into_values().collect();
        languages.sort_by(|a, b| b.code.cmp(&a.code));

        let total_files = languages.iter().map(|l| l.files).sum();
        let total_code = languages.iter().map(|l| l.code).sum();
        let total_comments = languages.iter().map(|l| l.comments).sum();
        let total_blanks = languages.iter().map(|l| l.blanks).sum();

        Summary {
            languages,
            total_files,
            total_code,
            total_comments,
            total_blanks,
            elapsed: None,
            file_stats: stats,
        }
    }

    pub fn with_elapsed(mut self, elapsed: Duration) -> Self {
        self.elapsed = Some(elapsed);
        self
    }

    pub fn lines_per_second(&self) -> Option<f64> {
        self.elapsed.map(|d| {
            let secs = d.as_secs_f64();
            if secs > 0.0 {
                self.total_lines() as f64 / secs
            } else {
                0.0
            }
        })
    }

    pub fn files_per_second(&self) -> Option<f64> {
        self.elapsed.map(|d| {
            let secs = d.as_secs_f64();
            if secs > 0.0 {
                self.total_files as f64 / secs
            } else {
                0.0
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonOutput {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub header: Option<JsonHeader>,
    #[serde(flatten)]
    pub languages: HashMap<String, JsonLanguageStats>,
    #[serde(rename = "SUM")]
    pub sum: JsonLanguageStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonHeader {
    pub cloc_version: String,
    pub elapsed_seconds: f64,
    pub n_files: u64,
    pub n_lines: u64,
    pub files_per_second: f64,
    pub lines_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsonLanguageStats {
    #[serde(rename = "nFiles")]
    pub n_files: u64,
    pub blank: u64,
    pub comment: u64,
    pub code: u64,
}

impl From<&Summary> for JsonOutput {
    fn from(summary: &Summary) -> Self {
        let header = summary.elapsed.map(|elapsed| JsonHeader {
            cloc_version: format!("rloc {}", env!("CARGO_PKG_VERSION")),
            elapsed_seconds: elapsed.as_secs_f64(),
            n_files: summary.total_files,
            n_lines: summary.total_lines(),
            files_per_second: summary.files_per_second().unwrap_or(0.0),
            lines_per_second: summary.lines_per_second().unwrap_or(0.0),
        });

        let languages: HashMap<String, JsonLanguageStats> = summary
            .languages
            .iter()
            .map(|lang| {
                (
                    lang.name.clone(),
                    JsonLanguageStats {
                        n_files: lang.files,
                        blank: lang.blanks,
                        comment: lang.comments,
                        code: lang.code,
                    },
                )
            })
            .collect();

        let sum = JsonLanguageStats {
            n_files: summary.total_files,
            blank: summary.total_blanks,
            comment: summary.total_comments,
            code: summary.total_code,
        };

        JsonOutput {
            header,
            languages,
            sum,
        }
    }
}

impl JsonOutput {
    pub fn sum_reports(reports: Vec<JsonOutput>) -> Self {
        let mut combined_langs: HashMap<String, JsonLanguageStats> = HashMap::new();
        let mut total_sum = JsonLanguageStats::default();

        for report in reports {
            for (name, stats) in report.languages {
                let entry = combined_langs.entry(name).or_default();
                entry.n_files += stats.n_files;
                entry.blank += stats.blank;
                entry.comment += stats.comment;
                entry.code += stats.code;
            }
            total_sum.n_files += report.sum.n_files;
            total_sum.blank += report.sum.blank;
            total_sum.comment += report.sum.comment;
            total_sum.code += report.sum.code;
        }

        JsonOutput {
            header: None,
            languages: combined_langs,
            sum: total_sum,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_aggregation() {
        let stats = vec![
            FileStats {
                path: "a.rs".into(),
                language: "Rust".into(),
                code: 100,
                comments: 20,
                blanks: 10,
            },
            FileStats {
                path: "b.rs".into(),
                language: "Rust".into(),
                code: 50,
                comments: 10,
                blanks: 5,
            },
            FileStats {
                path: "c.py".into(),
                language: "Python".into(),
                code: 30,
                comments: 5,
                blanks: 3,
            },
        ];

        let summary = Summary::from_file_stats(stats);

        assert_eq!(summary.total_files, 3);
        assert_eq!(summary.total_code, 180);
        assert_eq!(summary.total_comments, 35);
        assert_eq!(summary.total_blanks, 18);
        assert_eq!(summary.languages.len(), 2);
    }
}
