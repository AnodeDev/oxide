use ratatui::crossterm::event::{self, Event, KeyCode};

use anyhow;

use oxide::editor::Editor;
use oxide::buffer::Buffer;
use oxide::keybinding::{Keybindings, Key};

fn main() -> anyhow::Result<()> {


    let terminal = ratatui::init();
    let mut editor = Editor::new(terminal);
    let mut keybindings = Keybindings::new();

    loop {
        editor.renderer.render(&editor.buffers[editor.active_buffer].borrow());

        if matches!(event::read()?, Event::Key(_)) {
            break;
        }
    }

    ratatui::restore();

    Ok(())
}
