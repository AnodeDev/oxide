use std::cell::RefCell;
use std::rc::Rc;
use std::fs::{OpenOptions, File};
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;
use std::fmt;

pub trait Manipulation {
    fn cursor_left(&mut self);
    fn cursor_right(&mut self);
    fn cursor_up(&mut self);
    fn cursor_down(&mut self);
    fn cursor_half_down(&mut self);
    fn cursor_half_up(&mut self);
    fn remove_char(&mut self);
    fn add_char(&mut self, character: char);
    fn new_line(&mut self);
    fn remove_char_commandline(&mut self);
    fn add_char_commandline(&mut self, character: char);
    fn give_command(&mut self) -> String;
}

pub enum ContentSource {
    None,
    File(File),
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
}

pub struct Buffer {
    pub title: &'static str,
    pub content: Vec<String>,
    source: ContentSource,
    pub cursor: Cursor,
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
    pub fn new(title: &'static str, content: Vec<String>, source: ContentSource, killable: bool, mutable: bool) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Buffer {
            title,
            content,
            source,
            cursor: Cursor::new(),
            mode: Mode::Normal,
            killable,
            mutable,
            commandline: String::new(),
        }))
    }

    pub fn scratch() -> Rc<RefCell<Self>> {
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
            mode: Mode::Normal,
            killable: false,
            mutable: false,
            commandline: String::new(),
        }))
    }

    pub async fn from_file(path_str: &'static str) -> anyhow::Result<Rc<RefCell<Self>>> {
        let path = Path::new(path_str);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        let mut buf_reader = BufReader::new(&file);
        let mut content = String::new();
        let mut file_name = "[NO NAME]";

        if let Some(name_osstr) = path.file_name() {
            if let Some(name) = name_osstr.to_str() {
                file_name = name;
            }
        }
        buf_reader.read_to_string(&mut content)?;

        Ok(Rc::new(RefCell::new(Buffer {
            title: file_name,
            content: content.split("\n").map(|line| line.to_string()).collect(),
            source: ContentSource::File(file),
            cursor: Cursor::new(),
            mode: Mode::Normal,
            killable: true,
            mutable: true,
            commandline: String::new(),
        })))
    }

    pub async fn save_buffer(&mut self) -> anyhow::Result<()> {
        if !self.mutable {
            return Ok(())
        }

        match &mut self.source {
            ContentSource::File(file) => {
                let content_str = self.content.join("\n");
                let content_b = content_str.as_bytes();

                file.write_all(content_b)?;
            },
            _ => {},
        }

        Ok(())
    }
}

impl Manipulation for Buffer {
    fn cursor_left(&mut self) {
        if self.cursor.x > 0 {
            self.cursor.x -= 1;
            self.cursor.desired_x = self.cursor.x;
        }
    }

    fn cursor_right(&mut self) {
        // if self.content[self.cursor.y].len() != 0 && self.cursor.x < self.content[self.cursor.y].len() {
        if self.cursor.x < self.content[self.cursor.y].len() {
            self.cursor.x += 1;
            self.cursor.desired_x = self.cursor.x;
        }
    }

    fn cursor_up(&mut self) {
        if self.cursor.y > 0 {
            self.cursor.y -= 1;

            let line_len = self.content[self.cursor.y].len();

            self.cursor.x = self.cursor.desired_x.min(line_len.saturating_sub(1));
        }
    }

    fn cursor_down(&mut self) {
        if self.cursor.y < self.content.len() - 1 {
            self.cursor.y += 1;

            let line_len = self.content[self.cursor.y].len();

            self.cursor.x = self.cursor.desired_x.min(line_len.saturating_sub(1));
        }
    }

    fn cursor_half_down(&mut self) {
        if self.cursor.y < (self.content.len() -1) / 2 {
            self.cursor.y = (self.content.len() -1) / 2;
        } else {
            self.cursor.y = self.content.len() - 1;
        }

        let line_len = self.content[self.cursor.y].len();

        self.cursor.x = self.cursor.desired_x.min(line_len.saturating_sub(1));
    }

    fn cursor_half_up(&mut self) {
        if self.cursor.y > (self.content.len() -1) / 2 {
            self.cursor.y = (self.content.len() -1) / 2;
        } else {
            self.cursor.y = 0;
        }

        let line_len = self.content[self.cursor.y].len();

        self.cursor.x = self.cursor.desired_x.min(line_len.saturating_sub(1));
    }

    fn remove_char(&mut self) {
        if self.cursor.x > 0 {
            self.content[self.cursor.y].remove(self.cursor.x - 1);

            self.cursor.x -= 1;
        } else if self.cursor.y > 0 {
            let current_line = self.content.remove(self.cursor.y);

            self.cursor.y -= 1;
            self.cursor.x = self.content[self.cursor.y].len();
            self.content[self.cursor.y].push_str(&current_line);
        }
    }

    fn add_char(&mut self, character: char) {
        self.content[self.cursor.y].insert(self.cursor.x, character);
        self.cursor.x += 1;
    }

    fn new_line(&mut self) {
        let remaining_text = self.content[self.cursor.y].split_off(self.cursor.x);
        self.content.insert(self.cursor.y + 1, remaining_text);

        self.cursor.y += 1;
        self.cursor.x = 0;
    }

    fn remove_char_commandline(&mut self) {
        if self.cursor.x > 0 {
            self.commandline.remove(self.cursor.x - 1);

            self.cursor.x -= 1;
        }
    }

    fn add_char_commandline(&mut self, character: char) {
        self.commandline.insert(self.cursor.x, character);
        self.cursor.x += 1;
    }

    fn give_command(&mut self) -> String {
        let command = self.commandline.clone();
        self.commandline = String::new();

        command
    }
}