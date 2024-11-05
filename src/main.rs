use ratatui::crossterm::event::{self, Event};

use anyhow;

use std::cell::RefCell;

use oxide::editor::Editor;
use oxide::keybinding::{Action, KeybindingManager};
use oxide::buffer::{Buffer, Manipulation, ContentSource};
use oxide::utils::logging::setup_logger;

fn main() -> anyhow::Result<()> {
    setup_logger()?;

    let terminal = ratatui::init();
    let editor = RefCell::new(Editor::new(terminal));
    let tokio_runtime = tokio::runtime::Runtime::new()?;
    let mut keybinding_manager = KeybindingManager::new();

    // Test file (change to the directory of your choice)
    let file_path = "/home/dexter/Personal/Programming/Rust/oxide/test.txt";
    let file_buffer = tokio_runtime.block_on(Buffer::from_file(file_path))?;
    editor.borrow_mut().add_buffer(file_buffer);

    // A buffer that lists the currently open buffers
    let mut buffer_names: Vec<String> = editor.borrow().buffers.iter().map(|buffer| buffer.borrow().title.to_string()).collect();
    buffer_names.push("*Buffers*".to_string());
    let buffers_buffer = Buffer::new(
        "*Buffers*",
        buffer_names,
        ContentSource::None,
        false,
        false,
    );

    editor.borrow_mut().add_buffer(buffers_buffer);
    editor.borrow_mut().active_buffer = 2;

    loop {
        editor.borrow_mut().render()?;

        if let Event::Key(key_event) = event::read()? {
            if let Some(action) = keybinding_manager.handle_input(key_event) {
                match action {
                    Action::SwitchMode(mode) => {
                        editor.borrow().get_active_buffer_mut().mode = mode;
                        keybinding_manager.set_mode(mode);
                    },
                    Action::InsertChar(c) => editor.borrow().get_active_buffer_mut().add_char(c),
                    Action::NewLine => editor.borrow().get_active_buffer_mut().new_line(),
                    Action::DeleteChar => editor.borrow().get_active_buffer_mut().remove_char(),
                    Action::MoveCursor(x, y) => editor.borrow().get_active_buffer_mut().move_cursor(x, y),
                    Action::ExecuteCommand => {
                        let command = editor.borrow().get_active_buffer_mut().get_command();
                    },
                    _ => {},
                }
            }
        }
        if editor.borrow().should_quit {
            break;
        }
    }

    ratatui::restore();

    Ok(())
}
