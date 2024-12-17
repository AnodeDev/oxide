use crate::buffer::{Cursor, Error};

type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub enum MinibufferKind {
    #[default]
    Command,
    File,
    Buffer,
}

#[derive(Default)]
pub struct Minibuffer {
    pub cursor: Cursor,
    pub input: String,
    pub matched_input: Vec<String>,
    pub prefix: String,
    pub content: Vec<String>,
    pub kind: MinibufferKind,
}
