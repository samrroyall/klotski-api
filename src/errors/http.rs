use axum::http::StatusCode;

use std::fmt;

#[derive(Debug)]
pub enum HttpError {
    NotFound(String),
    BadRequest(String),
    InternalServerError(String),
}

impl std::error::Error for HttpError {}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpError::NotFound(ref msg) => write!(f, "Not found: {}", msg),
            HttpError::BadRequest(ref msg) => write!(f, "Invalid input: {}", msg),
            HttpError::InternalServerError(ref msg) => write!(f, "Internal server error: {}", msg),
        }
    }
}

impl axum::response::IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            HttpError::NotFound(_) => StatusCode::NOT_FOUND,
            HttpError::BadRequest(_) => StatusCode::BAD_REQUEST,
            HttpError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}
