use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::buffer::{BufferKind, MinibufferKind, Mode};
use crate::keybinding::actions::{self, Action, InsertDirection, ModeParams, NewLineDirection};

// ╭──────────────────────────────────────╮
// │ Keybinding Structs                   │
// ╰──────────────────────────────────────╯

// Stores the users currently pressed keys
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct KeySequence {
    pub keys: Vec<Keybinding>,
}

// Stores the key information for ease of access
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Keybinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
}

// Stores all available keybindings as well as the currently pressed one
pub struct KeybindingManager {
    mode_bindings:
        HashMap<Mode, HashMap<Option<BufferKind>, HashMap<KeySequence, Arc<dyn Action>>>>,
    current_buffer_kind: BufferKind,
    current_sequence: KeySequence,
}

// Handles parsing the command line commands
pub struct CommandParser;

impl KeybindingManager {
    pub fn new() -> Self {
        let mut manager = KeybindingManager {
            mode_bindings: HashMap::new(),
            current_buffer_kind: BufferKind::Normal,
            current_sequence: KeySequence { keys: Vec::new() },
        };

        manager.setup_default_bindings();
        manager
    }

    // Defines all default keybindings
    fn setup_default_bindings(&mut self) {
        // NORMAL MODE
        self.add_binding(
            Mode::Normal,
            None,
            vec![(KeyCode::Char('n'), KeyModifiers::NONE)],
            Arc::new(actions::MoveCursorAction::new(-1, 0)),
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![(KeyCode::Char('e'), KeyModifiers::NONE)],
            Arc::new(actions::MoveCursorAction::new(0, 1)),
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![(KeyCode::Char('i'), KeyModifiers::NONE)],
            Arc::new(actions::MoveCursorAction::new(0, -1)),
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![(KeyCode::Char('o'), KeyModifiers::NONE)],
            Arc::new(actions::MoveCursorAction::new(1, 0)),
        );

        self.add_binding(
            Mode::Normal,
            Some(BufferKind::Normal),
            vec![(KeyCode::Char('s'), KeyModifiers::NONE)],
            Arc::new(actions::SwitchModeAction::new(ModeParams::Insert {
                insert_direction: InsertDirection::Before,
            })),
        );

        self.add_binding(
            Mode::Normal,
            Some(BufferKind::Normal),
            vec![(KeyCode::Char('S'), KeyModifiers::SHIFT)],
            Arc::new(actions::SwitchModeAction::new(ModeParams::Insert {
                insert_direction: InsertDirection::Beginning,
            })),
        );

        self.add_binding(
            Mode::Normal,
            Some(BufferKind::Normal),
            vec![(KeyCode::Char('a'), KeyModifiers::NONE)],
            Arc::new(actions::SwitchModeAction::new(ModeParams::Insert {
                insert_direction: InsertDirection::After,
            })),
        );

        self.add_binding(
            Mode::Normal,
            Some(BufferKind::Normal),
            vec![(KeyCode::Char('A'), KeyModifiers::SHIFT)],
            Arc::new(actions::SwitchModeAction::new(ModeParams::Insert {
                insert_direction: InsertDirection::End,
            })),
        );

        self.add_binding(
            Mode::Normal,
            Some(BufferKind::Normal),
            vec![(KeyCode::Char('x'), KeyModifiers::NONE)],
            Arc::new(actions::DeleteCharAction),
        );

        self.add_binding(
            Mode::Normal,
            Some(BufferKind::Normal),
            vec![
                (KeyCode::Char('d'), KeyModifiers::NONE),
                (KeyCode::Char('d'), KeyModifiers::NONE),
            ],
            Arc::new(actions::DeleteLineAction),
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![
                (KeyCode::Char('g'), KeyModifiers::NONE),
                (KeyCode::Char('g'), KeyModifiers::NONE),
            ],
            Arc::new(actions::TopOfBufferAction),
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![(KeyCode::Char('G'), KeyModifiers::SHIFT)],
            Arc::new(actions::BotOfBufferAction),
        );

        self.add_binding(
            Mode::Normal,
            Some(BufferKind::Normal),
            vec![(KeyCode::Char('f'), KeyModifiers::NONE)],
            Arc::new(actions::NewLineAction::new(NewLineDirection::Under)),
        );

        self.add_binding(
            Mode::Normal,
            Some(BufferKind::Normal),
            vec![(KeyCode::Char('F'), KeyModifiers::SHIFT)],
            Arc::new(actions::NewLineAction::new(NewLineDirection::Over)),
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![(KeyCode::Char(':'), KeyModifiers::NONE)],
            Arc::new(actions::SwitchModeAction::new(ModeParams::Command {
                prefix: ":".to_string(),
            })),
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![(KeyCode::Char('v'), KeyModifiers::NONE)],
            Arc::new(actions::SwitchModeAction::new(ModeParams::Visual)),
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![
                (KeyCode::Char(' '), KeyModifiers::NONE),
                (KeyCode::Char('f'), KeyModifiers::NONE),
                (KeyCode::Char('f'), KeyModifiers::NONE),
            ],
            Arc::new(actions::MinibufferAction::new(MinibufferKind::File(
                PathBuf::new(),
            ))),
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![
                (KeyCode::Char(' '), KeyModifiers::NONE),
                (KeyCode::Char('f'), KeyModifiers::NONE),
                (KeyCode::Char('b'), KeyModifiers::NONE),
            ],
            Arc::new(actions::MinibufferAction::new(MinibufferKind::Buffer(
                Vec::new(),
            ))),
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![(KeyCode::Esc, KeyModifiers::NONE)],
            Arc::new(actions::EscapeAction),
        );

        // INSERT MODE
        self.add_binding(
            Mode::Insert,
            None,
            vec![(KeyCode::Esc, KeyModifiers::NONE)],
            Arc::new(actions::SwitchModeAction::new(ModeParams::Normal)),
        );

        self.add_binding(
            Mode::Insert,
            None,
            vec![(KeyCode::Enter, KeyModifiers::NONE)],
            Arc::new(actions::NewLineAction::new(NewLineDirection::Under)),
        );

        // VISUAL MODE
        self.add_binding(
            Mode::Visual,
            None,
            vec![(KeyCode::Char('n'), KeyModifiers::NONE)],
            Arc::new(actions::MoveCursorAction::new(-1, 0)),
        );

        self.add_binding(
            Mode::Visual,
            None,
            vec![(KeyCode::Char('e'), KeyModifiers::NONE)],
            Arc::new(actions::MoveCursorAction::new(0, 1)),
        );

        self.add_binding(
            Mode::Visual,
            None,
            vec![(KeyCode::Char('i'), KeyModifiers::NONE)],
            Arc::new(actions::MoveCursorAction::new(0, -1)),
        );

        self.add_binding(
            Mode::Visual,
            None,
            vec![(KeyCode::Char('o'), KeyModifiers::NONE)],
            Arc::new(actions::MoveCursorAction::new(1, 0)),
        );

        self.add_binding(
            Mode::Visual,
            None,
            vec![(KeyCode::Char('d'), KeyModifiers::NONE)],
            Arc::new(actions::DeleteCharAction),
        );

        self.add_binding(
            Mode::Visual,
            None,
            vec![(KeyCode::Char('x'), KeyModifiers::NONE)],
            Arc::new(actions::DeleteCharAction),
        );

        self.add_binding(
            Mode::Visual,
            None,
            vec![(KeyCode::Esc, KeyModifiers::NONE)],
            Arc::new(actions::SwitchModeAction::new(ModeParams::Normal)),
        );

        self.add_binding(
            Mode::Visual,
            None,
            vec![
                (KeyCode::Char('g'), KeyModifiers::NONE),
                (KeyCode::Char('g'), KeyModifiers::NONE),
            ],
            Arc::new(actions::TopOfBufferAction),
        );

        self.add_binding(
            Mode::Visual,
            None,
            vec![(KeyCode::Char('G'), KeyModifiers::SHIFT)],
            Arc::new(actions::BotOfBufferAction),
        );

        // COMMAND MODE
        self.add_binding(
            Mode::Command,
            None,
            vec![(KeyCode::Esc, KeyModifiers::NONE)],
            Arc::new(actions::SwitchModeAction::new(ModeParams::Normal)),
        );

        self.add_binding(
            Mode::Command,
            None,
            vec![(KeyCode::Enter, KeyModifiers::NONE)],
            Arc::new(actions::ExecuteCommandAction),
        );

        self.add_binding(
            Mode::Command,
            None,
            vec![(KeyCode::Left, KeyModifiers::NONE)],
            Arc::new(actions::MoveCursorAction::new(-1, 0)),
        );

        self.add_binding(
            Mode::Command,
            None,
            vec![(KeyCode::Right, KeyModifiers::NONE)],
            Arc::new(actions::MoveCursorAction::new(1, 0)),
        );

        // MINIBUFFER MODE
        self.add_binding(
            Mode::Minibuffer,
            None,
            vec![(KeyCode::Esc, KeyModifiers::NONE)],
            Arc::new(actions::EscapeAction),
        );

        self.add_binding(
            Mode::Minibuffer,
            None,
            vec![(KeyCode::Enter, KeyModifiers::NONE)],
            Arc::new(actions::ExecuteMbCommandAction),
        );

        self.add_binding(
            Mode::Minibuffer,
            None,
            vec![(KeyCode::Left, KeyModifiers::NONE)],
            Arc::new(actions::MoveMbCursorAction::new(-1, 0)),
        );

        self.add_binding(
            Mode::Minibuffer,
            None,
            vec![(KeyCode::Down, KeyModifiers::NONE)],
            Arc::new(actions::MoveMbCursorAction::new(0, 1)),
        );

        self.add_binding(
            Mode::Minibuffer,
            None,
            vec![(KeyCode::Up, KeyModifiers::NONE)],
            Arc::new(actions::MoveMbCursorAction::new(0, -1)),
        );

        self.add_binding(
            Mode::Minibuffer,
            None,
            vec![(KeyCode::Right, KeyModifiers::NONE)],
            Arc::new(actions::MoveMbCursorAction::new(1, 0)),
        );

        self.add_binding(
            Mode::Minibuffer,
            None,
            vec![(KeyCode::Tab, KeyModifiers::NONE)],
            Arc::new(actions::AppendAction),
        );
    }

