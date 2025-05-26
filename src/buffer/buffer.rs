use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

use crate::buffer::{Error, Viewport};
use crate::keybinding::actions::{InsertDirection, ModeParams};

// ╭──────────────────────────────────────╮
// │ Buffer Types                         │
// ╰──────────────────────────────────────╯

type Result<T> = std::result::Result<T, Error>;

// ╭──────────────────────────────────────╮
// │ Buffer Enums                         │
// ╰──────────────────────────────────────╯

// Enables different interactions and rendering options without having to hard-code buffer styles.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum BufferKind {
    Normal,
    BufferList,
}

// All available modal modes.
#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
    Minibuffer,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::Normal => write!(f, "NORMAL"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::Visual => write!(f, "VISUAL"),
            Mode::Command => write!(f, "COMMAND"),
            _ => write!(f, ""),
        }
    }
}

// ╭──────────────────────────────────────╮
// │ Buffer Structs                       │
// ╰──────────────────────────────────────╯

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
    pub desired_x: usize, // If line is shorter than x, the original x is stored here.
}

// Holds the states of the buffer. These states tell the editor if the buffer can be edited and/or
// closed.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct BufferState {
    pub killable: bool,
    pub mutable: bool,
}

impl BufferState {
    pub fn new(killable: bool, mutable: bool) -> Self {
        BufferState { killable, mutable }
    }

    // Buffer state presets for some commonly used buffers.
    pub fn scratch() -> Self {
        BufferState {
            killable: false,
            mutable: true,
        }
    }

    // A preset for buffers that are permanent and can't be modified in insert mode.
    pub fn locked() -> Self {
        BufferState {
            killable: false,
            mutable: false,
        }
    }
}

impl std::default::Default for BufferState {
    fn default() -> Self {
        BufferState {
            killable: true,
            mutable: true,
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct CommandLine {
    pub input: String,
    pub prefix: String,
    pub cursor: Cursor, // Has it's own cursor to avoid overflow conflicts.
}

// The main buffer struct. Holds all the information related to the buffer.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Buffer {
    pub title: String,
    pub content: Vec<String>,
    pub path: Option<PathBuf>,
    pub kind: BufferKind,
    pub cursor: Cursor,
    pub viewport: Viewport,
    pub mode: Mode,
    pub state: BufferState,
    pub command_line: CommandLine,
    pub visual_start: Option<Cursor>,
}

impl Buffer {
    pub fn new(
        title: String,
        content: Vec<String>,
        path: Option<PathBuf>,
        kind: BufferKind,
        height: usize,
        state: BufferState,
    ) -> Self {
        let content = if content.is_empty() {
            vec![String::new()]
        } else {
            content
        };

        Buffer {
            title,
            content,
            path,
            kind,
            cursor: Cursor::default(),
            viewport: Viewport::new(height - 2),
            mode: Mode::Normal,
            state,
            command_line: CommandLine::default(),
            visual_start: None,
        }
    }

    // The scratch buffer is similar to the one in Emacs. It's a free buffer with no file to save
    // to, meant to test configuration options (when that's available).
    pub fn scratch(height: usize) -> Self {
        Buffer {
            title: "*Scratch*".to_string(),
            content: vec![
                "This is the scratch buffer".to_string(),
                "This buffer isn't connected to a file, so nothing in here is saved.".to_string(),
                "It's meant to be used to play around, sketch, and try new plugins.".to_string(),
                String::new(),
            ],
            path: None,
            kind: BufferKind::Normal,
            cursor: Cursor::default(),
            viewport: Viewport::new(height - 2),
            mode: Mode::Normal,
            state: BufferState::scratch(),
            command_line: CommandLine::default(),
            visual_start: None,
        }
    }

    // The buffer list is similar to the one in Emacs. It's a list of the open buffers and when one
    // is pressed the editor switches to that buffer.
    pub fn buffer_list(height: usize) -> Self {
        Buffer {
            title: "*Buffers*".to_string(),
            content: vec![String::new()],
            path: None,
            kind: BufferKind::BufferList,
            cursor: Cursor::default(),
            viewport: Viewport::new(height - 2),
            mode: Mode::Normal,
            state: BufferState::locked(),
            command_line: CommandLine::default(),
            visual_start: None,
        }
    }

    // Loads a file into the buffer.
    pub async fn from_file(path: PathBuf, height: usize) -> Result<Self> {
        let mut file_name = "[NO NAME]".to_string(); // Placeholder if it can't get the file name.
        let mut content: Vec<String> = Vec::new();

        if path.exists() {
            let file = File::open(path.clone())?;
            let mut buf_reader = BufReader::new(file);
            let mut file_content = String::new();

            buf_reader.read_to_string(&mut file_content)?;

            if let Some(name_osstr) = path.file_name() {
                file_name = name_osstr.to_string_lossy().into_owned();
            }
            content = file_content.split("\n").map(|line| line.to_string()).collect();
        } else {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    return Err(Error::InvalidPathError { path: parent.to_path_buf() });
                }
            }
        }

        Ok(Buffer {
            title: file_name,
            content,
            path: Some(path),
            kind: BufferKind::Normal,
            cursor: Cursor::default(),
            viewport: Viewport::new(height - 2),
            mode: Mode::Normal,
            state: BufferState::default(),
            command_line: CommandLine::default(),
            visual_start: None,
        })
    }

