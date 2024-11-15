use ratatui::crossterm::event::{self, Event};

use std::cell::RefCell;

use oxide::editor::Editor;
use oxide::keybinding::KeybindingManager;
use oxide::buffer::{Buffer, BufferState, ContentSource};
use oxide::utils::logging::setup_logger;
use oxide::OxideError;

type Result<T> = std::result::Result<T, oxide::OxideError>;

fn main() -> Result<()> {
    match setup_logger() {
        Ok(_) => {},
        Err(e) => return Err(oxide::OxideError::new(oxide::ErrorKind::UtilsError(e))),
    };

    // Initializes core components
    let terminal = ratatui::init();
    let editor = RefCell::new(Editor::new(terminal));
    let tokio_runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(e) => return Err(OxideError::new(oxide::ErrorKind::ExternError(e))),
    };
    let keybinding_manager = RefCell::new(KeybindingManager::new());
    let terminal_height = editor.borrow().renderer.get_terminal_size().height as usize;

    // Test file (change to the directory of your choice)
    let file_path   = "/home/dexter/Personal/Programming/Rust/oxide/test.txt";
    let file_buffer = match tokio_runtime.block_on(Buffer::from_file(file_path, terminal_height)) {
        Ok(buffer) => buffer,
        Err(e) => {
            eprintln!("ERROR: {}", e);

            return Err(oxide::OxideError::new(oxide::ErrorKind::BufferError(e)));
        }
    };
    editor.borrow_mut().add_buffer(file_buffer);

    // A buffer that lists the currently open buffers
    let mut buffer_names: Vec<String> = editor
        .borrow()
        .buffers
        .iter()
        .map(|buffer| buffer.borrow().title.to_string())
        .collect();
    buffer_names.push("*Buffers*".to_string());
    let buffers_buffer = Buffer::new(
        "*Buffers*".to_string(),
        buffer_names,
        ContentSource::NoSource,
        terminal_height,
        BufferState::locked(),
    );

    editor.borrow_mut().add_buffer(buffers_buffer);
    editor.borrow_mut().active_buffer = 1;

    // Main loop
    while editor.borrow().is_running {
        // Renders the buffer and makes sure the keybinding manager has the correct mode set
        match editor.borrow_mut().render() {
            Ok(_) => {},
            Err(e) => return Err(e),
        };
        keybinding_manager
            .borrow_mut()
            .set_mode(editor.borrow().get_active_buffer().mode);

        // Checks the user keypresses
        match event::read() {
            Ok(event) => match event {
                Event::Key(key_event) => {
                    let input_result = keybinding_manager
                        .borrow_mut()
                        .handle_input(key_event);

                    if let Some(action) = input_result {
                        match editor.borrow_mut().parse_action(action, &keybinding_manager, &tokio_runtime) {
                            Ok(_) => {},
                            Err(e) => return Err(e),
                        }
                    }
                },
                _ => {},
            }
            Err(_) => {},
        }
    }

    // Restores the terminal to the correct mode
    ratatui::restore();

    Ok(())
}

