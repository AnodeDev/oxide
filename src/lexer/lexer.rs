use regex::Regex;

use crate::lexer::definition_parser as def_parser;
use crate::lexer::Error;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    End,
    Invalid,
    Keyword,
    Identifier(String),
    Number(f64),
    LeftDelimiter,
    RightDelimiter,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Loc {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    kind: TokenKind,
    value: &'a str,
    loc: Loc,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    content: Vec<&'a str>,
    pub lang_def: def_parser::LanguageDefinition,
    cursor: Loc,
}

impl<'a> Lexer<'a> {
    pub fn new(content: Vec<&'a str>, lang_def: def_parser::LanguageDefinition) -> Lexer<'a> {
        Lexer {
            content,
            lang_def,
            cursor: Loc::default(),
        }
    }

    fn tokenize(&mut self) -> Result<()> {
        while self.cursor.y < self.content.len() {
            let line = self.content[self.cursor.y];
            let remaining = &line[self.cursor.x..];

            while remaining.starts_with(char::is_whitespace) {
                self.cursor.x += 1;
            }

            self.next_token(remaining)?;
        }

        Ok(())
    }

    fn next_token(&mut self, remaining: &'a str) -> Result<Token<'a>> {
        for keyword in &self.lang_def.keywords.keywords {
            if remaining.starts_with(keyword) {
                let len = keyword.len();
                let token = self.create_token(TokenKind::Keyword, &remaining[..len]);
                self.cursor.x += len;

                return Ok(token);
            }
        }

        for op in &self.lang_def.operators.multi_char {
            if remaining.starts_with(op) {
                let len = op.len();
                let token = self.create_token(TokenKind::LeftDelimiter, &remaining[..len]);
                self.cursor.x += len;

                return Ok(token);
            }
        }

        for op in &self.lang_def.operators.single_char {
            if remaining.starts_with(op) {
                let len = op.len();
                let token = self.create_token(TokenKind::RightDelimiter, &remaining[..len]);
                self.cursor.x += len;

                return Ok(token);
            }
        }

        let ident_regex = Regex::new(&self.lang_def.identifiers.regex)?;
        if let Some(mat) = ident_regex.find(remaining) {
            let len = mat.end();
            let token = self.create_token(
                TokenKind::Identifier(mat.as_str().to_string()),
                &remaining[..len],
            );
            self.cursor.x += len;

            return Ok(token);
        }

        let num_regex = Regex::new(&self.lang_def.numbers.float_regex)?;
        if let Some(mat) = num_regex.find(remaining) {
            let len = mat.end();
            let token = self.create_token(
                TokenKind::Number(mat.as_str().parse().unwrap_or(0.0)),
                &remaining[..len],
            );
            self.cursor.x += len;

            return Ok(token);
        }

        let token = self.create_token(TokenKind::Invalid, &remaining[..1]);
        self.cursor.x += 1;
        return Ok(token);
    }

    fn create_token(&self, kind: TokenKind, value: &'a str) -> Token<'a> {
        Token {
            kind,
            value,
            loc: self.cursor,
        }
    }
}
