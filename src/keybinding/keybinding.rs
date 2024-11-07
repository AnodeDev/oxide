use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use std::collections::HashMap;

use crate::buffer::Mode;

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Action {
    Nop,
    SwitchMode(Mode),
    InsertChar(char),
    NewLine(NewLineDirection),
    DeleteChar(DeleteDirection),
    DeleteLine,
    MoveCursor(i32, i32),
    Quit,
    WriteBuffer,
    ExecuteCommand,
    FindFile,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum DeleteDirection {
    Behind,
    Under,
    Ahead,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum NewLineDirection {
    Under,
    Over,
}

#[derive(PartialEq, Eq, Hash)]
pub struct KeySequence {
    pub keys: Vec<Keybinding>,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Keybinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
}

pub struct KeybindingManager {
    mode_bindings: HashMap<Mode, HashMap<KeySequence, Action>>,
    current_mode: Mode,
    current_sequence: KeySequence,
}

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

    fn setup_default_bindings(&mut self) {
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
            Action::SwitchMode(Mode::Insert));

        self.add_binding(
            Mode::Normal,
            vec![ (KeyCode::Char('x'), KeyModifiers::NONE) ],
            Action::DeleteChar(DeleteDirection::Under));

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
            Action::SwitchMode(Mode::Command));

        self.add_binding(
            Mode::Insert,
            vec![ (KeyCode::Esc, KeyModifiers::NONE) ],
            Action::SwitchMode(Mode::Normal));

        self.add_binding(
            Mode::Insert,
            vec![ (KeyCode::Enter, KeyModifiers::NONE) ],
            Action::NewLine(NewLineDirection::Under));

        self.add_binding(
            Mode::Command,
            vec![ (KeyCode::Esc, KeyModifiers::NONE) ],
            Action::SwitchMode(Mode::Normal));

        self.add_binding(
            Mode::Command,
            vec![ (KeyCode::Enter, KeyModifiers::NONE) ],
            Action::ExecuteCommand);

    }

    pub fn add_binding(&mut self, mode: Mode, key_sequence: Vec<(KeyCode, KeyModifiers)>, action: Action) {
        let sequence = KeySequence {
            keys: key_sequence.into_iter()
                .map(|(key, modifiers)| Keybinding { key, modifiers })
                .collect()
        };

        self.mode_bindings
            .entry(mode)
            .or_insert_with(HashMap::new)
            .insert(sequence, action);
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        let key_binding = Keybinding {
            key: key_event.code,
            modifiers: key_event.modifiers,
        };

        self.current_sequence.keys.push(key_binding);

        let action = match self.current_mode {
            Mode::Normal => self.handle_normal_mode(),
            Mode::Insert => self.handle_insert_mode(key_binding),
            Mode::Command => self.handle_command_mode(key_binding),
        };


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
            KeyCode::Backspace => Some(Action::DeleteChar(DeleteDirection::Behind)),
            KeyCode::Enter     => Some(Action::NewLine(NewLineDirection::Under)),
            _ => self.mode_bindings.get(&Mode::Insert).and_then(|bindings| bindings.get(&self.current_sequence).cloned()),
        }
    }

    fn handle_command_mode(&self, key_binding: Keybinding) -> Option<Action> {
        match key_binding.key {
            KeyCode::Char(c) => Some(Action::InsertChar(c)),
            KeyCode::Backspace => Some(Action::DeleteChar(DeleteDirection::Behind)),
            _ => self.mode_bindings.get(&Mode::Command).and_then(|bindings| bindings.get(&self.current_sequence).cloned()),
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.current_mode = mode;
    }

    pub fn get_current_mode(&self) -> &Mode {
        &self.current_mode
    }
}

impl CommandParser {
    pub fn parse(input: String) -> Vec<Action> {
        input.chars()
            .map(|c| match c {
                'w' => Action::WriteBuffer,
                'q' => Action::Quit,
                _   => Action::Nop,
            })
            .collect()
    }
}
