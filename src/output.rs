use crate::stats::{JsonOutput, LanguageStats, Summary};
use comfy_table::{
    presets::UTF8_FULL_CONDENSED, Attribute, Cell, Color, ContentArrangement, Table,
};
use std::io::{self, Write};

fn apply_summary_cutoff(languages: &[LanguageStats], cutoff: usize) -> Vec<LanguageStats> {
    let mut kept: Vec<LanguageStats> = Vec::new();
    let mut other = LanguageStats {
        name: "Other".to_string(),
        files: 0,
        code: 0,
        comments: 0,
        blanks: 0,
    };

    for lang in languages {
        if lang.files as usize >= cutoff {
            kept.push(lang.clone());
        } else {
            other.files += lang.files;
            other.code += lang.code;
            other.comments += lang.comments;
            other.blanks += lang.blanks;
        }
    }

    if other.files > 0 {
        kept.push(other);
    }

    kept
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Csv,
    Yaml,
    Markdown,
    Sql,
    Xml,
}

#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub format: OutputFormat,
    pub by_file: bool,
    pub by_file_by_lang: bool,
    pub hide_rate: bool,
    pub sort_by: SortBy,
    pub show_total_column: bool,
    pub csv_delimiter: u8,
    pub by_percent: bool,
    pub summary_cutoff: Option<usize>,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Table,
            by_file: false,
            by_file_by_lang: false,
            hide_rate: false,
            sort_by: SortBy::Code,
            show_total_column: false,
            csv_delimiter: b',',
            by_percent: false,
            summary_cutoff: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortBy {
    Language,
    Files,
    #[default]
    Code,
    Comments,
    Blanks,
    Total,
}

pub fn render(summary: &Summary, config: &OutputConfig) -> io::Result<()> {
    let mut stdout = io::stdout().lock();

    match config.format {
        OutputFormat::Table => render_table(summary, config, &mut stdout),
        OutputFormat::Json => render_json(summary, config, &mut stdout),
        OutputFormat::Csv => render_csv(summary, config, &mut stdout),
        OutputFormat::Yaml => render_yaml(summary, config, &mut stdout),
        OutputFormat::Markdown => render_markdown(summary, config, &mut stdout),
        OutputFormat::Sql => render_sql(summary, config, &mut stdout),
        OutputFormat::Xml => render_xml(summary, config, &mut stdout),
    }
}

fn render_table(summary: &Summary, config: &OutputConfig, out: &mut impl Write) -> io::Result<()> {
    if !config.hide_rate {
        if let Some(elapsed) = summary.elapsed {
            writeln!(out)?;
            write!(
                out,
                "{} files processed in {:.3}s",
                summary.total_files,
                elapsed.as_secs_f64()
            )?;
            if let (Some(fps), Some(lps)) = (summary.files_per_second(), summary.lines_per_second())
            {
                write!(out, " ({:.0} files/s, {:.0} lines/s)", fps, lps)?;
            }
            writeln!(out)?;
        }
    }

    if config.by_file || config.by_file_by_lang {
        render_by_file_table(summary, config, out)?;
    }

    if !config.by_file || config.by_file_by_lang {
        render_language_table(summary, config, out)?;
    }

    Ok(())
}

