use std::fmt;

// ╭──────────────────────────────────────╮
// │ Error Types                          │
// ╰──────────────────────────────────────╯

#[derive(Debug)]
pub enum Error {
    DrawError,
}

// Allows for the use of error propagation using '?' for the custom errors.
impl std::error::Error for Error {}

// Defines the error messages for the errors.
// TODO: Add custom error messages.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DrawError => write!(f, "DrawError: Failed to draw to screen"),
        }
    }
}
