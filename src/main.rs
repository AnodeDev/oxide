use ratatui::crossterm::event::{self, Event};

use oxide::editor::Editor;
use oxide::keybinding::KeybindingManager;
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
    setup_logger()?;

    // Initializes core components
    let terminal = ratatui::init();
    let mut editor = Editor::new(terminal)?;
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

                    if let Some(action) = keybinding_manager.handle_input(buffer_mode, key_event) {
                        action.execute(&mut editor)?;
                    }
                }
                _ => {}
            },
            Err(e) => eprintln!("{}", e),
        }
    }

    // Restores the terminal to the correct mode
    ratatui::restore();

    Ok(())
}
