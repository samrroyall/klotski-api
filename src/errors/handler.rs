use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    InvalidBody,
    InvalidPath,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidBody => write!(f, "Invalid JSON payload"),
            Error::InvalidPath => write!(f, "Invalid path parameters"),
        }
    }
}
