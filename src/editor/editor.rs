use ratatui::backend::Backend;
use ratatui::Terminal;

use std::cell::RefCell;
use std::rc::Rc;

use crate::buffer::Buffer;
use crate::renderer::Renderer;
use crate::keybinding::Keybindings;

enum Mode {
    Normal,
    Insert,
    Command,
}

pub struct Editor<B: Backend> {
    pub buffers: Vec<Rc<RefCell<Buffer>>>,
    pub active_buffer: usize,
    pub mode: Mode,
    pub renderer: Renderer<B>,
    pub keybindings: Keybindings,
}

impl<B: Backend> Editor<B> {
    pub fn new(terminal: Terminal<B>) -> Self {

        Editor {
            buffers: vec![Buffer::scratch()],
            active_buffer: 0,
            mode: Mode::Normal,
            renderer: Renderer::new(terminal),
            keybindings: Keybindings::new(),
        }
    }

    pub fn add_buffer(&mut self, buffer: Rc<RefCell<Buffer>>) {
        self.buffers.push(buffer);
    }

    fn add_keybindings(&mut self) {
    }
}
