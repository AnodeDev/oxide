use std::path::{Path, PathBuf};
use std::fs;

use crate::buffer::{Cursor, Error};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub enum CommandLineState {
    #[default]
    Default,
    FindFile,
    SwitchBuffer,
    Error,
}

#[derive(Default)]
pub struct CommandLineManager {
    pub state: CommandLineState,
    pub content: Vec<String>,
    pub prefix: String,
    pub input: String,
    pub suffix: String,
    pub cursor: Cursor,
}

impl CommandLineManager {
    async fn load_directory(&mut self) -> Result<()> {
        let mut path = Path::new(&self.input).to_path_buf();

        if !path.exists() {
            path.pop();
        }

        self.content = Vec::new();

        if path.is_dir() {
            match fs::read_dir(path.clone()) {
                Ok(entries) => {
                    let content = entries
                        .map(|res| res.map(|e| e.path()))
                        .collect::<std::result::Result<Vec<_>, std::io::Error>>();

                    match content {
                        Ok(content) => {
                            let mut content_filtered: Vec<&PathBuf> = content
                                .iter()
                                .filter(|path| path.to_string_lossy().into_owned().contains(&self.suffix))
                                .collect();

                            content_filtered.sort();

                            let directories: Vec<&PathBuf> = content_filtered
                                .iter()
                               .filter(|entry| entry.is_dir())
                                .map(|path| *path)
                                .collect();
                            let files: Vec<&PathBuf> = content_filtered
                                .iter()
                                .filter(|entry| entry.is_file())
                                .map(|path| *path)
                                .collect();

                            let mut dot_count = 0;

                            for dir in &directories {
                                if let Some(name) = dir.file_name() {
                                    let name = name.to_string_lossy().into_owned();
                                    path.push(&name);

                                    let path_str = path.to_string_lossy().into_owned();

                                    if name.chars().nth(0) == Some('.') {
                                        self.content.insert(dot_count, path_str);

                                        dot_count += 1;
                                    } else {
                                        self.content.push(path_str);
                                    }

                                    path.pop();
                                }
                            }

                            dot_count = directories.len();

                            for file in files {
                                if let Some(name) = file.file_name() {
                                    let name = name.to_string_lossy().into_owned();
                                    path.push(&name);

                                    let path_str = path.to_string_lossy().into_owned();

                                    if name.chars().nth(0) == Some('.') {
                                        self.content.insert(dot_count, path_str);

                                        dot_count += 1;
                                    } else {
                                        self.content.push(path_str);
                                    }

                                    path.pop();
                                }
                            }
                        },
                        Err(e) => return Err(Error::IoError(e)),
                    }
                },
                Err(e) => return Err(Error::IoError(e)),
            }
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.prefix = String::new();
        self.input = String::new();
        self.suffix = String::new();
        self.state = CommandLineState::Default;
        self.content = Vec::new();
        self.cursor = Cursor::default();
    }

    pub fn display_error(&mut self, error_msg: String) {
        self.clear();

        self.state = CommandLineState::Error;
        self.prefix = error_msg;

        log::info!("COMMAND LINE: {}", self.prefix);
    }

    pub fn move_cursor(&mut self, x: i32, y: i32) {
        // Sets the new y value.
        // Clamp is used to make sure it doesn't exceed the length of the line or 0.
        let new_y = (self.cursor.y as i32 + y).clamp(0, self.content.len() as i32 - 1) as usize;
        self.cursor.y = new_y;

        // Checks if cursor is moved horiozontally.
        // If not, it checks if x is larger than the current lines length and adjusts accordingly.
        if x != 0 {
            let current_line_len = self.input.len();
            let new_x = (self.cursor.x as i32 + x).clamp(1, current_line_len as i32 + 1) as usize;

            self.cursor.x = new_x;
            self.cursor.desired_x = new_x;
        }
    }
    pub fn add_char(&mut self, character: char) -> Result<()> {
        if self.state == CommandLineState::Default {
            self.input.insert(self.cursor.x - self.prefix.len(), character);
            self.cursor.x += 1;
        } else if self.state == CommandLineState::SwitchBuffer {
            self.suffix.insert(self.cursor.x - self.prefix.len() - self.input.len(), character);
            self.cursor.x += 1;

        } else if self.state == CommandLineState::FindFile {
            self.suffix.insert(self.cursor.x - self.prefix.len() - self.input.len(), character);
            self.cursor.x += 1;

            if self.suffix.chars().last() == Some('/') && Path::new(&format!("{}{}", self.input, self.suffix)).exists() {
                self.input.push_str(&self.suffix);
                self.suffix = String::new();
                self.cursor.y = 0;
            }

            let tokio_runtime = tokio::runtime::Runtime::new()?;

            tokio_runtime.block_on(self.load_directory())?;
        }

        Ok(())
    }
    pub fn remove_char(&mut self) -> Result<()> {
        if self.cursor.x >= self.prefix.len() + 1 && self.cursor.x - (self.prefix.len() + 1) - self.suffix.len() < self.input.len() {
            if self.state == CommandLineState::FindFile && self.suffix.is_empty() {
                let path = Path::new(&self.input);

                let mut path = path.to_path_buf();

                path.pop();

                let path_str = path.to_string_lossy().into_owned();
                if path_str != "/".to_string() {
                    self.input = format!("{}/", path_str);
                } else {
                    self.input = "/".to_string();
                }


                self.cursor.x = self.prefix.len() + self.input.len();
                self.cursor.y = 0;
            } else if self.state == CommandLineState::FindFile {
                self.suffix.remove(self.cursor.x - (self.prefix.len() + 1) - self.input.len());
                self.cursor.x -= 1;
            } else  {
                self.input.remove(self.cursor.x - (self.prefix.len() + 1));
                self.cursor.x -= 1;
            }

            let tokio_runtime = tokio::runtime::Runtime::new()?;

            tokio_runtime.block_on(self.load_directory())?;
        }

        Ok(())
    }
}
