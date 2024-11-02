use log::info;

use std::cell::RefCell;
use std::rc::Rc;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

pub trait Manipulation {
    fn cursor_left(&mut self);
    fn cursor_right(&mut self);
    fn cursor_up(&mut self);
    fn cursor_down(&mut self);
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

pub struct Buffer {
    pub content: Vec<String>,
    source: ContentSource,
    pub cursor: (usize, usize),
    pub mode: Mode,
}

impl Buffer {
    pub fn scratch() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Buffer {
            content: vec![
                "This is the scratch buffer".to_string(),
                "This buffer isn't connected to a file, so nothing in here is saved.".to_string(),
                " ".to_string(),
            ],
            source: ContentSource::None,
            cursor: (0, 0),
            mode: Mode::Normal,
        }))
    }

    pub async fn from_file(path: &str) -> anyhow::Result<Rc<RefCell<Self>>> {
        let file = File::open(path)?;
        let mut buf_reader = BufReader::new(&file);
        let mut content = String::new();

        buf_reader.read_to_string(&mut content)?;

        Ok(Rc::new(RefCell::new(Buffer {
            content: content.split("\n").map(|line| line.to_string()).collect(),
            source: ContentSource::File(file),
            cursor: (0, 0),
            mode: Mode::Normal,
        })))
    }
}

impl Manipulation for Buffer {
    fn cursor_left(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
        }
    }

    fn cursor_right(&mut self) {
        if self.cursor.0 < self.content[self.cursor.1].len() - 1 {
            self.cursor.0 += 1;
        }
    }

    fn cursor_up(&mut self) {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        }
    }

    fn cursor_down(&mut self) {
        if self.cursor.1 < self.content.len() - 1 {
            self.cursor.1 += 1;
        }
    }
}
