use std::cell::RefCell;
use std::rc::Rc;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

enum ContentSource {
    None,
    File(File),
}

pub struct Buffer {
    pub content: Vec<String>,
    source: ContentSource,
    pub cursor: (usize, usize),
}

impl Buffer {
    pub fn scratch() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Buffer {
            content: vec![
                "This is the scratch buffer".to_string(),
                "This buffer isn't connected to a file, so nothing in here is saved.".to_string(),
            ],
            source: ContentSource::None,
            cursor: (0, 0),
        }))
    }

    pub fn from_file(path: &str) -> anyhow::Result<Rc<RefCell<Self>>> {
        let file = File::open(path)?;
        let mut buf_reader = BufReader::new(&file);
        let mut content = String::new();

        buf_reader.read_to_string(&mut content)?;

        Ok(Rc::new(RefCell::new(Buffer {
            content: content.split("\n").map(|line| line.to_string()).collect(),
            source: ContentSource::File(file),
            cursor: (0, 0),
        })))
    }
}
