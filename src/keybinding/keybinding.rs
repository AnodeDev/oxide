use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use std::collections::HashMap;

use crate::buffer::{CommandLineState, Mode};

/// Defines all the available actions
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Action {
    Nop,
    SwitchMode(ModeParams),
    InsertChar(char),
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
    FindFile,
    AppendSelected,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ModeParams {
    Normal{
        mode: Mode,
    },
    Insert {
        mode: Mode, 
        insert_direction: InsertDirection,
    },
    Visual{
        mode: Mode,
    },
    Command {
        mode: Mode,
        prefix: String,
        input: String,
        state: CommandLineState,
    },
}

/// Defines where a new line can go
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

/// Stores the users currently pressed keys
#[derive(PartialEq, Eq, Hash)]
pub struct KeySequence {
    pub keys: Vec<Keybinding>,
}

/// Stores the key information for ease of access
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Keybinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
}

/// Stores all available keybindings as well as the currently pressed one
pub struct KeybindingManager {
    mode_bindings: HashMap<Mode, HashMap<KeySequence, Action>>,
    current_mode: Mode,
    current_sequence: KeySequence,
}

/// Handles parsing the command line commands
pub struct CommandParser;

impl KeybindingManager {
    pub fn new() -> Self {
        let mut manager = KeybindingManager {
            mode_bindings: HashMap::new(),
            current_mode: Mode::Normal,
            current_sequence: KeySequence { keys: Vec::new() },
        };

        manager.setup_default_bindings();
        manager
    }

    /// Defines all default keybindings
    fn setup_default_bindings(&mut self) {
        // NORMAL MODE
        self.add_binding(
            Mode::Normal,
            vec![ (KeyCode::Char('n'), KeyModifiers::NONE) ],
            Action::MoveCursor(-1, 0));

        self.add_binding(
            Mode::Normal,
            vec![ (KeyCode::Char('e'), KeyModifiers::NONE) ],
            Action::MoveCursor(0, 1));

        self.add_binding(
            Mode::Normal,
            vec![ (KeyCode::Char('i'), KeyModifiers::NONE) ],
            Action::MoveCursor(0, -1));

        self.add_binding(
            Mode::Normal,
            vec![ (KeyCode::Char('o'), KeyModifiers::NONE) ],
            Action::MoveCursor(1, 0));

        self.add_binding(
            Mode::Normal,
            vec![ (KeyCode::Char('s'), KeyModifiers::NONE) ],
            Action::SwitchMode(ModeParams::Insert {
                mode: Mode::Insert,
                insert_direction: InsertDirection::Before,
            }));

        self.add_binding(
            Mode::Normal,
            vec![ (KeyCode::Char('a'), KeyModifiers::NONE) ],
            Action::SwitchMode(ModeParams::Insert {
                mode: Mode::Insert,
                insert_direction: InsertDirection::After,
            }));

        self.add_binding(
            Mode::Normal,
            vec![ (KeyCode::Char('x'), KeyModifiers::NONE) ],
            Action::DeleteChar);

        self.add_binding(
            Mode::Normal,
            vec![
                (KeyCode::Char('d'), KeyModifiers::NONE),
                (KeyCode::Char('d'), KeyModifiers::NONE)
            ],
            Action::DeleteLine);

        self.add_binding(
            Mode::Normal,
            vec![
                (KeyCode::Char('g'), KeyModifiers::NONE),
                (KeyCode::Char('g'), KeyModifiers::NONE)
            ],
            Action::TopOfBuffer);

        self.add_binding(
            Mode::Normal,
            vec![ (KeyCode::Char('G'), KeyModifiers::SHIFT) ],
            Action::EndOfBuffer);

        self.add_binding(
            Mode::Normal,
            vec![
                (KeyCode::Char(' '), KeyModifiers::NONE),
                (KeyCode::Char('f'), KeyModifiers::NONE),
                (KeyCode::Char('f'), KeyModifiers::NONE)
            ],
            Action::FindFile);

        self.add_binding(
            Mode::Normal,
            vec![ (KeyCode::Char('f'), KeyModifiers::NONE) ],
            Action::NewLine(NewLineDirection::Under));

        self.add_binding(
            Mode::Normal,
            vec![ (KeyCode::Char('F'), KeyModifiers::SHIFT) ],
            Action::NewLine(NewLineDirection::Over));

        self.add_binding(
            Mode::Normal,
            vec![ (KeyCode::Char(':'), KeyModifiers::NONE) ],
            Action::SwitchMode(ModeParams::Command {
                mode: Mode::Command, 
                prefix: ":".to_string(),
                input: String::new(),
                state: CommandLineState::Default,
            }));

        self.add_binding(
            Mode::Normal,
            vec![ (KeyCode::Char('v'), KeyModifiers::NONE) ],
            Action::SwitchMode(ModeParams::Visual { mode: Mode::Visual }));

        // INSERT MODE
        self.add_binding(
            Mode::Insert,
            vec![ (KeyCode::Esc, KeyModifiers::NONE) ],
            Action::SwitchMode(ModeParams::Normal { mode: Mode::Normal, }));

        self.add_binding(
            Mode::Insert,
            vec![ (KeyCode::Enter, KeyModifiers::NONE) ],
            Action::NewLine(NewLineDirection::Under));

        // VISUAL MODE
        self.add_binding(
            Mode::Visual,
            vec![ (KeyCode::Char('n'), KeyModifiers::NONE) ],
            Action::MoveCursor(-1, 0));

        self.add_binding(
            Mode::Visual,
            vec![ (KeyCode::Char('e'), KeyModifiers::NONE) ],
            Action::MoveCursor(0, 1));

        self.add_binding(
            Mode::Visual,
            vec![ (KeyCode::Char('i'), KeyModifiers::NONE) ],
            Action::MoveCursor(0, -1));

        self.add_binding(
            Mode::Visual,
            vec![ (KeyCode::Char('o'), KeyModifiers::NONE) ],
            Action::MoveCursor(1, 0));

        self.add_binding(
            Mode::Visual,
            vec![ (KeyCode::Char('d'), KeyModifiers::NONE) ],
            Action::DeleteChar);

        self.add_binding(
            Mode::Visual,
            vec![ (KeyCode::Esc, KeyModifiers::NONE) ],
            Action::SwitchMode(ModeParams::Normal { mode: Mode::Normal, }));

        // COMMAND MODE
        self.add_binding(
            Mode::Command,
            vec![ (KeyCode::Esc, KeyModifiers::NONE) ],
            Action::SwitchMode(ModeParams::Normal { mode: Mode::Normal, }));

        self.add_binding(
            Mode::Command,
            vec![ (KeyCode::Enter, KeyModifiers::NONE) ],
            Action::ExecuteCommand);

        self.add_binding(
            Mode::Command,
            vec![ (KeyCode::Left, KeyModifiers::NONE) ],
            Action::MoveCursor(-1, 0));

        self.add_binding(
            Mode::Command,
            vec![ (KeyCode::Right, KeyModifiers::NONE) ],
            Action::MoveCursor(1, 0));

        self.add_binding(
            Mode::Command,
            vec![ (KeyCode::Up, KeyModifiers::NONE) ],
            Action::MoveCursor(0, -1));

        self.add_binding(
            Mode::Command,
            vec![ (KeyCode::Down, KeyModifiers::NONE) ],
            Action::MoveCursor(0, 1));

        self.add_binding(
            Mode::Command,
            vec![ (KeyCode::Tab, KeyModifiers::NONE) ],
            Action::AppendSelected);
    }

