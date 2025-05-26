use crate::editor::Editor;
use crate::OxideError;

type Result<T> = std::result::Result<T, OxideError>;

pub trait Plugin: Send + Sync {
    fn init(&mut self, editor: &mut Editor);
}
