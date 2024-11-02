use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};

use anyhow;

use std::cell::{RefCell, RefMut};

use oxide::editor::Editor;
use oxide::keybinding::{Action, KeybindingContext, KeybindingRegistry};
use oxide::buffer::{Buffer, Manipulation};
use oxide::utils::logging::setup_logger;

fn main() -> anyhow::Result<()> {
    setup_logger()?;

    let terminal = ratatui::init();
    let editor = RefCell::new(Editor::new(terminal));
    let mut buffer_registry = KeybindingRegistry::<Buffer>::new();
    let mut quit = false;

    buffer_registry.register_keybinding(
        KeybindingContext::Buffer,
        vec![ KeyCode::Char('n') ],
        Action {
            id: "cusor_left",
            function: Box::new(|mut buffer: RefMut<Buffer>| buffer.cursor_left()),
            description: "Move the cursor left",
        }
    );

    buffer_registry.register_keybinding(
        KeybindingContext::Buffer,
        vec![ KeyCode::Char('e') ],
        Action {
            id: "cusor_down",
            function: Box::new(|mut buffer: RefMut<Buffer>| buffer.cursor_down()),
            description: "Move the cursor down",
        }
    );

    buffer_registry.register_keybinding(
        KeybindingContext::Buffer,
        vec![ KeyCode::Char('i') ],
        Action {
            id: "cusor_up",
            function: Box::new(|mut buffer: RefMut<Buffer>| buffer.cursor_up()),
            description: "Move the cursor up",
        }
    );

    buffer_registry.register_keybinding(
        KeybindingContext::Buffer,
        vec![ KeyCode::Char('o') ],
        Action {
            id: "cusor_right",
            function: Box::new(|mut buffer: RefMut<Buffer>| buffer.cursor_right()),
            description: "Move the cursor right",
        }
    );

    loop {
        editor.borrow_mut().render()?;

        if quit {
            break;
        }
    }

    ratatui::restore();

    Ok(())
}
