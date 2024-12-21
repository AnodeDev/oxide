use ratatui::prelude::*;
use ratatui::Terminal;

use std::io::Stdout;

use crate::buffer::{Buffer, Manipulation, Minibuffer, MinibufferKind, Navigation, Mode};
use crate::keybinding::{Action, CommandParser, KeybindingManager, ModeParams};
use crate::renderer::Renderer;
use crate::OxideError;

// ╭──────────────────────────────────────╮
// │ Editor Types                         │
// ╰──────────────────────────────────────╯

type Result<T> = std::result::Result<T, crate::OxideError>;

// ╭──────────────────────────────────────╮
// │ Editor Enums                         │
// ╰──────────────────────────────────────╯

// ╭──────────────────────────────────────╮
// │ Editor Struct                        │
// ╰──────────────────────────────────────╯

pub struct Editor {
    pub buffers: Vec<Buffer>,
    pub active_buffer: usize,
    pub renderer: Renderer,
    pub is_running: bool,
    pub minibuffer: Minibuffer,
}

impl Editor {
    pub fn new(terminal: Terminal<CrosstermBackend<Stdout>>) -> Self {
        let renderer = Renderer::new(terminal);
        let height = renderer.get_terminal_size().height as usize;

        Editor {
            buffers: vec![Buffer::scratch(height)],
            active_buffer: 0,
            renderer,
            is_running: true,
            minibuffer: Minibuffer::default(),
        }
    }

    pub fn add_buffer(&mut self, buffer: Buffer) {
        self.buffers.push(buffer);
    }

    // Borrows the current buffer
    pub fn get_active_buffer(&mut self) -> Result<&Buffer> {
        if let Some(buffer) = self.buffers.get(self.active_buffer) {
            Ok(buffer)
        } else {
            Err(OxideError::IndexError)
        }
    }

    // Borrows the current buffer as mutable
    pub fn get_active_buffer_mut(&mut self) -> Result<&mut Buffer> {
        if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
            Ok(buffer)
        } else {
            Err(OxideError::IndexError)
        }
    }

    // Calls the rendering function to not borrow past the editor's lifetime
    pub fn render(&mut self) -> Result<()> {
        let buffer = &self.buffers[self.active_buffer];

        let minibuffer: Option<&Minibuffer> = if buffer.mode == Mode::Minibuffer {
            Some(&self.minibuffer)
        } else {
            None
        };

        self.renderer.render(buffer, minibuffer)?;

        Ok(())
    }

    // Parses the keybinding and executes the corresponding action
    pub fn parse_action(
        &mut self,
        action: Action,
        keybinding_manager: &KeybindingManager,
        tokio_runtime: &tokio::runtime::Runtime,
    ) -> Result<()> {
        if self.get_active_buffer()?.mode != Mode::Minibuffer {
            match action {
                Action::SwitchMode(mode) => {
                    self.get_active_buffer_mut()?.switch_mode(mode);
                }
                Action::InsertChar(c) => self.get_active_buffer_mut()?.add_char(c)?,
                Action::InsertTab => self.get_active_buffer_mut()?.add_tab()?,
                Action::NewLine(direction) => self.get_active_buffer_mut()?.new_line(direction),
                Action::DeleteLine => self.get_active_buffer_mut()?.delete_line(),
                Action::MoveCursor(x, y) => self.get_active_buffer_mut()?.move_cursor(x, y),
                Action::TopOfBuffer => self.get_active_buffer_mut()?.move_cursor_to_top(),
                Action::EndOfBuffer => self.get_active_buffer_mut()?.move_cursor_to_bot(),
                Action::Quit => self.is_running = false,
                Action::DeleteChar => self.get_active_buffer_mut()?.remove_char()?,
                Action::WriteBuffer => {
                    tokio_runtime.block_on(self.get_active_buffer_mut()?.write_buffer())?
                }
                Action::ExecuteCommand => {
                    let input: &str = self.get_active_buffer_mut()?.get_command();
                    let commands = CommandParser::parse(input);

                    for command in commands {
                        self.parse_action(command, keybinding_manager, tokio_runtime)?;
                    }

                    self.get_active_buffer_mut()?
                        .switch_mode(ModeParams::Normal);
                }
                Action::OpenFile(path) => {
                    tokio_runtime.block_on(self.get_active_buffer_mut()?.load_file(&path))?;
                }
                Action::Minibuffer(kind) => {
                    self.get_active_buffer_mut()?.switch_mode(ModeParams::Minibuffer);

                    match kind {
                        MinibufferKind::Buffer(_) => {
                            let mut buffers: Vec<String> = Vec::new();

                            for buffer in &self.buffers {
                                buffers.push(buffer.title.clone());
                            }

                            self.minibuffer.kind = MinibufferKind::Buffer(buffers);
                        }
                        _ => self.minibuffer.kind = kind,
                    }

                    self.minibuffer.fill()?;
                }
                _ => {}
            }
        } else {
            match action {
                Action::Escape => self.get_active_buffer_mut()?.switch_mode(ModeParams::Normal),
                Action::InsertChar(c) => self.minibuffer.add_char(c)?,
                Action::MoveCursor(x, y) => self.minibuffer.move_cursor(x, y),
                Action::DeleteChar => self.minibuffer.remove_char()?,
                Action::Append => self.minibuffer.append(),
                Action::ExecuteCommand => match self.minibuffer.execute()? {
                    Some(action) => {
                        match action {
                            Action::OpenFile(path) => {
                                if self.get_active_buffer()?.path.is_some() {
                                    tokio_runtime.block_on(self.get_active_buffer_mut()?.load_file(&path))?;
                                } else {
                                    let height = self.renderer.get_terminal_size().height as usize;
                                    let buffer = tokio_runtime.block_on(Buffer::from_file(path, height))?;

                                    self.buffers.push(buffer);
                                    self.active_buffer = self.buffers.len() - 1;
                                }
                            },
                            Action::OpenBuffer(num) => {
                                if num < self.buffers.len() {
                                    self.active_buffer = num;
                                } else {
                                    return Err(OxideError::IndexError);
                                }
                            }
                            _ => {},
                        }

                        self.minibuffer = Minibuffer::default();
                        self.get_active_buffer_mut()?.switch_mode(ModeParams::Normal);
                    },
                    None => {},
                },
                _ => {},
            }

            self.minibuffer.fill()?;
        }

        Ok(())
    }
}
