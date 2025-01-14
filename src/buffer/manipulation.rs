use crate::buffer::{Buffer, CommandLine, Error, Minibuffer, MinibufferKind, Mode};
use crate::keybinding::actions::{ModeParams, NewLineDirection};

type Result<T> = std::result::Result<T, Error>;

pub trait Manipulation {
    fn add_char(&mut self, character: char) -> Result<()>;
    fn add_tab(&mut self) -> Result<()>;
    fn new_line(&mut self, direction: NewLineDirection);
    fn remove_char(&mut self) -> Result<()>;
    fn delete_line(&mut self);
}

// TODO: Implement Manipulation for Command Line.
impl Manipulation for Buffer {
    // Adds a character to the buffer or the command line.
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
            _ => {
                return Err(Error::WrongModeError {
                    current_mode: self.mode.to_string(),
                    valid_modes: vec![Mode::Insert.to_string(), Mode::Command.to_string()],
                })
            }
        };

        Ok(())
    }

    fn add_tab(&mut self) -> Result<()> {
        let mut spaces = 4;

        while (self.cursor.x + spaces) % 4 != 0 {
            spaces -= 1;
        }

        log::info!("{}", spaces);

        for _ in 0..spaces {
            self.add_char(' ')?;
        }

        Ok(())
    }

    // Inserts a new line either under or above the cursor.
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

        self.viewport.adjust(self.cursor.y, self.content.len());
    }

    // Implements the remove character logic for all modes.
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

                        if !self.content[self.cursor.y].is_empty()
                            && self.cursor.x >= self.content[self.cursor.y].len() - 1
                        {
                            self.cursor.x -= 1;
                        }
                    } else if self.cursor.x == self.content[self.cursor.y].len()
                        && !self.content[self.cursor.y].is_empty()
                    {
                        self.cursor.x -= 1;
                    }
                }
            }
            // Removes the selected characters.
            Mode::Visual => {
                if let Some(start) = self.visual_start {
                    if self.state.mutable {
                        // Determine the top and bottom positions.
                        let (top, bottom) = if start.y < self.cursor.y
                            || (start.y == self.cursor.y && start.x <= self.cursor.x)
                        {
                            (start, self.cursor)
                        } else {
                            (self.cursor, start)
                        };

                        // Ensure indices are within bounds.
                        if top.y >= self.content.len() || bottom.y >= self.content.len() {
                            return Ok(()); // Early return for invalid indices.
                        }

                        // Handle multi-line and single-line selection.
                        if top.y == bottom.y {
                            // Single-line selection.
                            let line = &self.content[top.y];
                            let new_line = if bottom.x < line.len() {
                                let before = &line[..top.x];
                                let after = &line[bottom.x..];
                                format!("{}{}", before, after)
                            } else {
                                line[..top.x].to_string()
                            };
                            self.content[top.y] = new_line;
                        } else {
                            // Multi-line selection.

                            // Check if the bottom line is fully selected.
                            if bottom.x == 0 || bottom.x >= self.content[bottom.y].len() {
                                self.content.remove(bottom.y);
                            } else {
                                // Modify the bottom line after the selection end.
                                let bottom_line = &self.content[bottom.y];
                                self.content[bottom.y] = bottom_line[bottom.x..].to_string();
                            }

                            // Remove all lines inbetween.
                            for _ in (top.y + 1..bottom.y).rev() {
                                self.content.remove(top.y + 1);
                            }

                            // Check if the top line is fully selected.
                            if top.x == 0 && top.y < self.content.len() && self.content.len() > 1 {
                                self.content.remove(top.y);
                            } else {
                                // Modify the top line up to the selection start.
                                let top_line = &self.content[top.y];
                                self.content[top.y] = top_line[..top.x].to_string();
                            }
                        }

                        // Update the cursor and switch back to normal mode.
                        self.cursor.x = top.x;
                        self.cursor.y = top.y;
                        self.switch_mode(ModeParams::Normal);
                    }
                }
            }
            Mode::Command => self.command_line.remove_char()?,
            Mode::Minibuffer => {
                return Err(Error::WrongModeError {
                    current_mode: self.mode.to_string(),
                    valid_modes: vec![
                        Mode::Normal.to_string(),
                        Mode::Insert.to_string(),
                        Mode::Visual.to_string(),
                        Mode::Command.to_string(),
                    ],
                })
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

impl Manipulation for CommandLine {
    fn add_char(&mut self, character: char) -> Result<()> {
        let prefix_len = self.prefix.len();

        self.input.insert(self.cursor.x - prefix_len, character);
        self.cursor.x += 1;

        Ok(())
    }

    fn remove_char(&mut self) -> Result<()> {
        let prefix_len = self.prefix.len();

        if !self.input.is_empty() {
            self.input.remove(self.cursor.x - prefix_len - 1);
            self.cursor.x -= 1;
        }

        Ok(())
    }

    fn add_tab(&mut self) -> Result<()> {
        unreachable!()
    }

    fn new_line(&mut self, _direction: NewLineDirection) {
        unreachable!()
    }

    fn delete_line(&mut self) {
        unreachable!()
    }
}

impl Manipulation for Minibuffer {
    fn add_char(&mut self, character: char) -> Result<()> {
        let matched_len = self.matched_input.len();

        self.input.insert(self.cursor.x - matched_len, character);
        self.cursor.x += 1;

        Ok(())
    }

    fn remove_char(&mut self) -> Result<()> {
        let matched_len = self.matched_input.len();

        if self.input.is_empty() {
            if self.matched_input.pop().is_some() {
                match &mut self.kind {
                    MinibufferKind::File(path) => {
                        path.pop();
                        self.cursor.y = 0;
                    }
                    _ => {}
                }
            }
        } else {
            self.input.remove(self.cursor.x - matched_len - 1);
        }

        if self.cursor.x > 0 {
            self.cursor.x -= 1;
        }

        Ok(())
    }

    fn add_tab(&mut self) -> Result<()> {
        unreachable!()
    }

    fn new_line(&mut self, _direction: NewLineDirection) {
        unreachable!()
    }

    fn delete_line(&mut self) {
        unreachable!()
    }
}
