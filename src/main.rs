use oxide::editor::Editor;
use oxide::utils::logging::setup_logger;
use std::env;
use std::path::{Path, PathBuf};

// ╭──────────────────────────────────────╮
// │ Types                                │
// ╰──────────────────────────────────────╯

type Result<T> = std::result::Result<T, oxide::OxideError>;

// ╭──────────────────────────────────────╮
// │ Main                                 │
// ╰──────────────────────────────────────╯

fn main() -> Result<()> {
    // Uncomment for logging
    setup_logger()?;

    // Check for args
    let mut args = env::args();
    args.next();
    let path: Option<PathBuf> = match args.next() {
        Some(path_str) => {
            let mut result: Option<PathBuf> = None;

            if !Path::new(&path_str).exists() {
                let mut path = PathBuf::from(std::env::current_dir()?);
                path.push(path_str);
                path.pop();

                if path.exists() {
                    result = Some(path);
                }
            } else {
                result = Some(PathBuf::from(&path_str));
            }

            result
        },
        None => None,
    };

    let runtime = tokio::runtime::Runtime::new()?;

    // Initializes core components
    let terminal = ratatui::init();
    let mut editor = Editor::new(terminal, path, runtime)?;

    editor.main_loop()?;

    // Restores the terminal to the correct mode
    ratatui::restore();

    Ok(())
}
