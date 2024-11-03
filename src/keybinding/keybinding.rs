use ratatui::crossterm::event::{self, Event, KeyEventKind, KeyCode, KeyEvent};

use std::collections::HashMap;
use std::any::Any;
use std::rc::Rc;

use crate::buffer::{Mode, Manipulation};
use crate::editor::Editor;

#[derive(Eq, Hash, PartialEq)]
pub enum KeybindingContext {
    Global,
    Buffer,
}

enum Command {
    Nop,
    QuitEditor,
    SaveCurrentBuffer,
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
    bindings: HashMap<(KeyCombination, Mode), (KeybindingContext, Action)>,
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
        mode: Mode,
        context: KeybindingContext,
        action: Action,
    ) {
        self.bindings.insert((keys, mode), (context, action));
    }

    fn process_command(&mut self, command: &str) -> Command {
        match command {
            "q" => Command::QuitEditor,
            "w" => Command::SaveCurrentBuffer,
            _ => Command::Nop,
        }
    }

    pub async fn process_key_event(&mut self, keys: KeyCombination, mode: Mode, editor: &mut Editor) -> anyhow::Result<()> {
        if mode == Mode::Insert {
            let mut buffer = editor.get_active_buffer_mut();

            if let KeyCode::Char(c) = keys[0].code {
                buffer.add_char(c);
            } else if keys[0].code == KeyCode::Backspace {
                buffer.remove_char();
            } else if KeyCode::Enter == keys[0].code {
                buffer.new_line();
            } else {
                if let Some((_context, action)) = self.bindings.get_mut(&(keys, mode)) {
                    (action.function)(&mut *buffer);
                }
            }

        } else if mode == Mode::Command {
            if let KeyCode::Char(c) = keys[0].code {
                editor.get_active_buffer_mut().add_char_commandline(c);
            } else if keys[0].code == KeyCode::Backspace {
                editor.get_active_buffer_mut().remove_char_commandline();
            } else if keys[0].code == KeyCode::Enter {
                let command = self.process_command(&editor.get_active_buffer_mut().give_command());

                match command {
                    Command::QuitEditor => editor.should_quit = true,
                    Command::SaveCurrentBuffer => editor.get_active_buffer_mut().save_buffer().await?,
                    _ => {},
                }

                editor.get_active_buffer_mut().mode = Mode::Normal;
            } else {
                if let Some((_context, action)) = self.bindings.get_mut(&(keys, mode)) {
                    let mut buffer = editor.get_active_buffer_mut();

                    (action.function)(&mut *buffer);
                }
            }
        } else if let Some((context, action)) = self.bindings.get_mut(&(keys, mode)) {
            match context {
                KeybindingContext::Global => {
                    (action.function)(&mut *editor);
                },
                KeybindingContext::Buffer => {
                    let mut buffer = editor.get_active_buffer_mut();
                    (action.function)(&mut *buffer);
                },
            }
        }

        Ok(())
    }

    pub async fn read_keys(&mut self, mode: Mode) -> anyhow::Result<KeyCombination> {
        let mut keys: KeyCombination = Vec::new();

        loop {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if mode == Mode::Insert || mode == Mode::Command {
                        keys.push(key);

                        break;
                    }
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

            let matching_keys = self.bindings.keys()
                .filter(|(combination, binding_mode)| combination.starts_with(&keys) && binding_mode == &mode)
                .count();

            if matching_keys == 0 {
                keys.clear();

                break;
            }

            let temp_keys = std::mem::take(&mut keys);

            if let Some(_) = self.bindings.get(&(temp_keys.clone(), mode)) {
                keys = temp_keys;

                break;
            }

            keys = temp_keys;
        }

        Ok(keys)
    }
}