fn render_language_table(
    summary: &Summary,
    config: &OutputConfig,
    out: &mut impl Write,
) -> io::Result<()> {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic);

    let mut headers = vec![
        Cell::new("Language").add_attribute(Attribute::Bold),
        Cell::new("Files").add_attribute(Attribute::Bold),
        Cell::new("Blank").add_attribute(Attribute::Bold),
        Cell::new("Comment").add_attribute(Attribute::Bold),
        Cell::new("Code").add_attribute(Attribute::Bold),
    ];

    if config.show_total_column {
        headers.push(Cell::new("Total").add_attribute(Attribute::Bold));
    }

    table.set_header(headers);

    let mut languages = if let Some(cutoff) = config.summary_cutoff {
        apply_summary_cutoff(&summary.languages, cutoff)
    } else {
        summary.languages.clone()
    };
    match config.sort_by {
        SortBy::Language => languages.sort_by(|a, b| a.name.cmp(&b.name)),
        SortBy::Files => languages.sort_by(|a, b| b.files.cmp(&a.files)),
        SortBy::Code => languages.sort_by(|a, b| b.code.cmp(&a.code)),
        SortBy::Comments => languages.sort_by(|a, b| b.comments.cmp(&a.comments)),
        SortBy::Blanks => languages.sort_by(|a, b| b.blanks.cmp(&a.blanks)),
        SortBy::Total => languages.sort_by_key(|l| std::cmp::Reverse(l.total())),
    }

    for lang in &languages {
        let mut row = if config.by_percent {
            vec![
                Cell::new(&lang.name),
                Cell::new(format_percent(lang.files, summary.total_files)),
                Cell::new(format_percent(lang.blanks, summary.total_blanks)),
                Cell::new(format_percent(lang.comments, summary.total_comments)),
                Cell::new(format_percent(lang.code, summary.total_code)).fg(Color::Green),
            ]
        } else {
            vec![
                Cell::new(&lang.name),
                Cell::new(lang.files),
                Cell::new(lang.blanks),
                Cell::new(lang.comments),
                Cell::new(lang.code).fg(Color::Green),
            ]
        };

        if config.show_total_column {
            if config.by_percent {
                row.push(Cell::new(format_percent(
                    lang.total(),
                    summary.total_lines(),
                )));
            } else {
                row.push(Cell::new(lang.total()));
            }
        }

        table.add_row(row);
    }

    let mut sum_row = if config.by_percent {
        vec![
            Cell::new("SUM").add_attribute(Attribute::Bold),
            Cell::new("100.00%").add_attribute(Attribute::Bold),
            Cell::new("100.00%").add_attribute(Attribute::Bold),
            Cell::new("100.00%").add_attribute(Attribute::Bold),
            Cell::new("100.00%")
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
        ]
    } else {
        vec![
            Cell::new("SUM").add_attribute(Attribute::Bold),
            Cell::new(summary.total_files).add_attribute(Attribute::Bold),
            Cell::new(summary.total_blanks).add_attribute(Attribute::Bold),
            Cell::new(summary.total_comments).add_attribute(Attribute::Bold),
            Cell::new(summary.total_code)
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
        ]
    };

    if config.show_total_column {
        if config.by_percent {
            sum_row.push(Cell::new("100.00%").add_attribute(Attribute::Bold));
        } else {
            sum_row.push(Cell::new(summary.total_lines()).add_attribute(Attribute::Bold));
        }
    }

    table.add_row(sum_row);

    writeln!(out)?;
    writeln!(out, "{}", table)?;

    Ok(())
}

fn render_by_file_table(
    summary: &Summary,
    _config: &OutputConfig,
    out: &mut impl Write,
) -> io::Result<()> {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        Cell::new("File").add_attribute(Attribute::Bold),
        Cell::new("Language").add_attribute(Attribute::Bold),
        Cell::new("Blank").add_attribute(Attribute::Bold),
        Cell::new("Comment").add_attribute(Attribute::Bold),
        Cell::new("Code").add_attribute(Attribute::Bold),
    ]);

    let mut files = summary.file_stats.clone();
    files.sort_by(|a, b| b.code.cmp(&a.code));

    for file in &files {
        table.add_row(vec![
            Cell::new(&file.path),
            Cell::new(&file.language),
            Cell::new(file.blanks),
            Cell::new(file.comments),
            Cell::new(file.code).fg(Color::Green),
        ]);
    }

    writeln!(out)?;
    writeln!(out, "{}", table)?;

    Ok(())
}

fn render_json(summary: &Summary, _config: &OutputConfig, out: &mut impl Write) -> io::Result<()> {
    let output = JsonOutput::from(summary);
    let json = serde_json::to_string_pretty(&output).map_err(io::Error::other)?;
    writeln!(out, "{}", json)?;
    Ok(())
}

