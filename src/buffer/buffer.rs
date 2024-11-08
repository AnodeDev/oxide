use std::cell::RefCell;
use std::rc::Rc;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::fmt;

use crate::keybinding::{DeleteDirection, NewLineDirection};

pub trait Manipulation {
    fn move_cursor(&mut self, x: i32, y: i32);
    fn move_cursor_to_top(&mut self);
    fn move_cursor_to_bot(&mut self);
    fn add_char(&mut self, character: char);
    fn new_line(&mut self, direction: NewLineDirection);
    fn remove_char(&mut self, direction: DeleteDirection);
    fn delete_line(&mut self);
    fn get_command(&mut self) -> String;
}

pub enum ContentSource {
    None,
    File(PathBuf),
}

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

pub struct Cursor {
    pub x: usize,
    pub y: usize,
    pub desired_x: usize,
    pub desired_y: usize,
}

pub struct Viewport {
    pub top: usize,
    pub height: usize,
}

pub struct Buffer {
    pub title: &'static str,
    pub content: Vec<String>,
    source: ContentSource,
    pub cursor: Cursor,
    pub viewport: Viewport,
    pub mode: Mode,
    pub killable: bool,
    pub mutable: bool,
    pub commandline: String,
}

impl Cursor {
    fn new() -> Self {
        Cursor {
            x: 0,
            y: 0,
            desired_x: 0,
            desired_y: 0,
        }
    }
}

impl Viewport {
    fn new(height: usize) -> Self {
        Viewport {
            top: 0,
            height,
        }
    }

    pub fn bottom(&self) -> usize {
        self.top + self.height
    }

    fn adjust(&mut self, cursor_y: usize, content_len: usize) {
        if cursor_y < self.top {
            self.top = cursor_y;
        } else if cursor_y >= self.bottom() {
            self.top = cursor_y.saturating_sub(self.height) + 1;
        }

        if self.bottom() > content_len {
            self.top = content_len.saturating_sub(self.height);
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::Normal => write!(f, "NORMAL"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::Command => write!(f, "COMMAND"),
        }
    }
}

impl Buffer {
    pub fn new(title: &'static str, content: Vec<String>, source: ContentSource, height: usize, killable: bool, mutable: bool) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Buffer {
            title,
            content,
            source,
            cursor: Cursor::new(),
            viewport: Viewport::new(height - 2),
            mode: Mode::Normal,
            killable,
            mutable,
            commandline: String::new(),
        }))
    }

    pub fn scratch(height: usize) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Buffer {
            title: "*Scratch*",
            content: vec![
                "This is the scratch buffer".to_string(),
                "This buffer isn't connected to a file, so nothing in here is saved.".to_string(),
                "It's meant to be used to play around, sketch, and try new plugins.".to_string(),
                String::new(),
            ],
            source: ContentSource::None,
            cursor: Cursor::new(),
            viewport: Viewport::new(height - 2),
            mode: Mode::Normal,
            killable: false,
            mutable: false,
            commandline: String::new(),
        }))
    }

    pub async fn from_file(path_str: &'static str, height: usize) -> anyhow::Result<Rc<RefCell<Self>>> {
        let mut path = PathBuf::new();
        path.push(path_str);
        let file = File::open(path.clone())?;
        let mut buf_reader = BufReader::new(&file);
        let mut content = String::new();
        let mut file_name = "[NO NAME]";

        if let Some(name_osstr) = Path::new(path_str).file_name() {
            if let Some(name) = name_osstr.to_str() {
                file_name = name;
            }
        }
        buf_reader.read_to_string(&mut content)?;

        Ok(Rc::new(RefCell::new(Buffer {
            title: file_name,
            content: content.split("\n").map(|line| line.to_string()).collect(),
            source: ContentSource::File(path),
            cursor: Cursor::new(),
            viewport: Viewport::new(height - 2),
            mode: Mode::Normal,
            killable: true,
            mutable: true,
            commandline: String::new(),
        })))
    }

    pub async fn write_buffer(&mut self) -> anyhow::Result<()> {
        if !self.mutable {
            return Ok(())
        }

        match &mut self.source {
            ContentSource::File(path) => {
                let content_str = self.content.join("\n");
                let content_b = content_str.as_bytes();
                let mut file = File::create(path)?;

                file.write_all(content_b)?;
            },
            _ => {},
        }

        Ok(())
    }
}

