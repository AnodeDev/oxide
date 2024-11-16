use std::fmt;

#[derive(Debug)]
pub enum Error {
    DrawError,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DrawError => write!(f, "DrawError: Failed to draw to screen"),
        }
    }
}