    // Adds keybindings to the keybinding manager
    pub fn add_binding(
        &mut self,
        mode: Mode,
        buffer_kind: Option<BufferKind>,
        key_sequence: Vec<(KeyCode, KeyModifiers)>,
        action: Arc<dyn Action>,
    ) {
        // Parses the key sequence
        let sequence = KeySequence {
            keys: key_sequence
                .into_iter()
                .map(|(key, modifiers)| Keybinding { key, modifiers })
                .collect(),
        };

        // Creates a new entry
        self.mode_bindings
            .entry(mode)
            .or_insert_with(HashMap::new)
            .entry(buffer_kind)
            .or_insert_with(HashMap::new)
            .insert(sequence, action);
    }

    // Checks the mode of the keybinding and the current buffer mode and redirects to the
    // appropriate parser
    pub fn handle_input(
        &mut self,
        current_mode: &Mode,
        key_event: KeyEvent,
    ) -> Option<Arc<dyn Action>> {
        let key_binding = Keybinding {
            key: key_event.code,
            modifiers: key_event.modifiers,
        };

        self.current_sequence.keys.push(key_binding);

        let action = match current_mode {
            Mode::Normal => self.handle_normal_mode(current_mode),
            Mode::Insert => self.handle_insert_mode(current_mode, key_binding),
            Mode::Visual => self.handle_visual_mode(current_mode),
            Mode::Command => self.handle_command_mode(current_mode, key_binding),
            Mode::Minibuffer => self.handle_minibuffer_mode(current_mode, key_binding),
        };

        // If the keybinding exists, it's sent back
        // If not it checks if the current key sequence exists in any existing
        // keybinding and stores the current key sequence
        if action.is_some() {
            self.current_sequence.keys.clear();
            return Some(action.unwrap());
        } else {
            if let Some(mode_bindings) = self.mode_bindings.get(current_mode) {
                let mut sequence_matches = false;

                // Checks if keybinding exists in any buffer kind
                if let Some(bindings) = mode_bindings.get(&None) {
                    sequence_matches = bindings
                        .keys()
                        .any(|seq| seq.keys.starts_with(&self.current_sequence.keys));
                }

                if !sequence_matches {
                    // Checks if keybinding exists in the current buffer kind
                    if let Some(bindings) = mode_bindings.get(&Some(self.current_buffer_kind)) {
                        sequence_matches = bindings
                            .keys()
                            .any(|seq| seq.keys.starts_with(&self.current_sequence.keys));
                    }
                }

                // If not, it clears the current key sequence
                if !sequence_matches {
                    self.current_sequence.keys.clear();
                }
            }
        }

        None
    }

