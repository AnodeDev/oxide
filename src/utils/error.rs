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
pub struct Error<'a> {
    kind: ErrorKind,
    msg: &'a str,
}

impl<'a> Error<'a> {
    pub fn new(kind: ErrorKind, msg: &'a str) -> Self {
        Error { kind, msg }
    }
}

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Kind: {}, Message: {}", self.kind, self.msg)
    }
}
