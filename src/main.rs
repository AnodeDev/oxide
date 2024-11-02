use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use anyhow;

use std::cell::RefCell;
use std::rc::Rc;
use std::any::Any;

use oxide::editor::Editor;
use oxide::keybinding::{Action, KeybindingContext, KeybindingRegistry, KeyCombination};
use oxide::buffer::{Buffer, Manipulation};
use oxide::utils::logging::setup_logger;

fn main() -> anyhow::Result<()> {
    setup_logger()?;

    let terminal = ratatui::init();
    let editor = RefCell::new(Editor::new(terminal));
    let mut keybinds_registry = KeybindingRegistry::new();
    let tokio_runtime = tokio::runtime::Runtime::new()?;


    keybinds_registry.register_keybinding(
        vec![ KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE) ],
        KeybindingContext::Buffer,
        Action {
            id: "cusor_left",
            function: Rc::new(|target: &mut dyn Any| {
                if let Some(buffer) = target.downcast_mut::<Buffer>() {
                    buffer.cursor_left();
                }
            }),
            description: "Move the cursor left",
        }
    );

    keybinds_registry.register_keybinding(
        vec![ KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE) ],
        KeybindingContext::Buffer,
        Action {
            id: "cusor_down",
            function: Rc::new(|target: &mut dyn Any| {
                if let Some(buffer) = target.downcast_mut::<Buffer>() {
                    buffer.cursor_down();
                }
            }),
            description: "Move the cursor down",
        }
    );

    keybinds_registry.register_keybinding(
        vec![ KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE) ],
        KeybindingContext::Buffer,
        Action {
            id: "cusor_up",
            function: Rc::new(|target: &mut dyn Any| {
                if let Some(buffer) = target.downcast_mut::<Buffer>() {
                    buffer.cursor_up();
                }
            }),
            description: "Move the cursor up",
        }
    );

    keybinds_registry.register_keybinding(
        vec![ KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE) ],
        KeybindingContext::Buffer,
        Action {
            id: "cusor_right",
            function: Rc::new(|target: &mut dyn Any| {
                if let Some(buffer) = target.downcast_mut::<Buffer>() {
                    buffer.cursor_right();
                }
            }),
            description: "Move the cursor right",
        }
    );

    keybinds_registry.register_keybinding(
        vec![ KeyEvent::new(KeyCode::Char('e'), KeyModifiers::SHIFT) ],
        KeybindingContext::Buffer,
        Action {
            id: "half_down",
            function: Rc::new(|target: &mut dyn Any| {
                if let Some(buffer) = target.downcast_mut::<Buffer>() {
                    buffer.cursor_half_down();
                }
            }),
            description: "Move cursor half way down the screen",
        }
    );

    keybinds_registry.register_keybinding(
        vec![ KeyEvent::new(KeyCode::Char('i'), KeyModifiers::SHIFT) ],
        KeybindingContext::Buffer,
        Action {
            id: "half_up",
            function: Rc::new(|target: &mut dyn Any| {
                if let Some(buffer) = target.downcast_mut::<Buffer>() {
                    buffer.cursor_half_up();
                }
            }),
            description: "Move cursor half way up the screen",
        }
    );

    keybinds_registry.register_keybinding(
        vec![ KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE), KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE) ],
        KeybindingContext::Global,
        Action {
            id: "quit",
            function: Rc::new(|target: &mut dyn Any| {
                if let Some(editor) = target.downcast_mut::<Editor>() {
                    editor.should_quit = true;
                }
            }),
            description: "Quit Oxide",
        }
    );

    let file_path = "/home/dexter/.zshrc";
    let file_buffer = tokio_runtime.block_on(Buffer::from_file(file_path))?;
    editor.borrow_mut().add_buffer(file_buffer);
    editor.borrow_mut().active_buffer = 1;

    loop {
        editor.borrow_mut().render()?;

        let keys: KeyCombination = tokio_runtime.block_on(keybinds_registry.read_keys())?;
        keybinds_registry.process_key_event(keys, &mut editor.borrow_mut());

        if editor.borrow().should_quit {
            break;
        }
    }

    ratatui::restore();

    Ok(())
}