    fn handle_normal_mode(&self, current_mode: &Mode) -> Option<Arc<dyn Action>> {
        if let Some(mode_bindings) = self.mode_bindings.get(current_mode) {
            if let Some(action) = mode_bindings
                .get(&Some(self.current_buffer_kind.clone()))
                .and_then(|bindings| bindings.get(&self.current_sequence))
            {
                return Some(action.clone());
            } else if let Some(action) = mode_bindings
                .get(&None)
                .and_then(|bindings| bindings.get(&self.current_sequence))
            {
                return Some(action.clone());
            }
        }

        None
    }

    fn handle_insert_mode(
        &self,
        current_mode: &Mode,
        key_binding: Keybinding,
    ) -> Option<Arc<dyn Action>> {
        match key_binding {
            Keybinding {
                key: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
            } => Some(Arc::new(actions::AddCharAction::new(c))),
            Keybinding {
                key: KeyCode::Char(c),
                modifiers: KeyModifiers::SHIFT,
            } => Some(Arc::new(actions::AddCharAction::new(c))),
            Keybinding {
                key: KeyCode::Tab,
                modifiers: KeyModifiers::NONE,
            } => Some(Arc::new(actions::AddTabAction)),
            Keybinding {
                key: KeyCode::Tab,
                modifiers: KeyModifiers::SHIFT,
            } => Some(Arc::new(actions::AddTabAction)),
            Keybinding {
                key: KeyCode::Backspace,
                ..
            } => Some(Arc::new(actions::DeleteCharAction)),
            Keybinding {
                key: KeyCode::Enter,
                ..
            } => Some(Arc::new(actions::NewLineAction::new(
                NewLineDirection::Under,
            ))),
            _ => {
                if let Some(mode_bindings) = self.mode_bindings.get(current_mode) {
                    if let Some(action) = mode_bindings
                        .get(&Some(self.current_buffer_kind.clone()))
                        .and_then(|bindings| bindings.get(&self.current_sequence))
                    {
                        return Some(action.clone());
                    } else if let Some(action) = mode_bindings
                        .get(&None)
                        .and_then(|bindings| bindings.get(&self.current_sequence))
                    {
                        return Some(action.clone());
                    }
                }

                None
            }
        }
    }

