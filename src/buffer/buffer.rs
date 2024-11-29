use std::fmt;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::buffer::{CommandLineManager, CommandLineState, Error, Viewport};
use crate::keybinding::{Action, InsertDirection, ModeParams};

// ╭──────────────────────────────────────╮
// │ Buffer Types                         │
// ╰──────────────────────────────────────╯

type Result<T> = std::result::Result<T, Error>;

// ╭──────────────────────────────────────╮
// │ Buffer Enums                         │
// ╰──────────────────────────────────────╯

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum BufferKind {
    Normal,
    BufferList,
}

/// All available modal modes.
#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::Normal => write!(f, "NORMAL"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::Visual => write!(f, "VISUAL"),
            Mode::Command => write!(f, "COMMAND"),
        }
    }
}

// ╭──────────────────────────────────────╮
// │ Buffer Structs                       │
// ╰──────────────────────────────────────╯

#[derive(Debug, Clone, Copy, Default)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
    pub desired_x: usize, // If line is shorter than x, the original x is stored here.
}

// Holds the states of the buffer. These states tell the editor if the buffer can be edited and/or
// closed.
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

// The main buffer struct. Holds all the information related to the buffer
pub struct Buffer {
    pub title: String,
    pub content: Vec<String>,
    pub path: Option<PathBuf>,
    pub kind: BufferKind,
    pub cursor: Cursor,
    pub viewport: Viewport,
    pub mode: Mode,
    pub state: BufferState,
    pub command_line: CommandLineManager,
    pub visual_start: Option<Cursor>,
    pub visual_end: Option<Cursor>,
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
            command_line: CommandLineManager::default(),
            visual_start: None,
            visual_end: None,
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
            command_line: CommandLineManager::default(),
            visual_start: None,
            visual_end: None,
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
            command_line: CommandLineManager::default(),
            visual_start: None,
            visual_end: None,
        }
    }

    pub async fn from_file(path_str: &'static str, height: usize) -> Result<Self> {
        let mut path = PathBuf::new();
        let mut content = String::new();

        path.push(path_str);
        let file = File::open(path.clone())?;
        let mut buf_reader = BufReader::new(file);
        // If it can't find the name of the file, it won't display an empty string
        let mut file_name = "[NO NAME]".to_string();

        buf_reader.read_to_string(&mut content)?;

        if let Some(name_osstr) = Path::new(path_str).file_name() {
            file_name = name_osstr.to_string_lossy().into_owned();
        }
        let content: Vec<String> = content.split("\n").map(|line| line.to_string()).collect();

        Ok(Buffer {
            title: file_name,
            content,
            path: Some(path),
            kind: BufferKind::Normal,
            cursor: Cursor::default(),
            viewport: Viewport::new(height - 2),
            mode: Mode::Normal,
            state: BufferState::default(),
            command_line: CommandLineManager::default(),
            visual_start: None,
            visual_end: None,
        })
    }

    // Writes the buffer content to it's source file, if there is one. It's async as to not disable
    // the editor in case something happens.
    pub async fn write_buffer(&mut self) -> Result<()> {
        if !self.state.mutable {
            return Err(Error::FileNotFoundError);
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
            Mode::Visual => {
                self.visual_start = None;
                self.visual_end = None;
            }
            Mode::Command => {
                self.command_line.clear();
            }
            _ => {}
        }

        match mode {
            ModeParams::Visual { mode } => {
                self.visual_start = Some(self.cursor);
                self.visual_end = Some(self.cursor);

                self.mode = mode;
            }
            ModeParams::Command {
                mode,
                prefix,
                input,
                state,
            } => {
                self.command_line.prefix = prefix;
                self.command_line.input = format!("{}", input);
                self.command_line.state = state;
                self.command_line.cursor.x =
                    self.command_line.prefix.len() + self.command_line.input.len();
                self.command_line.cursor.y = 0;

                self.mode = mode;
            }
            ModeParams::Insert {
                mode,
                insert_direction,
            } => {
                if self.state.mutable {
                    match insert_direction {
                        InsertDirection::Before => {}
                        InsertDirection::After => {
                            if self.content[self.cursor.y].len() > self.cursor.x {
                                self.cursor.x += 1;
                            }
                        }
                    }

                    self.mode = mode;
                }
            }
            ModeParams::Normal { mode } => self.mode = mode,
        }
    }

    // Returns the current command from the command line.
    pub fn get_command(&mut self) -> String {
        let mut command = self.command_line.input.clone();
        command.push_str(&self.command_line.suffix);

        command
    }

    pub async fn load_file(&mut self, path: String) -> Result<()> {
        // Checks if the path points to a file.
        if Path::new(&path).is_file() {
            self.path = Some(Path::new(&path).to_path_buf());

            let mut content = String::new();

            let file = File::open(&path)?;
            let mut buf_reader = BufReader::new(&file);
            buf_reader.read_to_string(&mut content)?;

            // If the program can't fetch the name of the file, it's displayed like this.
            self.title = "[NO NAME]".to_string();

            // Tries to fetch the file name.
            if let Some(name_osstr) = Path::new(&path).file_name() {
                self.title = name_osstr.to_string_lossy().into_owned();
            }

            self.content = content.split("\n").map(|line| line.to_string()).collect();

            Ok(())
        } else {
            Err(Error::FileNotFoundError)
        }
    }

    pub async fn find_file(&mut self) -> Result<()> {
        let stored_path = &mut self.path.clone();

        // Checks if current buffer contains a path, otherwise it sets it to the home directory.
        let path: &mut PathBuf = if let Some(file_path) = stored_path {
            file_path.pop();

            file_path
        } else {
            let mut path = PathBuf::new();

            path.push("~/");

            &mut path.clone()
        };

        // Checks if the path is a directory.
        if path.is_dir() {
            // Reads the directory.
            let entries = fs::read_dir(path.clone())?;

            // Creates a PathBuf from entries.
            let content = entries
                .map(|res| res.map(|e| e.path()))
                .collect::<std::result::Result<Vec<_>, std::io::Error>>()?;

            // Filters the content to only display what matches the current input
            let mut content_filtered: Vec<&PathBuf> = content
                .iter()
                .filter(|path| {
                    path.to_string_lossy()
                        .into_owned()
                        .contains(&self.command_line.input)
                })
                .collect();

            // Sorts the content in alphabetical order.
            content_filtered.sort();

            // Separates the content into directories and files.
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

            // Keeps track of dotfiles, they go at the top of their respective category.
            let mut dot_count = 0;

            // Directories are displayed first
            for dir in &directories {
                if let Some(name) = dir.file_name() {
                    // Pushes the name of the directory to the PathBuf.
                    let name = name.to_string_lossy().into_owned();
                    path.push(&name);

                    let path_str = path.to_string_lossy().into_owned();

                    // Checks if directory is a dotfile.
                    if name.chars().nth(0) == Some('.') {
                        self.command_line.content.insert(dot_count, path_str);

                        dot_count += 1;
                    } else {
                        self.command_line.content.push(path_str);
                    }

                    // Removes the directory name in preparation for the next entry.
                    path.pop();
                }
            }

            // Sets the count to the length of the directories as to not mix files and directories.
            dot_count = directories.len();

            // Displays the files under the directories.
            for file in files {
                if let Some(name) = file.file_name() {
                    let name = name.to_string_lossy().into_owned();
                    path.push(&name);

                    let path_str = path.to_string_lossy().into_owned();

                    // Checks if file is a dotfile.
                    if name.chars().nth(0) == Some('.') {
                        self.command_line.content.insert(dot_count, path_str);

                        dot_count += 1;
                    } else {
                        self.command_line.content.push(path_str);
                    }

                    path.pop();
                }
            }

            // Switches the mode to command mode.
            self.switch_mode(ModeParams::Command {
                mode: Mode::Command,
                prefix: "Find File ".to_string(),
                input: format!("{}/", path.to_string_lossy().into_owned()),
                state: CommandLineState::FindFile,
            });
        }

        Ok(())
    }

    // Appends the currently selected entry to the command line.
    pub fn append_selected(&mut self) -> Result<()> {
        if self.mode != Mode::Command {
            return Err(Error::WrongModeError);
        } else {
            let content = &self.command_line.content[self.command_line.cursor.y];

            if Path::new(content).exists() {
                if let Some(file_name) = Path::new(content).file_name() {
                    self.command_line.suffix = file_name.to_string_lossy().into_owned();
                }
            } else {
                self.command_line.input = content.to_string();
            }

            self.command_line.cursor.x = self.command_line.prefix.len()
                + self.command_line.input.len()
                + self.command_line.suffix.len();

            Ok(())
        }
    }

    pub fn switch_buffer(&mut self, content: Vec<String>) {
        self.switch_mode(ModeParams::Command {
            mode: Mode::Command,
            prefix: "Switch Buffer ".to_string(),
            input: String::new(),
            state: CommandLineState::SwitchBuffer,
        });

        self.command_line.content = content;
    }

    pub fn select_entry(&mut self) -> Result<Option<Action>> {
        let return_type = match self.kind {
            BufferKind::BufferList => {
                Some(Action::SwitchBuffer(self.content[self.cursor.y].clone()))
            }
            BufferKind::Normal => None,
        };

        Ok(return_type)
    }
}
