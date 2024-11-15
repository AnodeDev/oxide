use crate::buffer;
use crate::utils;

use std::fmt;

#[derive(Debug)]
pub enum ErrorKind {
    BufferError(buffer::Error),
    UtilsError(utils::Error),
    ExternError(std::io::Error),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::BufferError(e) => write!(f, "ERROR: {}", e),
            ErrorKind::UtilsError(e)  => write!(f, "ERROR: {}", e),
            ErrorKind::ExternError(e) => write!(f, "ERROR: {}", e),
        }
    }
}

#[derive(Debug)]
pub struct OxideError {
    kind: ErrorKind,
}

impl OxideError {
    pub fn new(kind: ErrorKind) -> Self {
        OxideError { kind }
    }
}

impl fmt::Display for OxideError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::BufferError(e) => write!(f, "KIND: {}, MESSAGE: {}", self.kind, format!("{}", e)),
            ErrorKind::UtilsError(e)  => write!(f, "KIND: {}, MESSAGE: {}", self.kind, format!("{}", e)),
            ErrorKind::ExternError(e) => write!(f, "KIND: {}, MESSAGE: {}", self.kind, format!("{}", e)),
        }
    }
}
