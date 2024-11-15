use std::fmt;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    LogInitError,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::LogInitError => write!(f, "LogInitError"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
    msg: String,
}

impl Error {
    pub fn new(kind: ErrorKind, msg: String) -> Self {
        Error { kind, msg }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Kind: {}, Message: {}", self.kind, self.msg)
    }
}
