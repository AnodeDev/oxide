use std::fmt;
use std::path::PathBuf;

// ╭──────────────────────────────────────╮
// │ Error Types                          │
// ╰──────────────────────────────────────╯

#[derive(Debug)]
pub enum Error {
    WriteToSourceError {
        source: String,
        reason: String,
    },
    FileNotFoundError {
        path: PathBuf,
    },
    WrongModeError {
        current_mode: String,
        valid_modes: Vec<String>,
    },
    WrongKindError {
        expected_kind: String,
        actual_kind: String,
    },
    InvalidSourceError {
        details: String,
    },
    VisualModeInitError {
        details: String,
    },
    ConvertToPathError {
        input: String,
    },
    ReadDirectoryError {
        directory: PathBuf,
    },
    NoMatchError {
        input: String,
    },
    InvalidPathError {
        path: PathBuf,
    },
    ImmutableBufferError {
        title: String,
    },
    IoError(std::io::Error),
}

// Allows for the use of error propagation using '?' for Results that return an IO error.
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

// Allows for the use of error propagation using '?' for the custom errors.
impl std::error::Error for Error {}

// Defines the error messages for the errors.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::WriteToSourceError { source, reason } => {
                write!(f, "WriteToSourceError: Failed to write to source '{}' due to '{}'", source, reason)
            }
            Error::FileNotFoundError { path } => {
                write!(f, "FileNotFoundError: File not found at path '{}'", path.display())
            }
            Error::WrongModeError { current_mode, valid_modes } => {
                write!(f, "WrongModeError: Current mode is invalid for this action. Current mode: '{}'. Valid mode(s): '{}'", current_mode, valid_modes.join(", "))
            }
            Error::WrongKindError { expected_kind, actual_kind } => {
                write!(f, "WrongKindError: Expected kind '{}', but found '{}'", expected_kind, actual_kind)
            }
            Error::InvalidSourceError { details } => {
                write!(f, "InvalidSourceError: {}", details)
            }
            Error::VisualModeInitError { details } => {
                write!(f, "VisualModeInitError: {}", details)
            }
            Error::ConvertToPathError { input } => {
                write!(f, "ConvertToPathError: Failed to convert input '{}' to a valid path", input)
            }
            Error::ReadDirectoryError { directory } => {
                write!(f, "ReadDirectoryError: Failed to read directory '{}'", directory.display())
            }
            Error::NoMatchError { input } => {
                write!(f, "NoMatchError: No match found for input '{}'", input)
            }
            Error::InvalidPathError { path } => {
                write!(f, "InvalidPathError: Path '{}' is invalid", path.display())
            }
            Error::ImmutableBufferError { title } => {
                write!(f, "ImmutableBufferError: Current buffer '{}' is immutable and cannot be manipulated", title)
            }
            Error::IoError(e) => write!(f, "IoError: {}", e),
        }
    }
}

