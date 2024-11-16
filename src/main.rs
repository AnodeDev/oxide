use ratatui::crossterm::event::{self, Event};

use oxide::buffer::{Buffer, Mode};
use oxide::editor::Editor;
use oxide::keybinding::{KeybindingManager, ModeParams};
use oxide::utils::logging::setup_logger;

type Result<T> = std::result::Result<T, oxide::OxideError>;

fn main() -> Result<()> {
    setup_logger()?;

    // Initializes core components
    let terminal = ratatui::init();
    let mut editor = Editor::new(terminal);
    let tokio_runtime = tokio::runtime::Runtime::new()?;
    let mut keybinding_manager = KeybindingManager::new();
    let terminal_height = editor.renderer.get_terminal_size().height as usize;

    // Test file (change to the directory of your choice)
    let file_path = "/home/dexter/Personal/Programming/Rust/oxide/test.txt";
    let file_buffer = tokio_runtime.block_on(Buffer::from_file(file_path, terminal_height))?;
    editor.add_buffer(file_buffer);

    let buffer_list_buffer = Buffer::buffer_list(terminal_height);
    editor.add_buffer(buffer_list_buffer);

    editor.active_buffer = 1;

    // Main loop
    while editor.is_running {
        // Renders the buffer and makes sure the keybinding manager has the correct mode set
        match editor.render() {
            Ok(_) => {}
            Err(e) => return Err(e),
        };
        keybinding_manager.set_mode(editor.get_active_buffer().mode);
        keybinding_manager.set_buffer_kind(editor.get_active_buffer().kind);

        // Checks the user keypresses
        match event::read() {
            Ok(event) => match event {
                Event::Key(key_event) => {
                    let input_result = keybinding_manager.handle_input(key_event);

                    if let Some(action) = input_result {
                        match editor.parse_action(action, &keybinding_manager, &tokio_runtime) {
                            Ok(_) => {}
                            Err(e) => {
                                editor
                                    .get_active_buffer_mut()
                                    .switch_mode(ModeParams::Normal { mode: Mode::Normal });
                                editor
                                    .get_active_buffer_mut()
                                    .command_line
                                    .display_error(e.to_string());
                            }
                        }
                    }
                }
                _ => {}
            },
            Err(_) => {}
        }
    }

    // Restores the terminal to the correct mode
    ratatui::restore();

    Ok(())
}
