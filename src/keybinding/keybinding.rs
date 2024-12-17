use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use std::collections::HashMap;

use crate::buffer::{BufferKind, Mode};

// ╭──────────────────────────────────────╮
// │ Keybinding Consts                    │
// ╰──────────────────────────────────────╯

pub const COMMANDS: [&str; 3] = ["wq", "w", "q"];

// ╭──────────────────────────────────────╮
// │ Keybinding Enums                     │
// ╰──────────────────────────────────────╯

// Defines all the available actions
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Action {
    Nop,
    SwitchMode(ModeParams),
    InsertChar(char),
    InsertTab,
    NewLine(NewLineDirection),
    DeleteChar,
    DeleteLine,
    MoveCursor(i32, i32),
    TopOfBuffer,
    EndOfBuffer,
    Quit,
    WriteBuffer,
    ExecuteCommand,
    OpenFile(String),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ModeParams {
    Normal,
    Insert { insert_direction: InsertDirection },
    Visual,
    Command { prefix: String, input: String },
}

// Defines where a new line can go
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum NewLineDirection {
    Under,
    Over,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum InsertDirection {
    Before,
    After,
}

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
    mode_bindings: HashMap<Mode, HashMap<Option<BufferKind>, HashMap<KeySequence, Action>>>,
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
            Action::MoveCursor(-1, 0),
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![(KeyCode::Char('e'), KeyModifiers::NONE)],
            Action::MoveCursor(0, 1),
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![(KeyCode::Char('i'), KeyModifiers::NONE)],
            Action::MoveCursor(0, -1),
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![(KeyCode::Char('o'), KeyModifiers::NONE)],
            Action::MoveCursor(1, 0),
        );

        self.add_binding(
            Mode::Normal,
            Some(BufferKind::Normal),
            vec![(KeyCode::Char('s'), KeyModifiers::NONE)],
            Action::SwitchMode(ModeParams::Insert {
                insert_direction: InsertDirection::Before,
            }),
        );

        self.add_binding(
            Mode::Normal,
            Some(BufferKind::Normal),
            vec![(KeyCode::Char('a'), KeyModifiers::NONE)],
            Action::SwitchMode(ModeParams::Insert {
                insert_direction: InsertDirection::After,
            }),
        );

        self.add_binding(
            Mode::Normal,
            Some(BufferKind::Normal),
            vec![(KeyCode::Char('x'), KeyModifiers::NONE)],
            Action::DeleteChar,
        );

        self.add_binding(
            Mode::Normal,
            Some(BufferKind::Normal),
            vec![
                (KeyCode::Char('d'), KeyModifiers::NONE),
                (KeyCode::Char('d'), KeyModifiers::NONE),
            ],
            Action::DeleteLine,
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![
                (KeyCode::Char('g'), KeyModifiers::NONE),
                (KeyCode::Char('g'), KeyModifiers::NONE),
            ],
            Action::TopOfBuffer,
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![(KeyCode::Char('G'), KeyModifiers::SHIFT)],
            Action::EndOfBuffer,
        );

        self.add_binding(
            Mode::Normal,
            Some(BufferKind::Normal),
            vec![(KeyCode::Char('f'), KeyModifiers::NONE)],
            Action::NewLine(NewLineDirection::Under),
        );

        self.add_binding(
            Mode::Normal,
            Some(BufferKind::Normal),
            vec![(KeyCode::Char('F'), KeyModifiers::SHIFT)],
            Action::NewLine(NewLineDirection::Over),
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![(KeyCode::Char(':'), KeyModifiers::NONE)],
            Action::SwitchMode(ModeParams::Command {
                prefix: ":".to_string(),
                input: String::new(),
            }),
        );

        self.add_binding(
            Mode::Normal,
            None,
            vec![(KeyCode::Char('v'), KeyModifiers::NONE)],
            Action::SwitchMode(ModeParams::Visual),
        );

        // INSERT MODE
        self.add_binding(
            Mode::Insert,
            None,
            vec![(KeyCode::Esc, KeyModifiers::NONE)],
            Action::SwitchMode(ModeParams::Normal),
        );

        self.add_binding(
            Mode::Insert,
            None,
            vec![(KeyCode::Enter, KeyModifiers::NONE)],
            Action::NewLine(NewLineDirection::Under),
        );

        // VISUAL MODE
        self.add_binding(
            Mode::Visual,
            None,
            vec![(KeyCode::Char('n'), KeyModifiers::NONE)],
            Action::MoveCursor(-1, 0),
        );

        self.add_binding(
            Mode::Visual,
            None,
            vec![(KeyCode::Char('e'), KeyModifiers::NONE)],
            Action::MoveCursor(0, 1),
        );

        self.add_binding(
            Mode::Visual,
            None,
            vec![(KeyCode::Char('i'), KeyModifiers::NONE)],
            Action::MoveCursor(0, -1),
        );

        self.add_binding(
            Mode::Visual,
            None,
            vec![(KeyCode::Char('o'), KeyModifiers::NONE)],
            Action::MoveCursor(1, 0),
        );

        self.add_binding(
            Mode::Visual,
            None,
            vec![(KeyCode::Char('d'), KeyModifiers::NONE)],
            Action::DeleteChar,
        );

        self.add_binding(
            Mode::Visual,
            None,
            vec![(KeyCode::Char('x'), KeyModifiers::NONE)],
            Action::DeleteChar,
        );

        self.add_binding(
            Mode::Visual,
            None,
            vec![(KeyCode::Esc, KeyModifiers::NONE)],
            Action::SwitchMode(ModeParams::Normal),
        );

        self.add_binding(
            Mode::Visual,
            None,
            vec![
                (KeyCode::Char('g'), KeyModifiers::NONE),
                (KeyCode::Char('g'), KeyModifiers::NONE),
            ],
            Action::TopOfBuffer,
        );

        self.add_binding(
            Mode::Visual,
            None,
            vec![(KeyCode::Char('G'), KeyModifiers::SHIFT)],
            Action::EndOfBuffer,
        );

        // COMMAND MODE
        self.add_binding(
            Mode::Command,
            None,
            vec![(KeyCode::Esc, KeyModifiers::NONE)],
            Action::SwitchMode(ModeParams::Normal),
        );

        self.add_binding(
            Mode::Command,
            None,
            vec![(KeyCode::Enter, KeyModifiers::NONE)],
            Action::ExecuteCommand,
        );

        self.add_binding(
            Mode::Command,
            None,
            vec![(KeyCode::Char('n'), KeyModifiers::CONTROL)],
            Action::MoveCursor(-1, 0),
        );

        self.add_binding(
            Mode::Command,
            None,
            vec![(KeyCode::Char('e'), KeyModifiers::CONTROL)],
            Action::MoveCursor(1, 0),
        );

        self.add_binding(
            Mode::Command,
            None,
            vec![(KeyCode::Char('i'), KeyModifiers::CONTROL)],
            Action::MoveCursor(0, -1),
        );

        self.add_binding(
            Mode::Command,
            None,
            vec![(KeyCode::Char('o'), KeyModifiers::CONTROL)],
            Action::MoveCursor(0, 1),
        );
    }

    // Adds keybindings to the keybinding manager
    pub fn add_binding(
        &mut self,
        mode: Mode,
        buffer_kind: Option<BufferKind>,
        key_sequence: Vec<(KeyCode, KeyModifiers)>,
        action: Action,
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
    pub fn handle_input(&mut self, current_mode: &Mode, key_event: KeyEvent) -> Option<Action> {
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
        };

        // If the keybinding exists, it's sent back
        // If not it checks if the current key sequence exists in any existing
        // keybinding and stores the current key sequence
        if action.is_some() {
            self.current_sequence.keys.clear();
            action
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

            None
        }
    }

    fn handle_normal_mode(&self, current_mode: &Mode) -> Option<Action> {
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

    fn handle_insert_mode(&self, current_mode: &Mode, key_binding: Keybinding) -> Option<Action> {
        match key_binding.key {
            KeyCode::Char(c) => Some(Action::InsertChar(c)),
            KeyCode::Tab => Some(Action::InsertTab),
            KeyCode::Backspace => Some(Action::DeleteChar),
            KeyCode::Enter => Some(Action::NewLine(NewLineDirection::Under)),
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

    fn handle_visual_mode(&self, current_mode: &Mode) -> Option<Action> {
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

    fn handle_command_mode(&self, current_mode: &Mode, key_binding: Keybinding) -> Option<Action> {
        match key_binding.key {
            KeyCode::Char(c) => Some(Action::InsertChar(c)),
            KeyCode::Backspace => Some(Action::DeleteChar),
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
    pub fn parse(input: &str) -> Vec<Action> {
        if COMMANDS.contains(&input) {
            match input {
                "wq" => vec![Action::WriteBuffer, Action::Quit],
                "w" => vec![Action::WriteBuffer],
                "q" => vec![Action::Quit],
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        }
    }
}
