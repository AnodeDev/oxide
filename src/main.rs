use ratatui::crossterm::event::{self, Event};

use std::cell::RefCell;

use oxide::editor::Editor;
use oxide::keybinding::KeybindingManager;
use oxide::buffer::{Buffer, ContentSource};
use oxide::utils::logging::setup_logger;

type Result<'a, T> = std::result::Result<T, oxide::Error<'a>>;

fn main() -> Result<'static, ()> {
    match setup_logger() {
        Ok(_) => {},
        Err(_) => {},
    };

    // Initializes core components
    let terminal = ratatui::init();
    let editor = RefCell::new(Editor::new(terminal));
    let tokio_runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(_) => todo!(),
    };
    let keybinding_manager = RefCell::new(KeybindingManager::new());
    let terminal_height = editor.borrow().renderer.get_terminal_size().height as usize;

    // Test file (change to the directory of your choice)
    let file_path   = "/home/dexter/Personal/Programming/Rust/oxide/test.txt";
    let file_buffer = match tokio_runtime.block_on(Buffer::from_file(file_path, terminal_height)) {
        Ok(buffer) => buffer,
        Err(error) => {
            eprintln!("ERROR: {}", error);

            return Err(oxide::Error::BufferError(error));
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
        match editor.borrow_mut().render() {
            Ok(_) => {},
            Err(_) => {},
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
                            Err(_) => {},
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

