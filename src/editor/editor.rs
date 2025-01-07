use ratatui::prelude::*;
use ratatui::Terminal;
use tokio::runtime::Runtime;

use std::io::Stdout;

use crate::buffer::{Buffer, Minibuffer, Mode};
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

pub struct BufferManager {
    pub buffers: Vec<Buffer>,
    pub active_buffer: usize,
}

impl BufferManager {
    fn new(height: usize) -> Self {
        BufferManager {
            buffers: vec![Buffer::scratch(height)],
            active_buffer: 0,
        }
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

    pub fn add_buffer(&mut self, buffer: Buffer) {
        self.buffers.push(buffer);
    }
}

pub struct Editor {
    pub buffer_manager: BufferManager,
    pub renderer: Renderer,
    pub is_running: bool,
    pub minibuffer: Minibuffer,
    pub runtime: Runtime,
}

impl Editor {
    pub fn new(terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<Self> {
        let renderer = Renderer::new(terminal);
        let height = renderer.get_terminal_size().height as usize;
        let buffer_manager = BufferManager::new(height);
        let minibuffer = Minibuffer::default();
        let runtime = Runtime::new()?;

        Ok(Editor {
            buffer_manager,
            renderer,
            is_running: true,
            minibuffer,
            runtime,
        })
    }

    // Calls the rendering function to not borrow past the editor's lifetime
    pub fn render(&mut self) -> Result<()> {
        let buffer = &self.buffer_manager.buffers[self.buffer_manager.active_buffer];

        let minibuffer: Option<&Minibuffer> = if buffer.mode == Mode::Minibuffer {
            Some(&self.minibuffer)
        } else {
            None
        };

        self.renderer.render(buffer, minibuffer)?;

        Ok(())
    }
}
