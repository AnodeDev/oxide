use oxide::editor::Editor;
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
    let mut editor = Editor::new(terminal)?;

    editor.main_loop()?;

    // Restores the terminal to the correct mode
    ratatui::restore();

    Ok(())
}
