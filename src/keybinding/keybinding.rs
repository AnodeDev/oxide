use ratatui::crossterm::event::{self, Event, KeyEventKind, KeyCode, KeyEvent};

use std::collections::HashMap;
use std::any::Any;
use std::rc::Rc;

use crate::buffer::Mode;
use crate::editor::Editor;

#[derive(Eq, Hash, PartialEq)]
pub enum KeybindingContext {
    Global,
    Buffer,
    Mode(Mode),
}

pub type KeyCombination = Vec<KeyEvent>;

pub trait ActionFn: Fn(&mut dyn Any) + 'static {}
impl<F: Fn(&mut dyn Any) + 'static> ActionFn for F {}

pub struct Action {
    pub id: &'static str,
    pub function: Rc<dyn ActionFn>,
    pub description: &'static str,
}

pub struct KeybindingRegistry {
    bindings: HashMap<KeyCombination, (KeybindingContext, Action)>,
}

impl KeybindingRegistry {
    pub fn new() -> Self {
        KeybindingRegistry {
            bindings: HashMap::new(),
        }
    }

    pub fn register_keybinding(
        &mut self,
        keys: KeyCombination,
        context: KeybindingContext,
        action: Action,
    ) {
        self.bindings.insert(keys, (context, action));
    }

    pub fn process_key_event(&mut self, keys: KeyCombination, editor: &mut Editor) {
        if let Some((context, action)) = self.bindings.get_mut(&keys) {
            match context {
                KeybindingContext::Global => {
                    (action.function)(&mut *editor);
                },
                KeybindingContext::Buffer => {
                    let mut buffer = editor.get_active_buffer_mut();
                    (action.function)(&mut *buffer);
                },
                _ => {}
            }
        }
    }

    pub async fn read_keys(&mut self) -> anyhow::Result<KeyCombination> {
        let mut keys: KeyCombination = Vec::new();

        loop {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Esc => {
                            keys.clear();

                            break;
                        },
                        _ => keys.push(key),
                    }
                }
                _ => {}
            }

            let matching_keys = self.bindings.keys().filter(|combination| combination.starts_with(&keys)).count();

            if matching_keys == 0 {
                keys.clear();

                break;
            } else if let Some(_) = self.bindings.get(&keys) {
                break;
            }
        }

        Ok(keys)
    }
}
