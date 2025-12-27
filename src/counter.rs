use crate::languages::Language;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct FileStats {
    pub path: String,
    pub language: String,
    pub code: u64,
    pub comments: u64,
    pub blanks: u64,
}

impl FileStats {
    pub fn total(&self) -> u64 {
        self.code + self.comments + self.blanks
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Code,
    BlockComment { depth: u32 },
    String { delimiter: char },
}

pub fn count_lines(path: &Path, language: &Language) -> std::io::Result<FileStats> {
    let file = File::open(path)?;

    if is_binary(&file)? {
        return Ok(FileStats {
            path: path.display().to_string(),
            language: language.name.to_string(),
            ..Default::default()
        });
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut stats = FileStats {
        path: path.display().to_string(),
        language: language.name.to_string(),
        ..Default::default()
    };

    let has_comments = !language.line_comments.is_empty() || language.block_comment_start.is_some();

    if !has_comments {
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            if line.trim().is_empty() {
                stats.blanks += 1;
            } else {
                stats.code += 1;
            }
        }
        return Ok(stats);
    }

    let mut state = State::Code;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        let trimmed = line.trim();

        if trimmed.is_empty() {
            if matches!(state, State::BlockComment { .. }) {
                stats.comments += 1;
            } else {
                stats.blanks += 1;
            }
            continue;
        }

        let (new_state, line_type) = classify_line(trimmed, state, language);
        state = new_state;

        match line_type {
            LineType::Code => stats.code += 1,
            LineType::Comment => stats.comments += 1,
            LineType::Mixed => {
                stats.code += 1;
            }
            LineType::Blank => stats.blanks += 1,
        }
    }

    Ok(stats)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LineType {
    Code,
    Comment,
    Mixed,
    Blank,
}

#[allow(unused_assignments)]
fn classify_line(line: &str, initial_state: State, lang: &Language) -> (State, LineType) {
    let mut state = initial_state;
    let mut has_code = false;
    let mut has_comment = matches!(state, State::BlockComment { .. });

    let mut chars = line.char_indices().peekable();

    while let Some((byte_idx, c)) = chars.next() {
        let remaining = &line[byte_idx..];

        match state {
            State::Code => {
                if c.is_whitespace() {
                    continue;
                }

                if let Some(block_start) = lang.block_comment_start
                    && remaining.starts_with(block_start) {
                        has_comment = true;
                        state = State::BlockComment { depth: 1 };
                        for _ in 0..block_start.chars().count().saturating_sub(1) {
                            chars.next();
                        }
                        continue;
                    }

                for &line_comment in lang.line_comments {
                    if remaining.starts_with(line_comment) {
                        has_comment = true;
                        return (State::Code, if has_code { LineType::Mixed } else { LineType::Comment });
                    }
                }

                if c == '"' || c == '\'' {
                    for &delim in lang.string_delimiters {
                        if remaining.starts_with(delim) && delim.len() == 1 {
                            has_code = true;
                            state = State::String { delimiter: c };
                            break;
                        }
                    }
                    if matches!(state, State::String { .. }) {
                        continue;
                    }
                }

                has_code = true;
            }

            State::BlockComment { depth } => {
                if let Some(block_end) = lang.block_comment_end
                    && remaining.starts_with(block_end) {
                        let new_depth = depth - 1;
                        if new_depth == 0 {
                            state = State::Code;
                        } else {
                            state = State::BlockComment { depth: new_depth };
                        }
                        for _ in 0..block_end.chars().count().saturating_sub(1) {
                            chars.next();
                        }
                        continue;
                    }

                if lang.nested_comments
                    && let Some(block_start) = lang.block_comment_start
                        && remaining.starts_with(block_start) {
                            state = State::BlockComment { depth: depth + 1 };
                            for _ in 0..block_start.chars().count().saturating_sub(1) {
                                chars.next();
                            }
                            continue;
                        }
            }

            State::String { delimiter } => {
                if c == '\\' {
                    chars.next();
                    continue;
                }
                if c == delimiter {
                    state = State::Code;
                }
            }
        }
    }

    if matches!(state, State::String { .. }) {
        state = State::Code;
    }

    let line_type = match (has_code, has_comment) {
        (true, true) => LineType::Mixed,
        (true, false) => LineType::Code,
        (false, true) => LineType::Comment,
        (false, false) => LineType::Blank,
    };

    (state, line_type)
}

fn is_binary(file: &File) -> std::io::Result<bool> {
    let mut buffer = [0u8; 8192];
    let mut handle = file.try_clone()?;
    let bytes_read = handle.read(&mut buffer)?;

    if bytes_read == 0 {
        return Ok(false);
    }

    let null_count = buffer[..bytes_read].iter().filter(|&&b| b == 0).count();
    let binary_threshold = bytes_read / 10;

    Ok(null_count > binary_threshold.max(1))
}

pub fn compute_file_hash(path: &Path) -> std::io::Result<u64> {
    let content = std::fs::read(path)?;
    let mut hasher = ahash::AHasher::default();
    content.hash(&mut hasher);
    Ok(hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::languages::LANGUAGES;

    #[test]
    fn test_c_style_comments() {
        let rust = LANGUAGES.get("Rust").unwrap();

        let cases = [
            ("let x = 5;", State::Code, LineType::Code),
            ("// comment", State::Code, LineType::Comment),
            ("let x = 5; // comment", State::Code, LineType::Mixed),
            ("/* block */", State::Code, LineType::Comment),
            ("/* start", State::BlockComment { depth: 1 }, LineType::Comment),
        ];

        for (line, expected_state, expected_type) in cases {
            let (state, line_type) = classify_line(line, State::Code, rust);
            assert_eq!(state, expected_state, "Failed state for: {}", line);
            assert_eq!(line_type, expected_type, "Failed type for: {}", line);
        }
    }

    #[test]
    fn test_nested_comments() {
        let rust = LANGUAGES.get("Rust").unwrap();
        assert!(rust.nested_comments);

        let (state, _) = classify_line("/* outer /* inner */", State::Code, rust);
        assert_eq!(state, State::BlockComment { depth: 1 });
    }
}
