use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::fmt;
use std::rc::Rc;

use crate::keybinding::{NewLineDirection, ModeParams, InsertDirection};
use crate::buffer::{Viewport, Error, ErrorKind};

type Result<'a, T> = std::result::Result<T, Error<'a>>;

/// Handles buffer manipulation.
pub trait Manipulation {
    fn move_cursor(&mut self, x: i32, y: i32);
    fn move_cursor_to_top(&mut self);
    fn move_cursor_to_bot(&mut self);
    fn add_char(&mut self, character: char) -> Result<()>;
    fn new_line(&mut self, direction: NewLineDirection);
    fn remove_char(&mut self);
    fn delete_line(&mut self);
}

pub enum ContentSource {
    NoSource,
    File(PathBuf),
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
            Mode::Normal  => write!(f, "NORMAL"),
            Mode::Insert  => write!(f, "INSERT"),
            Mode::Visual  => write!(f, "VISUAL"),
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
    pub desired_y: usize, // If cursor is moved, ex to command line, y is stored here.
}

/// The state of the buffer.
pub struct BufferState {
    pub killable: bool, // If the buffer can be killed by the user or not.
    pub mutable: bool,  // If the buffer can be mutated by the user or not.
}

/// Implements some preset buffer states for code cleanliness.
impl BufferState {
    fn new(killable: bool, mutable: bool) -> Self {
        BufferState { killable, mutable }
    }

    fn scratch() -> Self {
        BufferState {
            killable: false,
            mutable: true,
        }
    }

    fn locked() -> Self {
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

#[derive(Default)]
pub struct CommandLineManager {
    pub content: Vec<String>,
    pub prefix: String,
    pub cursor: Cursor,
    pub input: String,
}

impl Manipulation for CommandLineManager {
    fn move_cursor(&mut self, x: i32, y: i32) {
        // Sets the new y value.
        // Clamp is used to make sure it doesn't exceed the length of the line or 0.
        let new_y = (self.cursor.y as i32 + y).clamp(0, self.content.len() as i32) as usize;
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
    fn move_cursor_to_top(&mut self) {}
    fn move_cursor_to_bot(&mut self) {}
    fn add_char(&mut self, character: char) -> Result<()> {
        self.input.insert(self.cursor.x - self.prefix.len(), character);
        self.cursor.x += 1;

        Ok(())
    }
    fn new_line(&mut self, _: NewLineDirection) {}
    fn remove_char(&mut self) {
        if self.cursor.x >= 2 && self.cursor.x - 2 < self.input.len() {
            self.input.remove(self.cursor.x - 2);

            self.cursor.x -= 1;
        }
    }
    fn delete_line(&mut self) {}
}

/// Buffer holds the content from a specific source.
pub struct Buffer {
    pub title: &'static str,
    pub content: Vec<String>,
    source: ContentSource,
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
    pub fn new(title: &'static str, content: Vec<String>, source: ContentSource, height: usize, killable: bool, mutable: bool) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Buffer {
            title,
            content,
            source,
            cursor: Cursor::default(),
            viewport: Viewport::new(height - 2),
            mode: Mode::Normal,
            state: BufferState::new(killable, mutable),
            command_line: CommandLineManager::default(),
            visual_start: None,
            visual_end: None,
        }))
    }

