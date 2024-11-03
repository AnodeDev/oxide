use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use anyhow;

use log::warn;

use std::cell::RefCell;
use std::rc::Rc;
use std::any::Any;

use oxide::editor::Editor;
use oxide::keybinding::{Action, KeybindingContext, KeybindingRegistry, KeyCombination};
use oxide::buffer::{Buffer, Manipulation, Mode, ContentSource};
use oxide::utils::logging::setup_logger;

fn main() -> anyhow::Result<()> {
    setup_logger()?;

    let terminal = ratatui::init();
    let editor = RefCell::new(Editor::new(terminal));
    let mut keybinds_registry = KeybindingRegistry::new();
    let tokio_runtime = tokio::runtime::Runtime::new()?;

    // Test keybindings
    keybinds_registry.register_keybinding(
        vec![ KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE) ],
        Mode::Normal,
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
        Mode::Normal,
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
        Mode::Normal,
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
        Mode::Normal,
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
        Mode::Normal,
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
        Mode::Normal,
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
        vec![ KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE) ],
        Mode::Normal,
        KeybindingContext::Buffer,
        Action {
            id: "insert_mode",
            function: Rc::new(|target: &mut dyn Any| {
                if let Some(buffer) = target.downcast_mut::<Buffer>() {
                    buffer.mode = Mode::Insert;
                }
            }),
            description: "Switch to insert mode",
        }
    );

    keybinds_registry.register_keybinding(
        vec![ KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE) ],
        Mode::Normal,
        KeybindingContext::Buffer,
        Action {
            id: "command_mode",
            function: Rc::new(|target: &mut dyn Any| {
                if let Some(buffer) = target.downcast_mut::<Buffer>() {
                    buffer.cursor.desired_x = buffer.cursor.x;
                    buffer.cursor.x = 0;
                    buffer.mode = Mode::Command;
                }
            }),
            description: "Switch to command mode",
        }
    );

    keybinds_registry.register_keybinding(
        vec![ KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE) ],
        Mode::Insert,
        KeybindingContext::Buffer,
        Action {
            id: "normal_mode",
            function: Rc::new(|target: &mut dyn Any| {
                if let Some(buffer) = target.downcast_mut::<Buffer>() {
                    buffer.mode = Mode::Normal;
                }
            }),
            description: "Switch to normal mode",
        }
    );

    keybinds_registry.register_keybinding(
        vec![ KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE) ],
        Mode::Command,
        KeybindingContext::Buffer,
        Action {
            id: "normal_mode",
            function: Rc::new(|target: &mut dyn Any| {
                if let Some(buffer) = target.downcast_mut::<Buffer>() {
                    buffer.commandline = String::new();
                    buffer.mode = Mode::Normal;
                }
            }),
            description: "Switch to normal mode",
        }
    );

    // Test file (change to the directory of your choice)
    let file_path = "/home/dexter/Personal/Programming/Rust/oxide/test.txt";
    let file_buffer = tokio_runtime.block_on(Buffer::from_file(file_path))?;
    editor.borrow_mut().add_buffer(file_buffer);

    // A buffer that lists the currently open buffers
    let mut buffer_names: Vec<String> = editor.borrow().buffers.iter().map(|buffer| buffer.borrow().title.to_string()).collect();
    buffer_names.push("*Buffers*".to_string());
    let buffers_buffer = Buffer::new(
        "*Buffers*",
        buffer_names,
        ContentSource::None,
        false,
        false,
    );

    editor.borrow_mut().add_buffer(buffers_buffer);
    editor.borrow_mut().active_buffer = 2;

    loop {
        editor.borrow_mut().render()?;

        let mode = editor.borrow().get_active_buffer().mode;

        let keys: KeyCombination = tokio_runtime.block_on(keybinds_registry.read_keys(mode))?;
        match tokio_runtime.block_on(keybinds_registry.process_key_event(keys, mode, &mut editor.borrow_mut())) {
            Ok(_) => {},
            Err(e) => {
                warn!("ERROR: {}", e);

                eprintln!("ERROR: {}", e);
                
                break;
            },
        };

        if editor.borrow().should_quit {
            break;
        }
    }

    ratatui::restore();

    Ok(())
}