    // Writes the buffer content to it's source file, if there is one. It's async as to not disable
    // the editor in case something happens.
    pub async fn write_buffer(&mut self) -> Result<()> {
        if !self.state.mutable {
            return Err(Error::ImmutableBufferError {
                title: self.title.clone(),
            });
        }

        if let Some(path) = &self.path {
            let content_str = self.content.join("\n");
            let content_b = content_str.as_bytes();
            let mut file = File::create(&path)?;

            file.write_all(content_b)?;
        }

        Ok(())
    }

    pub fn switch_mode(&mut self, mode: ModeParams) {
        // Makes sure to reset the visual cursors and command line values
        match self.mode {
            Mode::Visual => self.visual_start = None,
            Mode::Command => {
                self.command_line.prefix = String::new();
                self.command_line.input = String::new();
                self.command_line.cursor = Cursor::default();
            }
            _ => {}
        }

        match mode {
            ModeParams::Visual => self.switch_mode_visual(),
            ModeParams::Command { prefix } => self.switch_mode_command(prefix),
            ModeParams::Insert { insert_direction } => self.switch_mode_insert(insert_direction),
            ModeParams::Normal => self.switch_mode_normal(),
            ModeParams::Minibuffer => self.switch_mode_minibuffer(),
        }
    }

    fn switch_mode_visual(&mut self) {
        self.visual_start = Some(self.cursor);
        self.mode = Mode::Visual;
    }

    fn switch_mode_command(&mut self, prefix: String) {
        self.command_line.prefix = prefix;
        self.command_line.input = String::new();
        self.command_line.cursor.x = self.command_line.prefix.len();

        self.mode = Mode::Command;
    }

    fn switch_mode_normal(&mut self) {
        self.mode = Mode::Normal;
    }

    fn switch_mode_insert(&mut self, direction: InsertDirection) {
        match direction {
            InsertDirection::Beginning => {
                if let Some(line) = self.content.get(self.cursor.y) {
                    let first_char = line
                        .char_indices()
                        .find(|(_, c)| !c.is_whitespace())
                        .map(|(index, _)| index);

                    if let Some(index) = first_char {
                        self.cursor.x = index;
                    }
                }
            }
            InsertDirection::After => {
                if let Some(line) = self.content.get(self.cursor.y) {
                    if line.len() > self.cursor.x {
                        self.cursor.x += 1;
                    }
                }
            }
            InsertDirection::End => {
                if let Some(line) = self.content.get(self.cursor.y) {
                    self.cursor.x = line.len();
                }
            }
            InsertDirection::Before => {}
        }

        self.mode = Mode::Insert;
    }

    fn switch_mode_minibuffer(&mut self) {
        self.mode = Mode::Minibuffer;
    }

    // Returns the current command from the command line.
    pub fn get_command(&mut self) -> &str {
        &self.command_line.input
    }
}