    /// The scratch buffer is similar to the one in Emacs, it's an unbound buffer where the user.
    /// can write stuff and it won't be saved to a file.
    pub fn scratch(height: usize) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Buffer {
            title: "*Scratch*",
            content: vec![
                "This is the scratch buffer".to_string(),
                "This buffer isn't connected to a file, so nothing in here is saved.".to_string(),
                "It's meant to be used to play around, sketch, and try new plugins.".to_string(),
                String::new(),
            ],
            source: ContentSource::NoSource,
            cursor: Cursor::default(),
            viewport: Viewport::new(height - 2),
            mode: Mode::Normal,
            state: BufferState::scratch(),
            command_line: CommandLineManager::default(),
            visual_start: None,
            visual_end: None,
        }))
    }

    pub async fn from_file(path_str: &'static str, height: usize) -> Result<'static, Rc<RefCell<Self>>> {
        let mut path = PathBuf::new();
        let mut content = String::new();
        path.push(path_str);
        match File::open(path.clone()) {
            Ok(file) => {
                let mut buf_reader = BufReader::new(&file);
                match buf_reader.read_to_string(&mut content) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Err(_) => {
                return Err(Error::new(ErrorKind::FileNotFoundError, "Was not able to open/create file"));
            },
        };
        let mut file_name = "[NO NAME]";

        if let Some(name_osstr) = Path::new(path_str).file_name() {
            if let Some(name) = name_osstr.to_str() {
                file_name = name;
            }
        }

        Ok(Rc::new(RefCell::new(Buffer {
            title: file_name,
            content: content.split("\n").map(|line| line.to_string()).collect(),
            source: ContentSource::File(path),
            cursor: Cursor::default(),
            viewport: Viewport::new(height - 2),
            mode: Mode::Normal,
            state: BufferState::default(),
            command_line: CommandLineManager::default(),
            visual_start: None,
            visual_end: None,
        })))
    }

    /// Saves to a file.
    /// Is async to not freeze the editor if it's a large file or something happens.
    pub async fn write_buffer(&mut self) -> Result<()> {
        if !self.state.mutable {
            return Ok(())
        }

        match &mut self.source {
            ContentSource::File(path) => {
                let content_str = self.content.join("\n");
                let content_b = content_str.as_bytes();
                let mut file = File::create(&path);

                match &mut file {
                    Ok(file) => match file.write_all(content_b) {
                        Ok(_) => {},
                        Err(_) => {},
                    }
                    Err(_) => {
                        return Err(Error::new(ErrorKind::WriteToSourceError, "Was not able to write to source"));
                    },
                }

            },
            _ => {},
        }

        Ok(())
    }

    pub fn switch_mode(&mut self, mode: ModeParams) {
        // Makes sure to reset the visual cursors.
        match self.mode {
            Mode::Visual => {
                self.visual_start = None;
                self.visual_end = None;
            },
            Mode::Command => {
                self.command_line.input = String::new();
                self.command_line.content = Vec::new();
                self.command_line.cursor = Cursor::default();
            },
            _ => {},
        }

        match mode {
            ModeParams::Visual{mode} => {
                self.visual_start = Some(self.cursor);
                self.visual_end = Some(self.cursor);

                self.mode = mode;
            },
            ModeParams::Command{mode, prefix} => {
                self.command_line.prefix = prefix;
                self.command_line.cursor.x = self.command_line.prefix.len();
                self.command_line.cursor.y = 0;

                self.mode = mode;
            },
            ModeParams::Insert{mode, insert_direction} => {
                match insert_direction {
                    InsertDirection::Before => {},
                    InsertDirection::After => self.cursor.x += 1,
                }

                self.mode = mode;
            }
            ModeParams::Normal{mode} => self.mode = mode,
        }
    }

    // Returns the current command from the command line.
    pub fn get_command(&mut self) -> String {
        self.command_line.input.clone()
    }

    pub fn find_file(&mut self) {
    }
}

impl Manipulation for Buffer {
    fn move_cursor(&mut self, x: i32, y: i32) {
        match self.mode {
            Mode::Normal | Mode::Visual => {
                // Sets the new y value.
                // Clamp is used to make sure it doesn't exceed the length of the line or 0.
                let new_y = (self.cursor.y as i32 + y).clamp(0, self.content.len() as i32 - 1) as usize;
                self.cursor.y = new_y;

                // Adjusts the viewport to match the cursor position.
                self.viewport.adjust(self.cursor.y, self.content.len());

                // Checks if cursor is moved horiozontally.
                // If not, it checks if x is larger than the current lines length and adjusts accordingly.
                if x != 0 {
                    let current_line_len = self.content[self.cursor.y].len();
                    let new_x = (self.cursor.x as i32 + x).clamp(0, current_line_len as i32) as usize;

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
                    visual_end.desired_y = self.cursor.desired_y;
                }
            },
            Mode::Command => {
                self.command_line.move_cursor(x, y);
            },
            _ => {},
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

    /// Adds a character to the buffer or the command line.
    fn add_char(&mut self, character: char) -> Result<()> {
        // Minimizes repetetive code by editing the current line from either source.
        match self.mode {
            Mode::Insert => {
                self.content[self.cursor.y].insert(self.cursor.x, character);
                self.cursor.x += 1;
            },
            Mode::Command => {
                match self.command_line.add_char(character) {
                    Ok(_) => {},
                    Err(e) => return Err(e),
                };
            },
            // If user is in normal- or visual mode, something is wrong.
            Mode::Normal | Mode::Visual => return Err(Error::new(ErrorKind::WrongModeError, "Expected Insert or Command, found {}")),
        };

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

    /// Implements the remove character logic for all modes.
    fn remove_char(&mut self) {
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
            },
            // Removes the character under the cursor, like 'x' in Neovim.
            Mode::Normal => {
                if self.cursor.x < self.content[self.cursor.y].len() {
                    self.content[self.cursor.y].remove(self.cursor.x);
                }
            },
            Mode::Command => {
                self.command_line.remove_char();
            },
            // Removes the selected characters.
            Mode::Visual => {
                if let (Some(start), Some(end)) = (&mut self.visual_start, &mut self.visual_end) {
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

                    let mut selected_lines: Vec<String> = self.content[top.y..bottom.y + 1].iter().map(|line| line.to_string()).collect();

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
                    if top.x == 0 && (bottom.x == selected_lines[0].len() || selected_lines.len() > 1) {
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
                        },
                        None => {},
                    }

                    // Updates the cursor position and switches back to normal mode.
                    self.cursor.x = top.x;
                    self.cursor.y = top.y;
                    self.switch_mode(ModeParams::Normal { mode: Mode::Normal });
                } else {
                    todo!("Throw error, visual mode wasn't properly initialized")
                }
            }
        };
    }

    // Deletes the current line.
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
}
