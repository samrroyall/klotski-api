use crate::{
    errors::{game::BoardError, http::HttpError},
    repositories::board_states::BoardStateRepositoryError,
};

pub fn handle_json_rejection() -> HttpError {
    HttpError::BadRequest("Invalid JSON payload".to_string())
}

pub fn handle_path_rejection() -> HttpError {
    HttpError::BadRequest("Invalid path parameters".to_string())
}

pub fn handle_query_rejection() -> HttpError {
    HttpError::BadRequest("Invalid query parameters".to_string())
}

pub fn handle_board_state_repository_error(e: BoardStateRepositoryError) -> HttpError {
    match e {
        BoardStateRepositoryError::BoardError(e) => match e {
            BoardError::BlockIndexOutOfBounds
            | BoardError::BlockInvalid
            | BoardError::BlockPlacementInvalid
            | BoardError::NoMovesToUndo => HttpError::BadRequest(e.to_string()),
            BoardError::BoardNotFound => HttpError::NotFound(e.to_string()),
        },
        BoardStateRepositoryError::DieselError(e) => HttpError::InternalServerError(e.to_string()),
    }
}
