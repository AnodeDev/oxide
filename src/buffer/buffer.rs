use std::cell::RefCell;
use std::rc::Rc;
use std::fs::File;
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
}

enum ContentSource {
    None,
    File(File),
}

#[derive(Eq, Hash, PartialEq)]
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
    pub fn scratch() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Buffer {
            title: "*Scratch*",
            content: vec![
                "This is the scratch buffer".to_string(),
                "This buffer isn't connected to a file, so nothing in here is saved.".to_string(),
                "It's meant to be used to play around, sketch, and try new plugins.".to_string(),
                " ".to_string(),
            ],
            source: ContentSource::None,
            cursor: Cursor::new(),
            mode: Mode::Normal,
        }))
    }

    pub async fn from_file(path_str: &'static str) -> anyhow::Result<Rc<RefCell<Self>>> {
        let path = Path::new(path_str);
        let file = File::open(path)?;
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
        })))
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
        if self.content[self.cursor.y].len() != 0 && self.cursor.x < self.content[self.cursor.y].len() - 1 {
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
}
