use std::path::{Path, PathBuf};
use std::env;
use std::fs;

use crate::buffer::{Cursor, Error};
use crate::keybinding::Action;

// ╭──────────────────────────────────────╮
// │ Minibuffer Types                     │
// ╰──────────────────────────────────────╯

type Result<T> = std::result::Result<T, Error>;

// ╭──────────────────────────────────────╮
// │ Minibuffer Structs                   │
// ╰──────────────────────────────────────╯

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone)]
pub enum MinibufferKind {
    #[default]
    Nop,
    File(PathBuf),
    Buffer(Vec<String>),
}

#[derive(Default, Debug)]
pub struct Minibuffer {
    pub cursor: Cursor,
    pub input: String,
    pub matched_input: Vec<String>,
    pub prefix: String,
    pub content: Vec<String>,
    pub kind: MinibufferKind,
}

impl Minibuffer {
    pub fn fill(&mut self) -> Result<()> {
        let runtime = tokio::runtime::Runtime::new()?;
        let mut matches: Vec<String> = Vec::new();

        match &mut self.kind {
            MinibufferKind::File(ref mut path) => {
                if path.display().to_string().is_empty() {
                    *path = env::current_dir()?;
                    matches = runtime.block_on(read_dir(&path))?;

                    for dir in path.into_iter() {
                        self.matched_input.push(dir.to_string_lossy().to_string());
                    }

                    self.prefix = "Find File:".to_string();
                    self.cursor.x = self.matched_input.len();
                } else {
                    let entries = runtime.block_on(read_dir(&path))?;

                    for entry in entries {
                        if entry == self.input {
                            path.push(&entry);

                            if path.is_file() {
                                matches.push(entry);
                                break;
                            }

                            self.matched_input.push(entry);
                            self.input.clear();
                            self.cursor.x = self.matched_input.len();
                            self.fill()?;

                            return Ok(())
                        } else if entry.contains(&self.input) {
                            matches.push(entry);
                        }
                    }
                }

                let mut dirs: Vec<String> = Vec::new();
                let mut files: Vec<String> = Vec::new();

                for entry in &matches {
                    if Path::new(&format!("{}/{}", path.display(), entry)).is_dir() {
                        dirs.push(entry.to_string());
                    } else {
                        files.push(entry.to_string());
                    }
                }

                dirs.sort();
                files.sort();

                matches.clear();
                matches.append(&mut dirs);
                matches.append(&mut files);
            },
            MinibufferKind::Buffer(buffer_list) => {
                self.prefix = "Find Buffer:".to_string();

                for entry in buffer_list {
                    if entry.contains(&self.input) {
                        matches.push(entry.to_string());
                    }
                }

                matches.sort();
            },
            _ => {},
        }

        self.content = matches;

        Ok(())
    }

    pub fn append(&mut self) {
        if let Some(item) = self.content.get(self.cursor.y) {
            self.cursor.x += item.len() - self.input.len();
            self.input = item.to_string();
        }
    }

    pub fn execute(&mut self) -> Result<Option<Action>> {
        match &self.kind {
            MinibufferKind::File(path) => {
                if path.is_file() {
                    return Ok(Some(Action::OpenFile(path.clone())));
                }
            },
            MinibufferKind::Buffer(buffer_list) => {
                let item: &String = if self.content.len() > 1 {
                    if let Some(item) = &self.content.get(self.cursor.y) {
                        item
                    } else {
                        &String::from("")
                    }
                } else {
                    &self.input
                };

                for (num, entry) in buffer_list.iter().enumerate() {
                    if entry.contains(item) {
                        return Ok(Some(Action::OpenBuffer(num)));
                    }
                }

                return Err(Error::NoMatchError)
            },
            _ => {},
        }

        Ok(None)
    }
}

async fn read_dir(path: &PathBuf) -> Result<Vec<String>> {
    let mut entries: Vec<String> = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name() {
            entries.push(name.to_string_lossy().to_string());
        }
    }

    Ok(entries)
}
