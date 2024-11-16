use crate::buffer;
use crate::renderer;
use crate::utils;

use std::fmt;

#[derive(Debug)]
pub enum OxideError {
    BufferError(buffer::Error),
    RendererError(renderer::Error),
    UtilsError(utils::Error),
    IoError(std::io::Error),
}

impl From<std::io::Error> for OxideError {
    fn from(error: std::io::Error) -> Self {
        OxideError::IoError(error)
    }
}

impl From<buffer::Error> for OxideError {
    fn from(error: buffer::Error) -> Self {
        OxideError::BufferError(error)
    }
}

impl From<renderer::Error> for OxideError {
    fn from(error: renderer::Error) -> Self {
        OxideError::RendererError(error)
    }
}

impl From<utils::Error> for OxideError {
    fn from(error: utils::Error) -> Self {
        OxideError::UtilsError(error)
    }
}

impl std::error::Error for OxideError {}

impl fmt::Display for OxideError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OxideError::BufferError(e) => write!(f, "ERROR: {}", e),
            OxideError::RendererError(e) => write!(f, "ERROR: {}", e),
            OxideError::UtilsError(e) => write!(f, "ERROR: {}", e),
            OxideError::IoError(e) => write!(f, "ERROR: {}", e),
        }
    }
}
