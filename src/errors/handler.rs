use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Body,
    Path,
    Query,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Body => write!(f, "Invalid JSON payload"),
            Error::Path => write!(f, "Invalid path parameters"),
            Error::Query => write!(f, "Invalid query parameters"),
        }
    }
}
