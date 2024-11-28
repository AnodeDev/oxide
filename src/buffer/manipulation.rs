use crate::buffer::Error;
use crate::buffer::{Buffer, Mode};
use crate::keybinding::{ModeParams, NewLineDirection};

type Result<T> = std::result::Result<T, Error>;

pub trait Manipulation {
    fn add_char(&mut self, character: char) -> Result<()>;
    fn add_tab(&mut self) -> Result<()>;
    fn new_line(&mut self, direction: NewLineDirection);
    fn remove_char(&mut self) -> Result<()>;
    fn delete_line(&mut self);
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
