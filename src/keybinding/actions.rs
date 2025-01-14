use std::path::PathBuf;

use crate::buffer::{Buffer, Manipulation, Minibuffer, MinibufferKind, Navigation};
use crate::editor::Editor;
use crate::keybinding::CommandParser;
use crate::OxideError;

// ╭──────────────────────────────────────╮
// │ Keybinding Type                      │
// ╰──────────────────────────────────────╯

type Result<T> = std::result::Result<T, OxideError>;

// ╭──────────────────────────────────────╮
// │ Keybinding Trait                     │
// ╰──────────────────────────────────────╯

pub trait Action: Send + Sync {
    fn execute(&self, editor: &mut Editor) -> Result<()>;
}

// ╭──────────────────────────────────────╮
// │ Keybinding Enums                     │
// ╰──────────────────────────────────────╯

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ModeParams {
    Normal,
    Insert { insert_direction: InsertDirection },
    Visual,
    Command { prefix: String },
    Minibuffer,
}

// Defines where a new line can go
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum NewLineDirection {
    Under,
    Over,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum InsertDirection {
    Beginning,
    Before,
    After,
    End,
}

// ╭──────────────────────────────────────╮
// │ Keybinding Actions                   │
// ╰──────────────────────────────────────╯

#[derive(Clone)]
pub struct MoveCursorAction {
    x: i32,
    y: i32,
}

impl MoveCursorAction {
    pub fn new(x: i32, y: i32) -> Self {
        MoveCursorAction { x, y }
    }
}

impl Action for MoveCursorAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor
            .buffer_manager
            .get_active_buffer_mut()?
            .move_cursor(self.x, self.y);

        Ok(())
    }
}

#[derive(Clone)]
pub struct SwitchModeAction {
    params: ModeParams,
}

impl SwitchModeAction {
    pub fn new(params: ModeParams) -> Self {
        SwitchModeAction { params }
    }
}

impl Action for SwitchModeAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor
            .buffer_manager
            .get_active_buffer_mut()?
            .switch_mode(self.params.clone());

        Ok(())
    }
}

#[derive(Clone)]
pub struct AddCharAction {
    character: char,
}

impl AddCharAction {
    pub fn new(character: char) -> Self {
        AddCharAction { character }
    }
}

impl Action for AddCharAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor
            .buffer_manager
            .get_active_buffer_mut()?
            .add_char(self.character)?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct AddTabAction;

impl Action for AddTabAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor.buffer_manager.get_active_buffer_mut()?.add_tab()?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct NewLineAction {
    direction: NewLineDirection,
}

impl NewLineAction {
    pub fn new(direction: NewLineDirection) -> Self {
        NewLineAction { direction }
    }
}

impl Action for NewLineAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor
            .buffer_manager
            .get_active_buffer_mut()?
            .new_line(self.direction.clone());

        Ok(())
    }
}

#[derive(Clone)]
pub struct DeleteLineAction;

impl Action for DeleteLineAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor.buffer_manager.get_active_buffer_mut()?.delete_line();

        Ok(())
    }
}

pub struct TopOfBufferAction;

impl Action for TopOfBufferAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor
            .buffer_manager
            .get_active_buffer_mut()?
            .move_cursor_to_top();

        Ok(())
    }
}

#[derive(Clone)]
pub struct BotOfBufferAction;

impl Action for BotOfBufferAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor
            .buffer_manager
            .get_active_buffer_mut()?
            .move_cursor_to_bot();

        Ok(())
    }
}

pub struct QuitAction;

impl Action for QuitAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor.is_running = false;

        Ok(())
    }
}

#[derive(Clone)]
pub struct DeleteCharAction;

impl Action for DeleteCharAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor
            .buffer_manager
            .get_active_buffer_mut()?
            .remove_char()?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct WriteBufferAction;

impl Action for WriteBufferAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor.runtime.block_on(
            editor
                .buffer_manager
                .get_active_buffer_mut()?
                .write_buffer(),
        )?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct ExecuteCommandAction;

