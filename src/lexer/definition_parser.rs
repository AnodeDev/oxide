use serde::Deserialize;
use toml;

use std::fs;

use crate::lexer::Error;

type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize, Debug)]
pub struct LanguageDefinition {
    pub language: LanguageMeta,
    pub keywords: Keywords,
    pub operators: Operators,
    pub comments: Comments,
    pub strings: Strings,
    pub numbers: Numbers,
    pub identifiers: Identifiers,
    pub theme: Theme,
}

#[derive(Deserialize, Debug)]
pub struct LanguageMeta {
    pub name: String,
    pub file_extensions: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Keywords {
    pub keywords: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Operators {
    pub single_char: Vec<String>,
    pub multi_char: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Comments {
    pub single_line: String,
    pub multi_line_start: String,
    pub multi_line_end: String,
}

#[derive(Deserialize, Debug)]
pub struct Strings {
    pub string_delimiters: Vec<String>,
    pub allow_escapes: bool,
    pub escape_characters: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Numbers {
    pub integer_regex: String,
    pub float_regex: String,
}

#[derive(Deserialize, Debug)]
pub struct Identifiers {
    pub regex: String,
}

#[derive(Deserialize, Debug)]
pub struct Theme {
    pub keywords: String,
    pub strings: String,
    pub comments: String,
    pub numbers: String,
    pub identifiers: String,
    pub operators: String,
}

pub fn load_language_definition(file_path: &str) -> Result<LanguageDefinition> {
    let content = fs::read_to_string(file_path)?;
    let definition = toml::from_str(&content)?;

    Ok(definition)
}
