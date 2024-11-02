use ratatui::crossterm::event::{self, Event, KeyEventKind, KeyCode};

use std::collections::HashMap;
use std::cell::RefMut;

use crate::buffer::Mode;

#[derive(Eq, Hash, PartialEq)]
pub enum KeybindingContext {
    Global,
    Buffer,
    Mode(Mode),
}

pub type KeyCombination = Vec<KeyCode>;

pub struct Action<T> {
    pub id: &'static str,
    pub function: Box<dyn FnMut(RefMut<T>)>,
    pub description: &'static str,
}

pub struct KeybindingRegistry<T> {
    bindings: HashMap<(KeybindingContext, KeyCombination), Action<T>>,
}

impl<T> KeybindingRegistry<T> {
    pub fn new() -> Self {
        KeybindingRegistry {
            bindings: HashMap::new(),
        }
    }

    pub fn register_keybinding(
        &mut self,
        context: KeybindingContext,
        keys: KeyCombination,
        action: Action<T>,
    ) {
        self.bindings.insert((context, keys), action);
    }

    pub fn process_key_event(&mut self, context: KeybindingContext, keys: KeyCombination, target: RefMut<T>) {
        if let Some(action) = self.bindings.get_mut(&(context, keys)) {
            (action.function)(target);
        }
    }

    pub async fn read_keys(&mut self) -> anyhow::Result<KeyCombination> {
        let mut keys: KeyCombination = Vec::new();

        loop {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Esc => {
                        keys.clear();

                        break;
                    },
                    _ => keys.push(key.code),
                },
                _ => {},
            }

            if !self.has_binding(&keys) {
                keys.clear();

                break;
            }
        }

        Ok(keys)
    }

    fn has_binding(&self, keys: &KeyCombination) -> bool {
        self.bindings.keys().any(|(_context, combination)| combination == keys)
    }
}
