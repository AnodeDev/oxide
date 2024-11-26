use std::fmt;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::buffer::{CommandLineManager, CommandLineState, Error, Viewport};
use crate::keybinding::{Action, InsertDirection, ModeParams, NewLineDirection};

type Result<T> = std::result::Result<T, Error>;

/// Handles buffer manipulation.
pub trait Navigation {
    fn move_cursor(&mut self, x: i32, y: i32);
    fn move_cursor_to_top(&mut self);
    fn move_cursor_to_bot(&mut self);
}

pub trait Manipulation {
    fn add_char(&mut self, character: char) -> Result<()>;
    fn add_tab(&mut self) -> Result<()>;
    fn new_line(&mut self, direction: NewLineDirection);
    fn remove_char(&mut self) -> Result<()>;
    fn delete_line(&mut self);
}

/// The different kinds of buffers.
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
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

/// Stores the cursor position.
#[derive(Debug, Clone, Copy, Default)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
    pub desired_x: usize, // If line is shorter than x, the original x is stored here.
}

/// The state of the buffer.
pub struct BufferState {
    pub killable: bool, // If the buffer can be killed by the user or not.
    pub mutable: bool,  // If the buffer can be mutated by the user or not.
}

/// Implements some preset buffer states for code cleanliness.
impl BufferState {
    pub fn new(killable: bool, mutable: bool) -> Self {
        BufferState { killable, mutable }
    }

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

/// Buffer holds the content from a specific source.
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

/// Implements some preset buffers for code cleanliness.
/// The functions returns RefCells to keep from having to clone the buffers when modifying them.
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

    /// The scratch buffer is similar to the one in Emacs, it's an unbound buffer where the user.
    /// can write stuff and it won't be saved to a file.
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

    /// Saves to a file.
    /// Is async to not freeze the editor if it's a large file or something happens.
    pub async fn write_buffer(&mut self) -> Result<()> {
        if !self.state.mutable {
            return Ok(());
        }

        if let Some(path) = &self.path {
            let content_str = self.content.join("\n");
            let content_b = content_str.as_bytes();
            let mut file = File::create(&path)?;

            file.write_all(content_b)?;
        }

        Ok(())
    }

    /// Switches the current mode.
    /// Resets the appropriate values and applies the new parameters.
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

    /// Loads a file from a path.
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

    /// Starts the FindFile command.
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

