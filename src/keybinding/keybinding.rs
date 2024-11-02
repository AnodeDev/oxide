use ratatui::crossterm::event::KeyCode;
use ratatui::backend::Backend;

use anyhow;

use std::collections::HashMap;

use crate::editor::Editor;

#[derive(PartialEq, Eq, Hash)]
pub enum Key {
    Single(KeyCode),
    Ctrl(KeyCode),
    Alt(KeyCode),
    Shift(KeyCode),
    Leader((KeyCode, KeyCode)),
    Chord(Vec<Key>),
}

type Action = Box<dyn Fn()>; // TODO!

pub struct Keybindings {
    bindings: HashMap<Key, Action>,
}

impl Keybindings {
    pub fn new() -> Self {
        Keybindings { bindings: HashMap::new() }
    }

    pub fn bind(&mut self, key: Key, action: Action) {
        self.bindings.insert(key, action);
    }

    pub fn execute(&self, key: &Key) {
        if let Some(action) = self.bindings.get(key) {
            action();
        }
    }
}
