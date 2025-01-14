use std::boxed::Box;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::buffer::{Cursor, Error};
use crate::keybinding::actions::{self, Action};

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
                    *path = env::current_dir()
                        .map_err(|_| Error::InvalidPathError { path: path.clone() })?;
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
                                matches.clear();
                                matches.push(entry);
                                break;
                            }

                            self.matched_input.push(entry);
                            self.input.clear();
                            self.cursor.x = self.matched_input.len();
                            self.fill()?;

                            return Ok(());
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
            }
            MinibufferKind::Buffer(buffer_list) => {
                self.prefix = "Find Buffer:".to_string();

                for entry in buffer_list {
                    if entry.contains(&self.input) {
                        matches.push(entry.to_string());
                    }
                }

                matches.sort();
            }
            _ => {}
        }

        self.content = matches;

        Ok(())
    }

    pub fn append(&mut self) -> Result<()> {
        if let Some(item) = self.content.get(self.cursor.y) {
            if let MinibufferKind::File(path) = &self.kind {
                let mut test = path.clone();
                test.push(item);

                if !test.is_file() && !test.is_dir() {
                    return Err(Error::InvalidPathError {
                        path: test.to_path_buf(),
                    });
                }
            }

            self.cursor.y = 0;
            self.cursor.x += item.len() - self.input.len();
            self.input = item.to_string();
        }

        Ok(())
    }

    pub fn execute(&mut self) -> Result<Option<Box<dyn Action>>> {
        match &self.kind {
            MinibufferKind::File(path) => {
                if path.is_file() {
                    return Ok(Some(Box::new(actions::OpenFileAction::new(path.clone()))));
                } else if !path.is_dir() {
                    return Err(Error::InvalidPathError { path: path.clone() });
                }
            }
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
                        return Ok(Some(Box::new(actions::OpenBufferAction::new(num))));
                    }
                }

                return Err(Error::NoMatchError {
                    input: item.to_string(),
                });
            }
            _ => {}
        }

        Ok(None)
    }
}

async fn read_dir(path: &PathBuf) -> Result<Vec<String>> {
    let mut entries: Vec<String> = Vec::new();
    let content = fs::read_dir(path).map_err(|_| Error::InvalidPathError { path: path.clone() })?;

    for entry in content {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name() {
            entries.push(name.to_string_lossy().to_string());
        }
    }

    Ok(entries)
}
