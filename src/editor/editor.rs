use ratatui::prelude::*;
use ratatui::Terminal;

use std::io::Stdout;

use crate::buffer::{
    Buffer, BufferKind, Manipulation, Minibuffer, MinibufferKind, Mode, Navigation,
};
use crate::keybinding::{Action, CommandParser, KeybindingManager, ModeParams, COMMANDS};
use crate::renderer::Renderer;
use crate::OxideError;

// ╭──────────────────────────────────────╮
// │ Editor Types                         │
// ╰──────────────────────────────────────╯

type Result<T> = std::result::Result<T, crate::OxideError>;

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
    pub fn get_active_buffer(&mut self) -> &Buffer {
        if self.buffers[self.active_buffer].kind == BufferKind::BufferList {
            self.buffers[self.active_buffer].content = self
                .buffers
                .iter()
                .map(|buffer| buffer.title.clone())
                .collect();
        }

        &self.buffers[self.active_buffer]
    }

    // Borrows the current buffer as mutable
    pub fn get_active_buffer_mut(&mut self) -> &mut Buffer {
        if self.buffers[self.active_buffer].kind == BufferKind::BufferList {
            self.buffers[self.active_buffer].content = self
                .buffers
                .iter()
                .map(|buffer| buffer.title.clone())
                .collect();
        }

        &mut self.buffers[self.active_buffer]
    }

    // Calls the rendering function to not borrow past the editor's lifetime
    pub fn render(&mut self) -> Result<()> {
        let buffer = &self.buffers[self.active_buffer];

        self.renderer.render_buffer(buffer)?;

        Ok(())
    }

    fn fill_minibuffer(&mut self) {
        let mut matches: Vec<String> = Vec::new();

        match self.minibuffer.kind {
            MinibufferKind::Command => {
                for command in COMMANDS {
                    if command.contains(&self.minibuffer.input) {
                        matches.push(String::from(command));
                    }
                }

                let match_len = matches.len();
                self.minibuffer.content = matches;

                if match_len != 0 && self.minibuffer.cursor.y > match_len - 1 {
                    self.minibuffer.cursor.y = match_len - 1;
                }
            }
            _ => {}
        }
    }

    // Parses the keybinding and executes the corresponding action
    pub fn parse_action(
        &mut self,
        action: Action,
        keybinding_manager: &KeybindingManager,
        tokio_runtime: &tokio::runtime::Runtime,
    ) -> Result<()> {
        match action {
            Action::SwitchMode(mode) => self.get_active_buffer_mut().switch_mode(mode),
            Action::InsertChar(c) => {
                match self.get_active_buffer_mut().add_char(c) {
                    Ok(_) => {}
                    Err(e) => return Err(OxideError::BufferError(e)),
                }

                if self.get_active_buffer().mode == Mode::Command {
                    self.fill_minibuffer();
                }
            }
            Action::InsertTab => match self.get_active_buffer_mut().add_tab() {
                Ok(_) => {}
                Err(e) => return Err(OxideError::BufferError(e)),
            },
            Action::NewLine(direction) => self.get_active_buffer_mut().new_line(direction),
            Action::DeleteLine => self.get_active_buffer_mut().delete_line(),
            Action::MoveCursor(x, y) => self.get_active_buffer_mut().move_cursor(x, y),
            Action::TopOfBuffer => self.get_active_buffer_mut().move_cursor_to_top(),
            Action::EndOfBuffer => self.get_active_buffer_mut().move_cursor_to_bot(),
            Action::Quit => self.is_running = false,
            Action::DeleteChar => {
                match self.get_active_buffer_mut().remove_char() {
                    Ok(_) => {}
                    Err(e) => return Err(OxideError::BufferError(e)),
                }

                if self.get_active_buffer().mode == Mode::Command {
                    self.fill_minibuffer();
                }
            }
            Action::WriteBuffer => {
                tokio_runtime.block_on(self.get_active_buffer_mut().write_buffer())?;
            }
            Action::ExecuteCommand => {
                let input: &str = self.get_active_buffer_mut().get_command();
                let commands = CommandParser::parse(input);

                for command in commands {
                    self.parse_action(command, keybinding_manager, tokio_runtime)?;
                }

                self.get_active_buffer_mut().switch_mode(ModeParams::Normal);
            }
            Action::OpenFile(path) => {
                tokio_runtime.block_on(self.get_active_buffer_mut().load_file(path))?;
            }
            _ => {}
        };

        Ok(())
    }
}
