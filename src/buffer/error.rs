use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    WriteToSourceError,
    FileNotFoundError,
    WrongModeError,
    VisualModeInitError,
    ConvertToPathError,
    ReadDirectoryError,
    ExternError,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::WriteToSourceError => write!(f, "WriteToSourceError"),
            ErrorKind::FileNotFoundError => write!(f, "FileNotFoundError"),
            ErrorKind::WrongModeError => write!(f, "WrongModeError"),
            ErrorKind::VisualModeInitError => write!(f, "VisualModeInitError"),
            ErrorKind::ConvertToPathError => write!(f, "ConvertToPathError"),
            ErrorKind::ReadDirectoryError => write!(f, "ReadDirectoryError"),
            ErrorKind::ExternError => write!(f, "ExternError"),
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
