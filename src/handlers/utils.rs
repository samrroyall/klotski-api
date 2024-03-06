use crate::{
    errors::{board::Error as BoardError, http::Error as HttpError},
    repositories::boards::Error as BoardStateRepositoryError,
};

pub fn handle_json_rejection() -> HttpError {
    HttpError::BadRequest("Invalid JSON payload".to_string())
}

pub fn handle_path_rejection() -> HttpError {
    HttpError::BadRequest("Invalid path parameters".to_string())
}

pub fn handle_board_error(e: BoardError) -> HttpError {
    match e {
        BoardError::BlockIndexOutOfBounds
        | BoardError::BlockInvalid
        | BoardError::BlockPlacementInvalid => HttpError::BadRequest(e.to_string()),
        BoardError::BoardStateInvalid | BoardError::NoMovesToUndo => {
            HttpError::Forbidden(e.to_string())
        }
        BoardError::BoardNotFound => HttpError::NotFound(e.to_string()),
    }
}

pub fn handle_board_state_repository_error(e: BoardStateRepositoryError) -> HttpError {
    match e {
        BoardStateRepositoryError::BoardError(e) => handle_board_error(e),
        BoardStateRepositoryError::DieselError(e) => HttpError::Unhandled(e.to_string()),
    }
}