impl Manipulation for Buffer {
    fn move_cursor(&mut self, x: i32, y: i32) {
        let new_y = (self.cursor.y as i32 + y).clamp(0, self.content.len() as i32 - 1) as usize;
        self.cursor.y = new_y;

        self.viewport.adjust(self.cursor.y, self.content.len());

        if x != 0 {
            let current_line_len = self.content[self.cursor.y].len();
            let new_x = (self.cursor.x as i32 + x).clamp(0, current_line_len as i32) as usize;

            self.cursor.x = new_x;
            self.cursor.desired_x = new_x;
        } else {
            let current_line_len = self.content[self.cursor.y].len();
            self.cursor.x = self.cursor.desired_x.min(current_line_len);
        }
     }

    fn move_cursor_to_top(&mut self) {
        self.cursor.x = 0;
        self.cursor.y = 0;

        self.viewport.adjust(self.cursor.y, self.content.len());
    }

    fn move_cursor_to_bot(&mut self) {
        self.cursor.x = 0;
        self.cursor.y = self.content.len() - 1;

        self.viewport.adjust(self.cursor.y, self.content.len());
    }

    fn add_char(&mut self, character: char) {
        let content: &mut String = match self.mode {
            Mode::Insert => {
                &mut self.content[self.cursor.y]
            },
            Mode::Command => {
                self.cursor.desired_y = self.cursor.y;
                self.cursor.y = 0;

                if self.cursor.x > self.commandline.len() {
                    self.cursor.desired_x = self.cursor.x;

                    self.cursor.x = self.commandline.len()
                }

                &mut self.commandline
            },
            Mode::Normal => todo!("Throw ERROR: Should never be Normal mode"),
        };

        content.insert(self.cursor.x, character);
        self.cursor.x += 1;
    }

    fn new_line(&mut self, direction: NewLineDirection) {
        match self.mode {
            Mode::Insert => {
                let remaining_text = self.content[self.cursor.y].split_off(self.cursor.x);
                self.content.insert(self.cursor.y + 1, remaining_text);
                self.cursor.y += 1;
                self.cursor.x = 0;
            },
            Mode::Normal => {
                match direction {
                    NewLineDirection::Under => {
                        self.content.insert(self.cursor.y + 1, String::new());
                        self.cursor.y += 1;
                        self.cursor.x = 0;
                    },
                    NewLineDirection::Over => {
                        self.content.insert(self.cursor.y, String::new());
                        self.cursor.x = 0;
                    },
                }

                self.mode = Mode::Insert;
            },
            _ => {},
        }
    }
    fn remove_char(&mut self, direction: DeleteDirection) {
        let content: &mut String = match self.mode {
            Mode::Insert | Mode::Normal => {
                &mut self.content[self.cursor.y]
            },
            Mode::Command => {
                &mut self.commandline
            },
        };

        match direction {
            DeleteDirection::Behind => {
                if self.cursor.x > 0 {
                    content.remove(self.cursor.x - 1);

                    self.cursor.x -= 1;
                } else if self.cursor.y > 0 && self.mode == Mode::Insert {
                    let current_line = self.content.remove(self.cursor.y);

                    self.cursor.y -= 1;
                    self.cursor.x = self.content[self.cursor.y].len();
                    self.content[self.cursor.y].push_str(&current_line);
                }
            },
            DeleteDirection::Under => {
                if self.cursor.x < content.len() {
                    content.remove(self.cursor.x);
                }
            },
            _ => {},
        }

    }

    fn delete_line(&mut self) {
        if self.content.len() > 1 {
            self.content.remove(self.cursor.y);

            if self.cursor.y > self.content.len() - 1 {
                self.cursor.y -= 1;
            }
        } else {
            self.content[self.cursor.y] = String::new();
        }
    }

    fn get_command(&mut self) -> String {
        let command = self.commandline.clone();

        self.commandline = String::new();
        self.cursor.x = self.cursor.desired_x;

        command
    }
}
