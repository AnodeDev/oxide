use std::fmt;

#[derive(Debug)]
pub enum Error {
    WriteToSourceError,
    FileNotFoundError,
    WrongModeError,
    InvalidSourceError,
    VisualModeInitError,
    ConvertToPathError,
    ReadDirectoryError,
    IoError(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::WriteToSourceError  => write!(f, "WriteToSourceError: Failed to write to source"),
            Error::FileNotFoundError   => write!(f, "FileNotFoundError: File was not found"),
            Error::WrongModeError      => write!(f, "WrongModeError: Wrong mode to execute function"),
            Error::InvalidSourceError  => write!(f, "InvalidSourceError: You're not able to perform this action from this buffer"),
            Error::VisualModeInitError => write!(f, "VisualModeInitError: Visual mode was not initialized correctly"),
            Error::ConvertToPathError  => write!(f, "ConvertToPathError: Failed to convert to path"),
            Error::ReadDirectoryError  => write!(f, "ReadDirectoryError: Failed to read directory"),
            Error::IoError(e)          => write!(f, "{}", e),
        }
    }
}
