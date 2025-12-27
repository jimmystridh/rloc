use crate::languages::Language;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
pub struct CustomLanguageDef {
    pub extensions: Vec<String>,
    #[serde(default)]
    pub line_comments: Vec<String>,
    pub block_comment_start: Option<String>,
    pub block_comment_end: Option<String>,
    #[serde(default)]
    pub nested_comments: bool,
    #[serde(default = "default_string_delimiters")]
    pub string_delimiters: Vec<String>,
}

fn default_string_delimiters() -> Vec<String> {
    vec!["\"".to_string(), "'".to_string()]
}

static CUSTOM_LANGUAGES: OnceLock<CustomLanguages> = OnceLock::new();

pub struct CustomLanguages {
    languages: HashMap<String, &'static Language>,
    extensions: HashMap<String, String>,
}

impl CustomLanguages {
    pub fn load(path: &Path) -> Result<(), String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        let defs: HashMap<String, CustomLanguageDef> = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;

        let mut languages = HashMap::new();
        let mut extensions = HashMap::new();

        for (name, def) in defs {
            for ext in &def.extensions {
                extensions.insert(ext.to_lowercase(), name.clone());
            }

            let lang = Box::leak(Box::new(Language {
                name: Box::leak(name.clone().into_boxed_str()),
                line_comments: Box::leak(
                    def.line_comments
                        .into_iter()
                        .map(|s| Box::leak(s.into_boxed_str()) as &'static str)
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                ),
                block_comment_start: def.block_comment_start
                    .map(|s| Box::leak(s.into_boxed_str()) as &'static str),
                block_comment_end: def.block_comment_end
                    .map(|s| Box::leak(s.into_boxed_str()) as &'static str),
                nested_comments: def.nested_comments,
                string_delimiters: Box::leak(
                    def.string_delimiters
                        .into_iter()
                        .map(|s| Box::leak(s.into_boxed_str()) as &'static str)
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                ),
                raw_string_start: None,
                raw_string_end: None,
            }));

            languages.insert(name, lang as &'static Language);
        }

        CUSTOM_LANGUAGES
            .set(CustomLanguages { languages, extensions })
            .map_err(|_| "Custom languages already loaded".to_string())?;

        Ok(())
    }

    pub fn get_by_extension(ext: &str) -> Option<&'static Language> {
        let custom = CUSTOM_LANGUAGES.get()?;
        let lang_name = custom.extensions.get(&ext.to_lowercase())?;
        custom.languages.get(lang_name).copied()
    }
}
