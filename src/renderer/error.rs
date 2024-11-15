use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    DrawError,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::DrawError => write!(f, "DrawError"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
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
        write!(f, "{}: {}", self.kind, self.msg)
    }
}
