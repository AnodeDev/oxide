use ratatui::crossterm::event::{self, Event};

use std::path::Path;

use oxide::buffer::Buffer;
use oxide::editor::Editor;
use oxide::keybinding::{KeybindingManager, ModeParams};
use oxide::utils::logging::setup_logger;

// ╭──────────────────────────────────────╮
// │ Types                                │
// ╰──────────────────────────────────────╯

type Result<T> = std::result::Result<T, oxide::OxideError>;

// ╭──────────────────────────────────────╮
// │ Main                                 │
// ╰──────────────────────────────────────╯

fn main() -> Result<()> {
    setup_logger()?;

    // Initializes core components
    let terminal = ratatui::init();
    let mut editor = Editor::new(terminal);
    let tokio_runtime = tokio::runtime::Runtime::new()?;
    let mut keybinding_manager = KeybindingManager::new();
    let terminal_height = editor.renderer.get_terminal_size().height as usize;

    // Test file (change to the directory of your choice)
    let file_path = Path::new("/home/dexter/Personal/Programming/Rust/oxide/test.txt").to_path_buf();
    let file_buffer = tokio_runtime.block_on(Buffer::from_file(file_path, terminal_height))?;
    editor.add_buffer(file_buffer);

    let buffer_list_buffer = Buffer::buffer_list(terminal_height);
    editor.add_buffer(buffer_list_buffer);

    // Main loop
    while editor.is_running {
        // Renders the buffer
        editor.render()?;

        // Checks the user keypresses
        match event::read() {
            Ok(event) => match event {
                Event::Key(key_event) => {
                    let buffer_mode = &editor.get_active_buffer()?.mode;
                    let input_result = keybinding_manager.handle_input(buffer_mode, key_event);

                    if let Some(action) = input_result {
                        match editor.parse_action(action, &keybinding_manager, &tokio_runtime) {
                            Ok(_) => {}
                            Err(_) => {
                                editor
                                    .get_active_buffer_mut()?
                                    .switch_mode(ModeParams::Normal);
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