    /// Adds the currently selected entry to the command line.
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

impl Navigation for Buffer {
    fn move_cursor(&mut self, x: i32, y: i32) {
        match self.mode {
            Mode::Normal | Mode::Visual => {
                // Sets the new y value.
                // Clamp is used to make sure it doesn't exceed the length of the line or 0.
                let new_y =
                    (self.cursor.y as i32 + y).clamp(0, self.content.len() as i32 - 1) as usize;
                self.cursor.y = new_y;

                // Adjusts the viewport to match the cursor position.
                self.viewport.adjust(self.cursor.y, self.content.len());

                // Checks if cursor is moved horiozontally.
                // If not, it checks if x is larger than the current lines length and adjusts accordingly.
                if x != 0 {
                    let current_line_len = self.content[self.cursor.y].len();
                    let new_x =
                        (self.cursor.x as i32 + x).clamp(0, current_line_len as i32) as usize;

                    self.cursor.x = new_x;
                    self.cursor.desired_x = new_x;
                } else {
                    let current_line_len = self.content[self.cursor.y].len();
                    self.cursor.x = self.cursor.desired_x.min(current_line_len);
                }

                // Checks if visual mode is on and makes sure to adjust the visual cursor accordingly.
                if let Some(visual_end) = &mut self.visual_end {
                    visual_end.x = self.cursor.x;
                    visual_end.y = self.cursor.y;
                    visual_end.desired_x = self.cursor.desired_x;
                }
            }
            Mode::Command => {
                self.command_line.move_cursor(x, y);
            }
            _ => {}
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
}

impl Manipulation for Buffer {
    /// Adds a character to the buffer or the command line.
    fn add_char(&mut self, character: char) -> Result<()> {
        // Minimizes repetetive code by editing the current line from either source.
        match self.mode {
            Mode::Insert => {
                self.content[self.cursor.y].insert(self.cursor.x, character);
                self.cursor.x += 1;
            }
            Mode::Command => {
                self.command_line.add_char(character)?;
            }
            // If user is in normal- or visual mode, something is wrong.
            Mode::Normal | Mode::Visual => return Err(Error::WrongModeError),
        };

        Ok(())
    }

    fn add_tab(&mut self) -> Result<()> {
        let mut spaces = 4;

        while (self.cursor.x + spaces) % 4 != 0 {
            spaces -= 1;
        }

        for _ in 0..spaces {
            self.add_char(' ')?;
        }

        Ok(())
    }

    /// Inserts a new line either under or above the cursor.
    fn new_line(&mut self, direction: NewLineDirection) {
        match self.mode {
            Mode::Insert => {
                let remaining_text = self.content[self.cursor.y].split_off(self.cursor.x);
                self.content.insert(self.cursor.y + 1, remaining_text);
                self.cursor.y += 1;
                self.cursor.x = 0;
            }
            Mode::Normal => {
                if self.state.mutable {
                    match direction {
                        NewLineDirection::Under => {
                            self.content.insert(self.cursor.y + 1, String::new());
                            self.cursor.y += 1;
                            self.cursor.x = 0;
                        }
                        NewLineDirection::Over => {
                            self.content.insert(self.cursor.y, String::new());
                            self.cursor.x = 0;
                        }
                    }

                    self.mode = Mode::Insert;
                }
            }
            _ => {}
        }
    }

    /// Implements the remove character logic for all modes.
    fn remove_char(&mut self) -> Result<()> {
        match self.mode {
            Mode::Insert => {
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
            // Removes the character under the cursor, like 'x' in Neovim.
            Mode::Normal => {
                if self.state.mutable {
                    if self.cursor.x < self.content[self.cursor.y].len() {
                        self.content[self.cursor.y].remove(self.cursor.x);
                    }
                }
            }
            Mode::Command => {
                self.command_line.remove_char()?;
            }
            // Removes the selected characters.
            Mode::Visual => {
                if self.state.mutable {
                    if let (Some(start), Some(end)) = (&mut self.visual_start, &mut self.visual_end)
                    {
                        // Sets the top and bottom cursor positions.
                        let (top, bottom) = if start.y < end.y {
                            (start, end)
                        } else if start.y == end.y && start.x < end.x {
                            (start, end)
                        } else if start.y == end.y && start.x > end.x {
                            (end, start)
                        } else {
                            (end, start)
                        };

                        let mut selected_lines: Vec<String> = self.content[top.y..bottom.y + 1]
                            .iter()
                            .map(|line| line.to_string())
                            .collect();

                        // Checks is selection is on one line or multiple lines.
                        let new_top_line = if top.y < bottom.y {
                            selected_lines[0][..top.x].to_string()
                        } else {
                            let mut beginning = selected_lines[0][..top.x].to_string();
                            let end = selected_lines[0][bottom.x + 1..].to_string();
                            beginning.push_str(&end);

                            beginning
                        };

                        // Checks if the whole line is selected.
                        if top.x == 0
                            && (bottom.x == selected_lines[0].len() || selected_lines.len() > 1)
                        {
                            self.content[top.y] = "".to_string();
                        } else {
                            self.content[top.y] = new_top_line;
                        }

                        // Removes first and last line from selected_lines.
                        selected_lines.remove(0);
                        let last_line = selected_lines.pop();

                        // Removes all selected lines between first and last.
                        for (num, _line) in selected_lines.iter().enumerate() {
                            self.content.remove(top.y + num + 1);
                        }

                        // Makes sure bottom.y is set correctly.
                        bottom.y = top.y + 1;

                        // Checks if last line even exists.
                        match last_line {
                            Some(line) => {
                                if line.len() > 0 {
                                    if bottom.x == line.len() {
                                        bottom.x -= 1;
                                    }

                                    self.content[bottom.y] = line[bottom.x + 1..].to_string();

                                    let current_line = self.content.remove(bottom.y);

                                    self.cursor.x = top.y + self.content[top.y].len();
                                    self.content[top.y].push_str(&current_line);
                                } else {
                                    self.content.remove(bottom.y);
                                }
                            }
                            None => {}
                        }

                        // Updates the cursor position and switches back to normal mode.
                        self.cursor.x = top.x;
                        self.cursor.y = top.y;
                        self.switch_mode(ModeParams::Normal { mode: Mode::Normal });
                    } else {
                        return Err(Error::VisualModeInitError);
                    }
                }
            }
        }

        Ok(())
    }

    // Deletes the current line.
    fn delete_line(&mut self) {
        if self.state.mutable {
            if self.content.len() > 1 {
                self.content.remove(self.cursor.y);

                if self.cursor.y > self.content.len() - 1 {
                    self.cursor.y -= 1;
                }
            } else {
                self.content[self.cursor.y] = String::new();
            }

            self.cursor.x = 0;
        }
    }
}
