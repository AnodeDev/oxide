use ratatui::crossterm::event::{self, Event};

use anyhow;

use std::cell::RefCell;

use oxide::editor::Editor;
use oxide::keybinding::{Action, KeybindingManager, CommandParser};
use oxide::buffer::{Buffer, Manipulation, ContentSource, Mode};
use oxide::utils::logging::setup_logger;

fn main() -> anyhow::Result<()> {
    setup_logger()?;

    // Initializes core components
    let terminal = ratatui::init();
    let editor = RefCell::new(Editor::new(terminal)?);
    let tokio_runtime = tokio::runtime::Runtime::new()?;
    let keybinding_manager = RefCell::new(KeybindingManager::new());
    let terminal_height = editor.borrow().renderer.get_terminal_size()?.height as usize;

    // Test file (change to the directory of your choice)
    let file_path   = "/home/dexter/Personal/Programming/Rust/oxide/test.txt";
    let file_buffer = tokio_runtime.block_on(Buffer::from_file(file_path, terminal_height))?;
    editor.borrow_mut().add_buffer(file_buffer);

    // A buffer that lists the currently open buffers
    let mut buffer_names: Vec<String> = editor
        .borrow()
        .buffers
        .iter()
        .map(|buffer| buffer.borrow().title.to_string())
        .collect();
    buffer_names.push("*Buffers*".to_string());
    let buffers_buffer = Buffer::new("*Buffers*",
                                     buffer_names,
                                     ContentSource::NoSource,
                                     terminal_height,
                                     false,
                                     false);

    editor.borrow_mut().add_buffer(buffers_buffer);
    editor.borrow_mut().active_buffer = 1;


    // Main loop
    while editor.borrow().is_running {
        // Renders the buffer and makes sure the keybinding manager has the correct mode set
        editor.borrow_mut().render()?;
        keybinding_manager
            .borrow_mut()
            .set_mode(editor.borrow().get_active_buffer().mode);

        // Checks the user keypresses
        if let Event::Key(key_event) = event::read()? {
            let input_result = keybinding_manager
                .borrow_mut()
                .handle_input(key_event);

            if let Some(action) = input_result {
                parse_action(action, &editor, &keybinding_manager, &tokio_runtime)?;
            }
        }
    }

    // Restores the terminal to the correct mode
    ratatui::restore();

    Ok(())
}


/// Parses the keybinding and executes the corresponding action
fn parse_action(action: Action, editor: &RefCell<Editor>, keybinding_manager: &RefCell<KeybindingManager>, tokio_runtime: &tokio::runtime::Runtime) -> anyhow::Result<()> {
    let editor_ref = editor.borrow_mut();
    let mut buffer = editor_ref.get_active_buffer_mut();

    match action {
        Action::SwitchMode(mode)   => buffer.switch_mode(mode),
        Action::InsertChar(c)      => buffer.add_char(c),
        Action::NewLine(direction) => buffer.new_line(direction),
        Action::DeleteChar         => buffer.remove_char(),
        Action::DeleteLine         => buffer.delete_line(),
        Action::MoveCursor(x, y)   => buffer.move_cursor(x, y),
        Action::TopOfBuffer        => buffer.move_cursor_to_top(),
        Action::EndOfBuffer        => buffer.move_cursor_to_bot(),
        Action::Quit               => editor.borrow_mut().is_running = false,
        Action::WriteBuffer        => tokio_runtime.block_on(buffer.write_buffer())?,
        Action::ExecuteCommand     => {
            buffer.mode = Mode::Normal;

            let input: String = buffer.get_command();
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
