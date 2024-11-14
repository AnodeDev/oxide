use ratatui::prelude::*;
use ratatui::Terminal;

use std::cell::{Ref, RefMut, RefCell};
use std::rc::Rc;
use std::io::Stdout;

use crate::buffer::{Buffer, Manipulation, Mode};
use crate::renderer::Renderer;
use crate::keybinding::{Action, CommandParser, KeybindingManager, ModeParams};

type Result<'a, T> = std::result::Result<T, crate::Error<'a>>;

/// Holds all the editor states
pub struct Editor {
    pub buffers: Vec<Rc<RefCell<Buffer>>>,
    pub active_buffer: usize,
    pub renderer: Renderer,
    pub is_running: bool
}

impl Editor {
    pub fn new(terminal: Terminal<CrosstermBackend<Stdout>>) -> Self {
        let renderer = Renderer::new(terminal);
        let height   = renderer.get_terminal_size().height as usize;

        Editor {
            buffers: vec![Buffer::scratch(height)],
            active_buffer: 0,
            renderer,
            is_running: true,
        }
    }

    pub fn add_buffer(&mut self, buffer: Rc<RefCell<Buffer>>) {
        self.buffers.push(buffer);
    }

    /// Borrows the current buffer
    pub fn get_active_buffer(&self) -> Ref<Buffer> {
        self.buffers[self.active_buffer].borrow()
    }

    /// Borrows the current buffer as mutable
    pub fn get_active_buffer_mut(&self) -> RefMut<Buffer> {
        self.buffers[self.active_buffer].borrow_mut()
    }

    /// Calls the rendering function to not borrow past the editor's lifetime
    pub fn render(&mut self) -> Result<()> {
        match self.renderer.render(self.buffers[self.active_buffer].borrow()) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("ERROR: {}", e);
            },
        };

        Ok(())
    }

    /// Parses the keybinding and executes the corresponding action
    pub fn parse_action(&mut self, action: Action,keybinding_manager: &RefCell<KeybindingManager>, tokio_runtime: &tokio::runtime::Runtime) -> Result<()> {
        match action {
            Action::SwitchMode(mode)         => self.get_active_buffer_mut().switch_mode(mode),
            Action::InsertChar(c)            => {
                match self.get_active_buffer_mut().add_char(c) {
                    Ok(_) => {},
                    Err(e) => {
                        eprintln!("ERROR: {}", e);
                    },
                }
            },
            Action::NewLine(direction)       => self.get_active_buffer_mut().new_line(direction),
            Action::DeleteChar               => self.get_active_buffer_mut().remove_char(),
            Action::DeleteLine               => self.get_active_buffer_mut().delete_line(),
            Action::MoveCursor(x, y)         => self.get_active_buffer_mut().move_cursor(x, y),
            Action::TopOfBuffer              => self.get_active_buffer_mut().move_cursor_to_top(),
            Action::EndOfBuffer              => self.get_active_buffer_mut().move_cursor_to_bot(),
            Action::Quit                     => self.is_running = false,
            Action::WriteBuffer              => {
                match tokio_runtime.block_on(self.get_active_buffer_mut().write_buffer()) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Action::ExecuteCommand => {
                let input: String = self.get_active_buffer_mut().get_command();
                let commands = CommandParser::parse(input);

                for command in commands {
                    match self.parse_action(command, keybinding_manager, tokio_runtime) {
                        Ok(_) => {},
                        Err(e) => {
                            eprintln!("ERROR: {}", e);
                        },
                    };
                }

                self.get_active_buffer_mut().switch_mode(ModeParams::Normal { mode: Mode::Normal });
            },
            Action::FindFile => {
                todo!("Implement Find File functionality")
            },
            _ => {},
        };

        Ok(())
    }
}