fn render_csv(summary: &Summary, config: &OutputConfig, out: &mut impl Write) -> io::Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(config.csv_delimiter)
        .from_writer(out);

    if config.by_file {
        writer.write_record(["File", "Language", "Blank", "Comment", "Code"])?;
        for file in &summary.file_stats {
            writer.write_record([
                &file.path,
                &file.language,
                &file.blanks.to_string(),
                &file.comments.to_string(),
                &file.code.to_string(),
            ])?;
        }
    } else {
        let languages = if let Some(cutoff) = config.summary_cutoff {
            apply_summary_cutoff(&summary.languages, cutoff)
        } else {
            summary.languages.clone()
        };
        writer.write_record(["Language", "Files", "Blank", "Comment", "Code"])?;
        for lang in &languages {
            writer.write_record([
                &lang.name,
                &lang.files.to_string(),
                &lang.blanks.to_string(),
                &lang.comments.to_string(),
                &lang.code.to_string(),
            ])?;
        }
        writer.write_record([
            "SUM",
            &summary.total_files.to_string(),
            &summary.total_blanks.to_string(),
            &summary.total_comments.to_string(),
            &summary.total_code.to_string(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

fn render_yaml(summary: &Summary, _config: &OutputConfig, out: &mut impl Write) -> io::Result<()> {
    let output = JsonOutput::from(summary);
    let yaml = serde_yaml::to_string(&output).map_err(io::Error::other)?;
    write!(out, "{}", yaml)?;
    Ok(())
}

fn render_markdown(
    summary: &Summary,
    config: &OutputConfig,
    out: &mut impl Write,
) -> io::Result<()> {
    if !config.hide_rate {
        if let Some(elapsed) = summary.elapsed {
            writeln!(out)?;
            writeln!(
                out,
                "**{} files** processed in **{:.3}s**",
                summary.total_files,
                elapsed.as_secs_f64()
            )?;
            writeln!(out)?;
        }
    }

    if config.by_file {
        writeln!(out, "| File | Language | Blank | Comment | Code |")?;
        writeln!(out, "|------|----------|------:|--------:|-----:|")?;
        for file in &summary.file_stats {
            writeln!(
                out,
                "| {} | {} | {} | {} | {} |",
                file.path, file.language, file.blanks, file.comments, file.code
            )?;
        }
    } else {
        let languages = if let Some(cutoff) = config.summary_cutoff {
            apply_summary_cutoff(&summary.languages, cutoff)
        } else {
            summary.languages.clone()
        };
        let mut headers = vec!["Language", "Files", "Blank", "Comment", "Code"];
        let mut alignments = vec![":---", "---:", "---:", "---:", "---:"];

        if config.show_total_column {
            headers.push("Total");
            alignments.push("---:");
        }

        writeln!(out, "| {} |", headers.join(" | "))?;
        writeln!(out, "| {} |", alignments.join(" | "))?;

        for lang in &languages {
            let mut row = format!(
                "| {} | {} | {} | {} | {}",
                lang.name, lang.files, lang.blanks, lang.comments, lang.code
            );
            if config.show_total_column {
                row.push_str(&format!(" | {}", lang.total()));
            }
            writeln!(out, "{} |", row)?;
        }

        let mut sum_row = format!(
            "| **SUM** | **{}** | **{}** | **{}** | **{}**",
            summary.total_files, summary.total_blanks, summary.total_comments, summary.total_code
        );
        if config.show_total_column {
            sum_row.push_str(&format!(" | **{}**", summary.total_lines()));
        }
        writeln!(out, "{} |", sum_row)?;
    }

    Ok(())
}

fn render_sql(summary: &Summary, config: &OutputConfig, out: &mut impl Write) -> io::Result<()> {
    // Create table
    if config.by_file {
        writeln!(out, "CREATE TABLE t (")?;
        writeln!(out, "    File TEXT,")?;
        writeln!(out, "    Language TEXT,")?;
        writeln!(out, "    nBlank INTEGER,")?;
        writeln!(out, "    nComment INTEGER,")?;
        writeln!(out, "    nCode INTEGER")?;
        writeln!(out, ");")?;
        writeln!(out)?;

        for file in &summary.file_stats {
            writeln!(
                out,
                "INSERT INTO t VALUES ('{}', '{}', {}, {}, {});",
                file.path.replace('\'', "''"),
                file.language.replace('\'', "''"),
                file.blanks,
                file.comments,
                file.code
            )?;
        }
    } else {
        let languages = if let Some(cutoff) = config.summary_cutoff {
            apply_summary_cutoff(&summary.languages, cutoff)
        } else {
            summary.languages.clone()
        };
        writeln!(out, "CREATE TABLE t (")?;
        writeln!(out, "    Language TEXT,")?;
        writeln!(out, "    nFiles INTEGER,")?;
        writeln!(out, "    nBlank INTEGER,")?;
        writeln!(out, "    nComment INTEGER,")?;
        writeln!(out, "    nCode INTEGER")?;
        writeln!(out, ");")?;
        writeln!(out)?;

        for lang in &languages {
            writeln!(
                out,
                "INSERT INTO t VALUES ('{}', {}, {}, {}, {});",
                lang.name.replace('\'', "''"),
                lang.files,
                lang.blanks,
                lang.comments,
                lang.code
            )?;
        }

        writeln!(
            out,
            "INSERT INTO t VALUES ('SUM', {}, {}, {}, {});",
            summary.total_files, summary.total_blanks, summary.total_comments, summary.total_code
        )?;
    }

    Ok(())
}

fn render_xml(summary: &Summary, config: &OutputConfig, out: &mut impl Write) -> io::Result<()> {
    writeln!(out, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(out, "<results>")?;

    if let Some(elapsed) = summary.elapsed {
        writeln!(out, "  <header>")?;
        writeln!(out, "    <n_files>{}</n_files>", summary.total_files)?;
        writeln!(out, "    <n_lines>{}</n_lines>", summary.total_lines())?;
        writeln!(
            out,
            "    <elapsed_seconds>{:.3}</elapsed_seconds>",
            elapsed.as_secs_f64()
        )?;
        writeln!(out, "  </header>")?;
    }

    if config.by_file {
        writeln!(out, "  <files>")?;
        for file in &summary.file_stats {
            writeln!(out, "    <file>")?;
            writeln!(out, "      <name>{}</name>", escape_xml(&file.path))?;
            writeln!(
                out,
                "      <language>{}</language>",
                escape_xml(&file.language)
            )?;
            writeln!(out, "      <blank>{}</blank>", file.blanks)?;
            writeln!(out, "      <comment>{}</comment>", file.comments)?;
            writeln!(out, "      <code>{}</code>", file.code)?;
            writeln!(out, "    </file>")?;
        }
        writeln!(out, "  </files>")?;
    } else {
        let languages = if let Some(cutoff) = config.summary_cutoff {
            apply_summary_cutoff(&summary.languages, cutoff)
        } else {
            summary.languages.clone()
        };
        writeln!(out, "  <languages>")?;
        for lang in &languages {
            writeln!(out, "    <language name=\"{}\">", escape_xml(&lang.name))?;
            writeln!(out, "      <files>{}</files>", lang.files)?;
            writeln!(out, "      <blank>{}</blank>", lang.blanks)?;
            writeln!(out, "      <comment>{}</comment>", lang.comments)?;
            writeln!(out, "      <code>{}</code>", lang.code)?;
            writeln!(out, "    </language>")?;
        }
        writeln!(out, "  </languages>")?;

        writeln!(out, "  <total>")?;
        writeln!(out, "    <files>{}</files>", summary.total_files)?;
        writeln!(out, "    <blank>{}</blank>", summary.total_blanks)?;
        writeln!(out, "    <comment>{}</comment>", summary.total_comments)?;
        writeln!(out, "    <code>{}</code>", summary.total_code)?;
        writeln!(out, "  </total>")?;
    }

    writeln!(out, "</results>")?;
    Ok(())
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn format_percent(value: u64, total: u64) -> String {
    if total == 0 {
        "0.00%".to_string()
    } else {
        format!("{:.2}%", (value as f64 / total as f64) * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counter::FileStats;

    fn sample_summary() -> Summary {
        Summary::from_file_stats(vec![FileStats {
            path: "main.rs".into(),
            language: "Rust".into(),
            code: 100,
            comments: 20,
            blanks: 10,
        }])
    }

    #[test]
    fn test_json_output() {
        let summary = sample_summary();
        let mut output = Vec::new();
        render_json(&summary, &OutputConfig::default(), &mut output).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert!(json.get("Rust").is_some());
        assert!(json.get("SUM").is_some());
    }

    #[test]
    fn test_csv_output() {
        let summary = sample_summary();
        let mut output = Vec::new();
        render_csv(&summary, &OutputConfig::default(), &mut output).unwrap();
        let csv = String::from_utf8(output).unwrap();
        assert!(csv.contains("Language"));
        assert!(csv.contains("Rust"));
        assert!(csv.contains("SUM"));
    }
}
