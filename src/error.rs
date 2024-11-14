use crate::buffer;

use std::fmt;

#[derive(Debug, Clone)]
pub enum Error<'a> {
    BufferError(buffer::Error<'a>),
}

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::BufferError(error) => write!(f, "ERROR: {}", error),
        }
    }
}
