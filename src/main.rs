mod cli;
mod counter;
mod languages;
mod output;
mod stats;
mod walker;

use clap::Parser;
use cli::Cli;
use counter::count_lines;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use output::render;
use rayon::prelude::*;
use stats::Summary;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::process::ExitCode;
use std::time::Instant;
use walker::walk_files;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.show_lang {
        cli::show_languages();
        return Ok(());
    }

    if cli.show_ext {
        cli::show_extensions();
        return Ok(());
    }

    if cli.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(cli.threads)
            .build_global()
            .ok();
    }

    let walker_config = cli.to_walker_config()?;
    let output_config = cli.to_output_config();

    let start = Instant::now();

    let files = walk_files(&walker_config);

    if files.is_empty() {
        if !cli.quiet {
            eprintln!("No source files found.");
        }
        return Ok(());
    }

    let file_count = files.len();

    let progress = if cli.quiet || output_config.format != output::OutputFormat::Table {
        ProgressBar::hidden()
    } else {
        let pb = ProgressBar::new(file_count as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({per_sec})")
                .unwrap()
                .progress_chars("=>-"),
        );
        pb
    };

    let file_stats: Vec<_> = files
        .into_par_iter()
        .progress_with(progress.clone())
        .filter_map(|entry| {
            match count_lines(&entry.path, entry.language) {
                Ok(stats) if stats.total() > 0 => Some(stats),
                Ok(_) => None,
                Err(e) => {
                    if cli.verbose > 0 {
                        eprintln!("warning: {}: {}", entry.path.display(), e);
                    }
                    None
                }
            }
        })
        .collect();

    progress.finish_and_clear();

    let elapsed = start.elapsed();
    let summary = Summary::from_file_stats(file_stats).with_elapsed(elapsed);

    if let Some(output_path) = cli.output_path() {
        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);
        render_to_writer(&summary, &output_config, &mut writer)?;
        writer.flush()?;
    } else {
        render(&summary, &output_config)?;
    }

    Ok(())
}

fn render_to_writer(
    summary: &Summary,
    config: &output::OutputConfig,
    out: &mut impl Write,
) -> io::Result<()> {
    use output::OutputFormat;

    match config.format {
        OutputFormat::Table => {
            if !config.hide_rate
                && let Some(elapsed) = summary.elapsed {
                    writeln!(out)?;
                    write!(out, "{} files processed in {:.3}s", summary.total_files, elapsed.as_secs_f64())?;
                    if let (Some(fps), Some(lps)) = (summary.files_per_second(), summary.lines_per_second()) {
                        write!(out, " ({:.0} files/s, {:.0} lines/s)", fps, lps)?;
                    }
                    writeln!(out)?;
                }

            writeln!(out)?;
            writeln!(out, "Language       Files    Blank  Comment     Code")?;
            writeln!(out, "─────────────────────────────────────────────────")?;

            for lang in &summary.languages {
                writeln!(
                    out,
                    "{:<14} {:>5} {:>8} {:>8} {:>8}",
                    lang.name, lang.files, lang.blanks, lang.comments, lang.code
                )?;
            }

            writeln!(out, "─────────────────────────────────────────────────")?;
            writeln!(
                out,
                "{:<14} {:>5} {:>8} {:>8} {:>8}",
                "SUM", summary.total_files, summary.total_blanks, summary.total_comments, summary.total_code
            )?;
            Ok(())
        }
        OutputFormat::Json => {
            let output = stats::JsonOutput::from(summary);
            let json = serde_json::to_string_pretty(&output)
                .map_err(io::Error::other)?;
            writeln!(out, "{}", json)
        }
        OutputFormat::Csv => {
            let mut writer = csv::Writer::from_writer(out);
            writer.write_record(["Language", "Files", "Blank", "Comment", "Code"])?;
            for lang in &summary.languages {
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
            writer.flush()?;
            Ok(())
        }
        OutputFormat::Yaml => {
            let output = stats::JsonOutput::from(summary);
            let yaml = serde_yaml::to_string(&output)
                .map_err(io::Error::other)?;
            write!(out, "{}", yaml)
        }
        OutputFormat::Markdown => {
            writeln!(out, "| Language | Files | Blank | Comment | Code |")?;
            writeln!(out, "|----------|------:|------:|--------:|-----:|")?;
            for lang in &summary.languages {
                writeln!(
                    out,
                    "| {} | {} | {} | {} | {} |",
                    lang.name, lang.files, lang.blanks, lang.comments, lang.code
                )?;
            }
            writeln!(
                out,
                "| **SUM** | **{}** | **{}** | **{}** | **{}** |",
                summary.total_files, summary.total_blanks, summary.total_comments, summary.total_code
            )
        }
        OutputFormat::Sql => {
            writeln!(out, "CREATE TABLE t (")?;
            writeln!(out, "    Language TEXT,")?;
            writeln!(out, "    nFiles INTEGER,")?;
            writeln!(out, "    nBlank INTEGER,")?;
            writeln!(out, "    nComment INTEGER,")?;
            writeln!(out, "    nCode INTEGER")?;
            writeln!(out, ");")?;
            writeln!(out)?;

            for lang in &summary.languages {
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
                summary.total_files,
                summary.total_blanks,
                summary.total_comments,
                summary.total_code
            )
        }
        OutputFormat::Xml => {
            writeln!(out, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
            writeln!(out, "<results>")?;

            if let Some(elapsed) = summary.elapsed {
                writeln!(out, "  <header>")?;
                writeln!(out, "    <n_files>{}</n_files>", summary.total_files)?;
                writeln!(out, "    <n_lines>{}</n_lines>", summary.total_lines())?;
                writeln!(out, "    <elapsed_seconds>{:.3}</elapsed_seconds>", elapsed.as_secs_f64())?;
                writeln!(out, "  </header>")?;
            }

            writeln!(out, "  <languages>")?;
            for lang in &summary.languages {
                let escaped_name = lang.name.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;");
                writeln!(out, "    <language name=\"{}\">", escaped_name)?;
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

            writeln!(out, "</results>")
        }
    }
}
