use std::fmt;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    WriteToSourceError,
    FileNotFoundError,
    WrongModeError,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::WriteToSourceError => write!(f, "WriteToSourceError"),
            ErrorKind::FileNotFoundError => write!(f, "FileNotFoundError"),
            ErrorKind::WrongModeError => write!(f, "WrongModeError"),
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
