use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use std::collections::HashMap;

use crate::buffer::Mode;

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Action {
    SwitchMode(Mode),
    InsertChar(char),
    NewLine,
    DeleteChar,
    MoveCursor(i32, i32),
    Quit,
    SaveBuffer,
    ExecuteCommand,
}

#[derive(PartialEq, Eq, Hash)]
pub struct Keybinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
}

pub struct KeybindingManager {
    mode_bindings: HashMap<Mode, HashMap<Keybinding, Action>>,
    current_mode: Mode,
}

impl KeybindingManager {
    pub fn new() -> Self {
        let mut manager = KeybindingManager {
            mode_bindings: HashMap::new(),
            current_mode: Mode::Normal,
        };

        manager.setup_default_bindings();
        manager
    }

    fn setup_default_bindings(&mut self) {
        self.add_binding(
            Mode::Normal,
            KeyCode::Char('n'),
            KeyModifiers::NONE,
            Action::MoveCursor(-1, 0));

        self.add_binding(
            Mode::Normal,
            KeyCode::Char('e'),
            KeyModifiers::NONE,
            Action::MoveCursor(0, 1));

        self.add_binding(
            Mode::Normal,
            KeyCode::Char('i'),
            KeyModifiers::NONE,
            Action::MoveCursor(0, -1));

        self.add_binding(
            Mode::Normal,
            KeyCode::Char('o'),
            KeyModifiers::NONE,
            Action::MoveCursor(1, 0));

        self.add_binding(
            Mode::Normal,
            KeyCode::Char('s'),
            KeyModifiers::NONE,
            Action::SwitchMode(Mode::Insert));

        self.add_binding(
            Mode::Normal,
            KeyCode::Char(':'),
            KeyModifiers::NONE,
            Action::SwitchMode(Mode::Command));

        self.add_binding(
            Mode::Insert,
            KeyCode::Esc,
            KeyModifiers::NONE,
            Action::SwitchMode(Mode::Normal));

        self.add_binding(
            Mode::Insert,
            KeyCode::Enter,
            KeyModifiers::NONE,
            Action::NewLine);

        self.add_binding(
            Mode::Command,
            KeyCode::Esc,
            KeyModifiers::NONE,
            Action::SwitchMode(Mode::Normal));

        self.add_binding(
            Mode::Command,
            KeyCode::Enter,
            KeyModifiers::NONE,
            Action::ExecuteCommand);

    }

    pub fn add_binding(&mut self, mode: Mode, key: KeyCode, modifiers: KeyModifiers, action: Action) {
        self.mode_bindings
            .entry(mode)
            .or_insert_with(HashMap::new)
            .insert(Keybinding { key, modifiers }, action);
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        let key_binding = Keybinding {
            key: key_event.code,
            modifiers: key_event.modifiers,
        };

        match self.current_mode {
            Mode::Normal => self.handle_normal_mode(key_binding),
            Mode::Insert => self.handle_insert_mode(key_binding),
            Mode::Command => self.handle_command_mode(key_binding),
        }
    }

    fn handle_insert_mode(&self, key_binding: Keybinding) -> Option<Action> {
        match key_binding.key {
            KeyCode::Char(c) => Some(Action::InsertChar(c)),
            KeyCode::Backspace => Some(Action::DeleteChar),
            KeyCode::Enter => Some(Action::NewLine),
            _ => self.mode_bindings.get(&Mode::Insert).and_then(|bindings| bindings.get(&key_binding).cloned()),
        }
    }

    fn handle_command_mode(&self, key_binding: Keybinding) -> Option<Action> {
        match key_binding.key {
            KeyCode::Char(c) => Some(Action::InsertChar(c)),
            KeyCode::Backspace => Some(Action::DeleteChar),
            _ => self.mode_bindings.get(&Mode::Command).and_then(|bindings| bindings.get(&key_binding).cloned()),
        }
    }

    fn handle_normal_mode(&self, key_binding: Keybinding) -> Option<Action> {
        self.mode_bindings
            .get(&self.current_mode)
            .and_then(|bindings| bindings.get(&key_binding).cloned())
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.current_mode = mode;
    }

    pub fn get_current_mode(&self) -> &Mode {
        &self.current_mode
    }
}
