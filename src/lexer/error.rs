use std::fmt;

#[derive(Debug)]
pub enum Error {
    RegexError(regex::Error),
    TomlError(toml::de::Error),
    IoError(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Self {
        Error::TomlError(error)
    }
}

impl From<regex::Error> for Error {
    fn from(error: regex::Error) -> Self {
        Error::RegexError(error)
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::RegexError(e) => write!(f, "{}", e),
            Error::TomlError(e) => write!(f, "{}", e),
            Error::IoError(e) => write!(f, "{}", e),
        }
    }
}