    fn handle_visual_mode(&self, current_mode: &Mode) -> Option<Arc<dyn Action>> {
        if let Some(mode_bindings) = self.mode_bindings.get(current_mode) {
            if let Some(action) = mode_bindings
                .get(&Some(self.current_buffer_kind.clone()))
                .and_then(|bindings| bindings.get(&self.current_sequence))
            {
                return Some(action.clone());
            } else if let Some(action) = mode_bindings
                .get(&None)
                .and_then(|bindings| bindings.get(&self.current_sequence))
            {
                return Some(action.clone());
            }
        }

        None
    }

    fn handle_command_mode(
        &self,
        current_mode: &Mode,
        key_binding: Keybinding,
    ) -> Option<Arc<dyn Action>> {
        match key_binding {
            Keybinding {
                key: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
            } => Some(Arc::new(actions::AddCharAction::new(c))),
            Keybinding {
                key: KeyCode::Char(c),
                modifiers: KeyModifiers::SHIFT,
            } => Some(Arc::new(actions::AddCharAction::new(c))),
            Keybinding {
                key: KeyCode::Backspace,
                ..
            } => Some(Arc::new(actions::DeleteCharAction)),
            _ => {
                if let Some(mode_bindings) = self.mode_bindings.get(current_mode) {
                    if let Some(action) = mode_bindings
                        .get(&Some(self.current_buffer_kind.clone()))
                        .and_then(|bindings| bindings.get(&self.current_sequence))
                    {
                        return Some(action.clone());
                    } else if let Some(action) = mode_bindings
                        .get(&None)
                        .and_then(|bindings| bindings.get(&self.current_sequence))
                    {
                        return Some(action.clone());
                    }
                }

                None
            }
        }
    }

    fn handle_minibuffer_mode(
        &self,
        current_mode: &Mode,
        key_binding: Keybinding,
    ) -> Option<Arc<dyn Action>> {
        match key_binding {
            Keybinding {
                key: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
            } => Some(Arc::new(actions::AddMbCharAction::new(c))),
            Keybinding {
                key: KeyCode::Char(c),
                modifiers: KeyModifiers::SHIFT,
            } => Some(Arc::new(actions::AddMbCharAction::new(c))),
            Keybinding {
                key: KeyCode::Backspace,
                ..
            } => Some(Arc::new(actions::DeleteMbCharAction)),
            Keybinding {
                key: KeyCode::Esc, ..
            } => Some(Arc::new(actions::EscapeAction)),
            _ => {
                if let Some(mode_bindings) = self.mode_bindings.get(current_mode) {
                    if let Some(action) = mode_bindings
                        .get(&Some(self.current_buffer_kind.clone()))
                        .and_then(|bindings| bindings.get(&self.current_sequence))
                    {
                        return Some(action.clone());
                    } else if let Some(action) = mode_bindings
                        .get(&None)
                        .and_then(|bindings| bindings.get(&self.current_sequence))
                    {
                        return Some(action.clone());
                    }
                }

                None
            }
        }
    }

    pub fn set_buffer_kind(&mut self, kind: BufferKind) {
        self.current_buffer_kind = kind;
    }
}

impl CommandParser {
    pub fn parse(input: &str) -> Vec<Arc<dyn Action>> {
        match input {
            "wq" => vec![
                Arc::new(actions::WriteBufferAction),
                Arc::new(actions::QuitAction),
            ],
            "w" => vec![Arc::new(actions::WriteBufferAction)],
            "q" => vec![Arc::new(actions::QuitAction)],
            _ => Vec::new(),
        }
    }
}
