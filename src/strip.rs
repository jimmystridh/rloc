use crate::counter::{classify_line, LineType, State};
use crate::languages::Language;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub enum StripMode {
    Comments,
    Code,
}

pub fn strip_file(
    path: &Path,
    language: &Language,
    mode: StripMode,
    output_ext: &str,
) -> std::io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let output_path = path.with_extension(output_ext);
    let mut output = File::create(&output_path)?;

    let mut state = State::Code;

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            writeln!(output)?;
            continue;
        }

        let (new_state, line_type) = classify_line(trimmed, state, language);
        state = new_state;

        match mode {
            StripMode::Comments => {
                match line_type {
                    LineType::Code | LineType::Blank => writeln!(output, "{}", line)?,
                    LineType::Mixed => {
                        if let Some(stripped) = strip_comment_from_line(&line, language) {
                            writeln!(output, "{}", stripped)?;
                        } else {
                            writeln!(output, "{}", line)?;
                        }
                    }
                    LineType::Comment => {}
                }
            }
            StripMode::Code => {
                match line_type {
                    LineType::Comment => writeln!(output, "{}", line)?,
                    LineType::Mixed => {
                        if let Some(comment) = extract_comment_from_line(&line, language) {
                            writeln!(output, "{}", comment)?;
                        }
                    }
                    LineType::Code | LineType::Blank => {}
                }
            }
        }
    }

    Ok(())
}

fn strip_comment_from_line(line: &str, lang: &Language) -> Option<String> {
    for &comment_start in lang.line_comments {
        if let Some(pos) = line.find(comment_start) {
            let before = &line[..pos];
            if !before.trim().is_empty() {
                return Some(before.trim_end().to_string());
            }
        }
    }
    None
}

fn extract_comment_from_line(line: &str, lang: &Language) -> Option<String> {
    for &comment_start in lang.line_comments {
        if let Some(pos) = line.find(comment_start) {
            return Some(line[pos..].to_string());
        }
    }
    None
}