    /// Adds keybindings to the keybinding manager
    pub fn add_binding(&mut self, mode: Mode, key_sequence: Vec<(KeyCode, KeyModifiers)>, action: Action) {
        // Parses the key sequence
        let sequence = KeySequence {
            keys: key_sequence.into_iter()
                .map(|(key, modifiers)| Keybinding { key, modifiers })
                .collect()
        };

        // Creates a new entry
        self.mode_bindings
            .entry(mode)
            .or_insert_with(HashMap::new)
            .insert(sequence, action);
    }

    /// Checks the mode of the keybinding and the current buffer mode and redirects to the
    /// appropriate parser
    pub fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        let key_binding = Keybinding {
            key: key_event.code,
            modifiers: key_event.modifiers,
        };

        self.current_sequence.keys.push(key_binding);

        let action = match self.current_mode {
            Mode::Normal  => self.handle_normal_mode(),
            Mode::Insert  => self.handle_insert_mode(key_binding),
            Mode::Visual  => self.handle_visual_mode(),
            Mode::Command => self.handle_command_mode(key_binding),
        };


        // If the keybinding exists, it's sent back
        // If not it checks if the current key sequence exists in any existing
        // keybinding and stores the current key sequence
        if action.is_some() {
            self.current_sequence.keys.clear();
            action
        } else {
            let is_prefix = self.mode_bindings
                .get(&self.current_mode)
                .map(|bindings| {
                    bindings.keys().any(|seq| seq.keys.starts_with(&self.current_sequence.keys))
                })
                .unwrap_or(false);

            if !is_prefix {
                self.current_sequence.keys.clear();
            }

            None
        }
    }

    fn handle_normal_mode(&self) -> Option<Action> {
        self.mode_bindings
            .get(&self.current_mode)
            .and_then(|bindings| bindings.get(&self.current_sequence).cloned())
    }

    fn handle_insert_mode(&self, key_binding: Keybinding) -> Option<Action> {
        match key_binding.key {
            KeyCode::Char(c)   => Some(Action::InsertChar(c)),
            KeyCode::Backspace => Some(Action::DeleteChar),
            KeyCode::Enter     => Some(Action::NewLine(NewLineDirection::Under)),
            _ => self.mode_bindings.get(&Mode::Insert).and_then(|bindings| bindings.get(&self.current_sequence).cloned()),
        }
    }

    fn handle_visual_mode(&self) -> Option<Action> {
        self.mode_bindings
            .get(&self.current_mode)
            .and_then(|bindings| bindings.get(&self.current_sequence).cloned())
    }

    fn handle_command_mode(&self, key_binding: Keybinding) -> Option<Action> {
        match key_binding.key {
            KeyCode::Char(c)   => Some(Action::InsertChar(c)),
            KeyCode::Backspace => Some(Action::DeleteChar),
            _                  => self.mode_bindings
                                      .get(&Mode::Command)
                                      .and_then(|bindings| bindings.get(&self.current_sequence).cloned()),
        }
    }

    /// Sets the keybinding manager's mode
    pub fn set_mode(&mut self, mode: Mode) {
        self.current_mode = mode;
    }

    /// Returns the keybinding manager's mode
    pub fn get_current_mode(&self) -> &Mode {
        &self.current_mode
    }
}

impl CommandParser {
    pub fn parse(input: String, state: CommandLineState) -> Vec<Action> {
        match state {
            CommandLineState::Default => {
                input.chars()
                    .map(|c| match c {
                        'w' => Action::WriteBuffer,
                        'q' => Action::Quit,
                        _   => Action::Nop,
                    })
                    .collect()
            },
            CommandLineState::FindFile => {
                vec![ Action::OpenFile(input) ]
            },
            _ => {
                vec![ ]
            },
        }
    }
}
