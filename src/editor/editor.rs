use ratatui::prelude::*;
use ratatui::Terminal;

use anyhow;

use std::cell::{Ref, RefMut, RefCell};
use std::rc::Rc;
use std::io::Stdout;

use crate::buffer::Buffer;
use crate::renderer::Renderer;

pub struct Editor {
    pub buffers: Vec<Rc<RefCell<Buffer>>>,
    pub active_buffer: usize,
    pub renderer: Renderer,
    pub is_running: bool
}

impl Editor {
    pub fn new(terminal: Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<Self> {
        let renderer = Renderer::new(terminal);
        let height = renderer.get_terminal_size()?.height as usize;

        Ok(Editor {
            buffers: vec![Buffer::scratch(height)],
            active_buffer: 0,
            renderer,
            is_running: true,
        })
    }

    pub fn add_buffer(&mut self, buffer: Rc<RefCell<Buffer>>) {
        self.buffers.push(buffer);
    }

    pub fn get_active_buffer(&self) -> Ref<Buffer> {
        self.buffers[self.active_buffer].borrow()
    }

    pub fn get_active_buffer_mut(&self) -> RefMut<Buffer> {
        self.buffers[self.active_buffer].borrow_mut()
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.renderer.render(self.buffers[self.active_buffer].borrow())?;

        Ok(())
    }
}
