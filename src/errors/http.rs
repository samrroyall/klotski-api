use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::{error, fmt};

use crate::errors::{board::Error as BoardError, handler::Error as HandlerError};
use crate::repositories::boards::Error as BoardsRepositoryError;

#[derive(Debug)]
pub enum Error {
    Forbidden(String),
    NotFound(String),
    BadRequest(String),
    Unhandled(String),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Forbidden(ref msg) => write!(f, "Forbidden: {msg}"),
            Error::NotFound(ref msg) => write!(f, "Not found: {msg}"),
            Error::BadRequest(ref msg) => write!(f, "Invalid input: {msg}"),
            Error::Unhandled(ref msg) => write!(f, "Internal server error: {msg}"),
        }
    }
}

impl From<BoardError> for Error {
    fn from(err: BoardError) -> Self {
        match err {
            BoardError::BlockIndexOutOfBounds
            | BoardError::BlockInvalid
            | BoardError::BlockPlacementInvalid => Error::BadRequest(err.to_string()),
            BoardError::BoardStateInvalid | BoardError::NoMovesToUndo => {
                Error::Forbidden(err.to_string())
            }
            BoardError::BoardNotFound => Error::NotFound(err.to_string()),
        }
    }
}

impl From<BoardsRepositoryError> for Error {
    fn from(err: BoardsRepositoryError) -> Self {
        match err {
            BoardsRepositoryError::BoardError(err) => {
                tracing::error!("BoardError: {}", err);
                Error::from(err)
            }
            BoardsRepositoryError::DieselError(err) => {
                tracing::error!("DieselError: {}", err);
                Error::Unhandled(err.to_string())
            }
        }
    }
}

impl From<HandlerError> for Error {
    fn from(err: HandlerError) -> Self {
        match err {
            HandlerError::InvalidBody | HandlerError::InvalidPath => {
                tracing::error!("HandlerError: {}", err);
                Error::BadRequest(err.to_string())
            }
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Error::Forbidden(_) => StatusCode::FORBIDDEN,
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::BadRequest(_) => StatusCode::BAD_REQUEST,
            Error::Unhandled(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}
