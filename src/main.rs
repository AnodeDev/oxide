use ratatui::crossterm::event::{self, Event};

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
    // Enable if you want logging
    // setup_logger()?;

    // Initializes core components
    let terminal = ratatui::init();
    let mut editor = Editor::new(terminal);
    let tokio_runtime = tokio::runtime::Runtime::new()?;
    let mut keybinding_manager = KeybindingManager::new();

    // Main loop
    while editor.is_running {
        // Renders the buffer
        editor.render()?;

        // Checks the user keypresses
        match event::read() {
            Ok(event) => match event {
                Event::Key(key_event) => {
                    let buffer_mode = &editor.buffer_manager.get_active_buffer()?.mode;
                    let input_result = keybinding_manager.handle_input(buffer_mode, key_event);

                    if let Some(action) = input_result {
                        match editor.parse_action(action, &keybinding_manager, &tokio_runtime) {
                            Ok(_) => {}
                            Err(_) => {
                                editor
                                    .buffer_manager
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
