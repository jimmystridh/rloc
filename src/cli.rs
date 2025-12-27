use crate::output::{OutputConfig, OutputFormat, SortBy};
use crate::walker::{VcsMode, WalkerConfig};
use clap::{Parser, ValueEnum};
use regex::Regex;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "rloc",
    author,
    version,
    about = "A fast, modern Rust implementation of cloc (Count Lines of Code)",
    long_about = "rloc counts lines of code, comments, and blanks in source files.\n\n\
                  It automatically detects programming languages by file extension\n\
                  and uses language-specific comment syntax for accurate counting."
)]
pub struct Cli {
    #[arg(
        value_name = "PATH",
        help = "Files or directories to analyze",
        default_value = "."
    )]
    pub paths: Vec<PathBuf>,

    #[arg(long, help = "Report results for every source file")]
    pub by_file: bool,

    #[arg(long, help = "Report by file and by language")]
    pub by_file_by_lang: bool,

    #[arg(long, value_enum, help = "Output format")]
    pub format: Option<Format>,

    #[arg(long, help = "Write results as JSON")]
    pub json: bool,

    #[arg(long, help = "Write results as CSV")]
    pub csv: bool,

    #[arg(long, value_name = "C", help = "Use character <C> as CSV delimiter (default: ',')")]
    pub csv_delimiter: Option<char>,

    #[arg(long, help = "Write results as YAML")]
    pub yaml: bool,

    #[arg(long, help = "Write results as Markdown")]
    pub md: bool,

    #[arg(long, help = "Write results as SQL CREATE and INSERT statements")]
    pub sql: bool,

    #[arg(long, help = "Write results as XML")]
    pub xml: bool,

    #[arg(long, value_name = "DIR", help = "Exclude directories matching these names")]
    pub exclude_dir: Vec<String>,

    #[arg(long, value_name = "EXT", help = "Exclude files with these extensions")]
    pub exclude_ext: Vec<String>,

    #[arg(long, value_name = "LANG", help = "Exclude these languages")]
    pub exclude_lang: Vec<String>,

    #[arg(long, value_name = "EXT", help = "Only count files with these extensions")]
    pub include_ext: Vec<String>,

    #[arg(long, value_name = "LANG", help = "Only count these languages")]
    pub include_lang: Vec<String>,

    #[arg(long, value_name = "LANG,EXT", help = "Treat files with extension EXT as language LANG (e.g. Rust,txt)")]
    pub force_lang: Vec<String>,

    #[arg(long, value_name = "REGEX", help = "Only count files in directories matching regex")]
    pub match_d: Option<String>,

    #[arg(long, value_name = "REGEX", help = "Exclude directories matching regex")]
    pub not_match_d: Vec<String>,

    #[arg(long, value_name = "REGEX", help = "Only count files matching regex")]
    pub match_f: Option<String>,

    #[arg(long, value_name = "REGEX", help = "Exclude files matching regex")]
    pub not_match_f: Vec<String>,

    #[arg(long, value_name = "REGEX", help = "Only count files containing content matching regex")]
    pub include_content: Option<String>,

    #[arg(long, value_name = "REGEX", help = "Exclude files containing content matching regex")]
    pub exclude_content: Option<String>,

    #[arg(long, help = "Use full path in regex matching")]
    pub fullpath: bool,

    #[arg(long, value_enum, help = "Use version control to find files")]
    pub vcs: Option<Vcs>,

    #[arg(long, help = "Synonym for --vcs")]
    pub files_from: Option<Vcs>,

    #[arg(long, help = "Follow symbolic links")]
    pub follow_symlinks: bool,

    #[arg(long, help = "Include hidden files and directories")]
    pub hidden: bool,

    #[arg(long, help = "Disable default directory exclusions (node_modules, target, etc.)")]
    pub no_ignore: bool,

    #[arg(long, help = "Don't respect .gitignore files")]
    pub skip_gitignore: bool,

    #[arg(long, help = "Skip file uniqueness check (count duplicate files multiple times)")]
    pub skip_uniqueness: bool,

    #[arg(long, help = "Include files in git submodules (requires Git 2.11+)")]
    pub include_submodules: bool,

    #[arg(long, value_name = "FILE", help = "Read file paths from FILE (one per line)")]
    pub list_file: Option<PathBuf>,

    #[arg(long, value_name = "N", help = "Maximum directory depth")]
    pub max_depth: Option<usize>,

    #[arg(long, help = "Do not recurse into subdirectories")]
    pub no_recurse: bool,

    #[arg(long, value_name = "MB", help = "Skip files larger than <MB> megabytes")]
    pub max_file_size: Option<u64>,

    #[arg(long, value_enum, default_value = "code", help = "Sort output by")]
    pub sort: SortField,

    #[arg(long, value_name = "N", help = "Aggregate languages with fewer than N files into 'Other'")]
    pub summary_cutoff: Option<usize>,

    #[arg(long, help = "Do not show rate statistics")]
    pub hide_rate: bool,

    #[arg(long, help = "Show counts as percentages of column totals")]
    pub by_percent: bool,

    #[arg(long, help = "Suppress progress output")]
    pub quiet: bool,

    #[arg(short, long, action = clap::ArgAction::Count, help = "Verbose output")]
    pub verbose: u8,

    #[arg(long, value_name = "FILE", help = "Write output to file")]
    pub out: Option<PathBuf>,

    #[arg(long, alias = "report-file", value_name = "FILE", help = "Write output to file")]
    pub report_file: Option<PathBuf>,

    #[arg(long, help = "Show an extra column with total lines")]
    pub show_total: bool,

    #[arg(long, help = "Print all known languages and exit")]
    pub show_lang: bool,

    #[arg(long, help = "Print all known file extensions and exit")]
    pub show_ext: bool,

    #[arg(long, value_name = "FILE", help = "Read and sum JSON reports from files")]
    pub sum_reports: Vec<PathBuf>,

    #[arg(long, value_name = "N", default_value = "0", help = "Number of threads (0 = auto)")]
    pub threads: usize,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum Format {
    Table,
    Json,
    Csv,
    Yaml,
    Md,
    Sql,
    Xml,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum Vcs {
    Auto,
    Git,
    None,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum SortField {
    Language,
    Files,
    Code,
    Comments,
    Blanks,
    Total,
}

impl Cli {
    pub fn to_walker_config(&self) -> Result<WalkerConfig, String> {
        let mut config = WalkerConfig::default();

        if !self.paths.is_empty() {
            config.paths = self.paths.clone();
        }

        config.list_file = self.list_file.clone();

        if self.no_ignore {
            config.exclude_dirs.clear();
        }

        config.exclude_dirs.extend(self.exclude_dir.iter().cloned());
        config.exclude_exts.extend(self.exclude_ext.iter().cloned());
        config.exclude_langs.extend(self.exclude_lang.iter().cloned());
        config.include_exts.extend(self.include_ext.iter().cloned());
        config.include_langs.extend(self.include_lang.iter().cloned());

        for spec in &self.force_lang {
            if let Some((lang, ext)) = spec.split_once(',') {
                config.force_lang.insert(ext.to_lowercase(), lang.to_string());
            } else {
                return Err(format!("Invalid --force-lang format '{}', expected LANG,EXT", spec));
            }
        }

        if let Some(ref pattern) = self.match_d {
            config.match_dir = Some(Regex::new(pattern).map_err(|e| format!("Invalid --match-d regex: {}", e))?);
        }

        for pattern in &self.not_match_d {
            config.not_match_dir.push(
                Regex::new(pattern).map_err(|e| format!("Invalid --not-match-d regex: {}", e))?
            );
        }

        if let Some(ref pattern) = self.match_f {
            config.match_file = Some(Regex::new(pattern).map_err(|e| format!("Invalid --match-f regex: {}", e))?);
        }

        for pattern in &self.not_match_f {
            config.not_match_file.push(
                Regex::new(pattern).map_err(|e| format!("Invalid --not-match-f regex: {}", e))?
            );
        }

        if let Some(ref pattern) = self.include_content {
            config.include_content = Some(Regex::new(pattern).map_err(|e| format!("Invalid --include-content regex: {}", e))?);
        }

        if let Some(ref pattern) = self.exclude_content {
            config.exclude_content = Some(Regex::new(pattern).map_err(|e| format!("Invalid --exclude-content regex: {}", e))?);
        }

        config.vcs = self.vcs.or(self.files_from).map(|v| match v {
            Vcs::Auto => VcsMode::Auto,
            Vcs::Git => VcsMode::Git,
            Vcs::None => VcsMode::None,
        });

        config.follow_symlinks = self.follow_symlinks;
        config.hidden = self.hidden;
        config.fullpath = self.fullpath;
        config.max_depth = if self.no_recurse { Some(1) } else { self.max_depth };
        config.skip_gitignore = self.skip_gitignore;
        config.skip_uniqueness = self.skip_uniqueness;
        config.include_submodules = self.include_submodules;
        config.max_file_size = self.max_file_size;

        Ok(config)
    }

    pub fn to_output_config(&self) -> OutputConfig {
        let format = if self.json {
            OutputFormat::Json
        } else if self.csv {
            OutputFormat::Csv
        } else if self.yaml {
            OutputFormat::Yaml
        } else if self.md {
            OutputFormat::Markdown
        } else if self.sql {
            OutputFormat::Sql
        } else if self.xml {
            OutputFormat::Xml
        } else {
            match self.format {
                Some(Format::Json) => OutputFormat::Json,
                Some(Format::Csv) => OutputFormat::Csv,
                Some(Format::Yaml) => OutputFormat::Yaml,
                Some(Format::Md) => OutputFormat::Markdown,
                Some(Format::Sql) => OutputFormat::Sql,
                Some(Format::Xml) => OutputFormat::Xml,
                Some(Format::Table) | None => OutputFormat::Table,
            }
        };

        let sort_by = match self.sort {
            SortField::Language => SortBy::Language,
            SortField::Files => SortBy::Files,
            SortField::Code => SortBy::Code,
            SortField::Comments => SortBy::Comments,
            SortField::Blanks => SortBy::Blanks,
            SortField::Total => SortBy::Total,
        };

        OutputConfig {
            format,
            by_file: self.by_file,
            by_file_by_lang: self.by_file_by_lang,
            hide_rate: self.hide_rate,
            sort_by,
            show_total_column: self.show_total,
            csv_delimiter: self.csv_delimiter.map(|c| c as u8).unwrap_or(b','),
            by_percent: self.by_percent,
            summary_cutoff: self.summary_cutoff,
        }
    }

    pub fn output_path(&self) -> Option<&PathBuf> {
        self.out.as_ref().or(self.report_file.as_ref())
    }
}

pub fn show_languages() {
    use crate::languages::list_languages;
    use comfy_table::{presets::UTF8_FULL_CONDENSED, Table};

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(["Language", "Line Comments", "Block Start", "Block End", "Nested"]);

    let mut langs: Vec<_> = list_languages().collect();
    langs.sort_by_key(|(name, _)| *name);

    for (name, lang) in langs {
        table.add_row([
            name,
            &lang.line_comments.join(", "),
            lang.block_comment_start.unwrap_or("-"),
            lang.block_comment_end.unwrap_or("-"),
            if lang.nested_comments { "yes" } else { "no" },
        ]);
    }

    println!("{}", table);
}

pub fn show_extensions() {
    use crate::languages::list_extensions;
    use comfy_table::{presets::UTF8_FULL_CONDENSED, Table};

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(["Extension", "Language"]);

    let mut exts: Vec<_> = list_extensions().collect();
    exts.sort_by_key(|(ext, _)| *ext);

    for (ext, lang) in exts {
        table.add_row([ext, lang]);
    }

    println!("{}", table);
}