impl Action for ExecuteCommandAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        let input: &str = editor.buffer_manager.get_active_buffer_mut()?.get_command();
        let commands = CommandParser::parse(input);

        for command in commands {
            command.execute(editor)?;
        }

        editor
            .buffer_manager
            .get_active_buffer_mut()?
            .switch_mode(ModeParams::Normal);

        Ok(())
    }
}

#[derive(Clone)]
pub struct OpenFileAction {
    path: PathBuf,
}

impl OpenFileAction {
    pub fn new(path: PathBuf) -> Self {
        OpenFileAction { path }
    }
}

impl Action for OpenFileAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        let height = editor.renderer.get_terminal_size().height as usize;
        let buffer = editor
            .runtime
            .block_on(Buffer::from_file(self.path.clone(), height))?;

        editor.buffer_manager.add_buffer(buffer);
        editor.buffer_manager.active_buffer = editor.buffer_manager.buffers.len() - 1;

        Ok(())
    }
}

#[derive(Clone)]
pub struct MinibufferAction {
    kind: MinibufferKind,
}

impl MinibufferAction {
    pub fn new(kind: MinibufferKind) -> Self {
        MinibufferAction { kind }
    }
}

impl Action for MinibufferAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor
            .buffer_manager
            .get_active_buffer_mut()?
            .switch_mode(ModeParams::Minibuffer);

        match self.kind {
            MinibufferKind::Buffer(_) => {
                let mut buffers: Vec<String> = Vec::new();

                for buffer in &editor.buffer_manager.buffers {
                    buffers.push(buffer.title.clone());
                }

                editor.minibuffer.kind = MinibufferKind::Buffer(buffers);
            }
            _ => editor.minibuffer.kind = self.kind.clone(),
        }

        editor.minibuffer.fill()?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct AppendAction;

impl Action for AppendAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor.minibuffer.append()?;

        editor.minibuffer.fill()?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct OpenBufferAction {
    index: usize,
}

impl OpenBufferAction {
    pub fn new(index: usize) -> Self {
        OpenBufferAction { index }
    }
}

impl Action for OpenBufferAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        if self.index < editor.buffer_manager.buffers.len() {
            editor.buffer_manager.active_buffer = self.index;
        } else {
            return Err(OxideError::IndexError);
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct EscapeAction;

impl Action for EscapeAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor.minibuffer = Minibuffer::default();
        editor
            .buffer_manager
            .get_active_buffer_mut()?
            .switch_mode(ModeParams::Normal);

        Ok(())
    }
}

#[derive(Clone)]
pub struct AddMbCharAction {
    character: char,
}

impl AddMbCharAction {
    pub fn new(character: char) -> Self {
        AddMbCharAction { character }
    }
}

impl Action for AddMbCharAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor.minibuffer.add_char(self.character)?;

        editor.minibuffer.fill()?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct DeleteMbCharAction;

impl Action for DeleteMbCharAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor.minibuffer.remove_char()?;

        editor.minibuffer.fill()?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct ExecuteMbCommandAction;

impl Action for ExecuteMbCommandAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        match editor.minibuffer.execute()? {
            Some(action) => {
                action.execute(editor)?;

                editor.minibuffer = Minibuffer::default();
                editor
                    .buffer_manager
                    .get_active_buffer_mut()?
                    .switch_mode(ModeParams::Normal);
            }
            None => {}
        }

        editor.minibuffer.fill()?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct MoveMbCursorAction {
    x: i32,
    y: i32,
}

impl MoveMbCursorAction {
    pub fn new(x: i32, y: i32) -> Self {
        MoveMbCursorAction { x, y }
    }
}

impl Action for MoveMbCursorAction {
    fn execute(&self, editor: &mut Editor) -> Result<()> {
        editor.minibuffer.move_cursor(self.x, self.y);

        editor.minibuffer.fill()?;

        Ok(())
    }
}
