use axum::http::StatusCode;

use std::fmt;

#[derive(Debug)]
pub enum Error {
    NotAllowed(String),
    NotFound(String),
    BadRequest(String),
    Unhandled(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NotAllowed(ref msg) => write!(f, "Not allowed: {msg}"),
            Error::NotFound(ref msg) => write!(f, "Not found: {msg}"),
            Error::BadRequest(ref msg) => write!(f, "Invalid input: {msg}"),
            Error::Unhandled(ref msg) => write!(f, "Internal server error: {msg}"),
        }
    }
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Error::NotAllowed(_) => StatusCode::METHOD_NOT_ALLOWED,
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::BadRequest(_) => StatusCode::BAD_REQUEST,
            Error::Unhandled(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}
