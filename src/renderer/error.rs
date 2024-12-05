use std::fmt;

// ╭──────────────────────────────────────╮
// │ Error Types                          │
// ╰──────────────────────────────────────╯

#[derive(Debug)]
pub enum Error {
    DrawError,
    WrongModeError,
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
// TODO: Add custom error messages.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DrawError => write!(f, "DrawError: Failed to draw to screen"),
            Error::WrongModeError => write!(f, "WrongModeError: Editor is in the wrong mode"),
            Error::IoError(e) => write!(f, "{}", e),
        }
    }
}
