use ratatui::crossterm::event::{self, Event};

use anyhow;

use std::cell::RefCell;

use oxide::editor::Editor;
use oxide::keybinding::{Action, KeybindingManager, CommandParser};
use oxide::buffer::{Buffer, Manipulation, ContentSource, Mode};
use oxide::utils::logging::setup_logger;

fn main() -> anyhow::Result<()> {
    setup_logger()?;

    let terminal = ratatui::init();
    let editor = RefCell::new(Editor::new(terminal)?);
    let tokio_runtime = tokio::runtime::Runtime::new()?;
    let keybinding_manager = RefCell::new(KeybindingManager::new());
    let terminal_height = editor.borrow().renderer.get_terminal_size()?.height as usize;

    // Test file (change to the directory of your choice)
    let file_path = "/home/dexter/Personal/Programming/Rust/oxide/test.txt";
    let file_buffer = tokio_runtime.block_on(Buffer::from_file(file_path, terminal_height))?;
    editor.borrow_mut().add_buffer(file_buffer);

    // A buffer that lists the currently open buffers
    let mut buffer_names: Vec<String> = editor.borrow().buffers.iter().map(|buffer| buffer.borrow().title.to_string()).collect();
    buffer_names.push("*Buffers*".to_string());
    let buffers_buffer = Buffer::new(
        "*Buffers*",
        buffer_names,
        ContentSource::None,
        terminal_height,
        false,
        false,
    );

    editor.borrow_mut().add_buffer(buffers_buffer);
    editor.borrow_mut().active_buffer = 1;


    loop {
        editor.borrow_mut().render()?;
        keybinding_manager.borrow_mut().set_mode(editor.borrow().get_active_buffer().mode);

        if let Event::Key(key_event) = event::read()? {
            let input_result = keybinding_manager.borrow_mut().handle_input(key_event);

            if let Some(action) = input_result {
                parse_action(action, &editor, &keybinding_manager, &tokio_runtime)?;
            }
        }

        if !editor.borrow().is_running {
            break;
        }
    }

    ratatui::restore();

    Ok(())
}


fn parse_action(action: Action, editor: &RefCell<Editor>, keybinding_manager: &RefCell<KeybindingManager>, tokio_runtime: &tokio::runtime::Runtime) -> anyhow::Result<()> {
    match action {
        Action::SwitchMode(mode) => {
            editor.borrow().get_active_buffer_mut().switch_mode(mode);
        },
        Action::InsertChar(c) => {
            editor.borrow().get_active_buffer_mut().add_char(c);
        },
        Action::NewLine(direction) => editor.borrow().get_active_buffer_mut().new_line(direction),
        Action::DeleteChar => editor.borrow().get_active_buffer_mut().remove_char(),
        Action::DeleteLine => editor.borrow().get_active_buffer_mut().delete_line(),
        Action::MoveCursor(x, y) => editor.borrow().get_active_buffer_mut().move_cursor(x, y),
        Action::TopOfBuffer => editor.borrow().get_active_buffer_mut().move_cursor_to_top(),
        Action::EndOfBuffer => editor.borrow().get_active_buffer_mut().move_cursor_to_bot(),
        Action::Quit => editor.borrow_mut().is_running = false,
        Action::WriteBuffer => tokio_runtime.block_on(editor.borrow().get_active_buffer_mut().write_buffer())?,
        Action::ExecuteCommand => {
            editor.borrow_mut().get_active_buffer_mut().mode = Mode::Normal;

            let input: String = editor.borrow().get_active_buffer_mut().get_command();
            let commands = CommandParser::parse(input);

            for command in commands {
                parse_action(command, editor, keybinding_manager, tokio_runtime)?;
            }
        },
        Action::FindFile => {
            todo!("Implement Find File functionality")
        },
        _ => {},
    }

    Ok(())
}
