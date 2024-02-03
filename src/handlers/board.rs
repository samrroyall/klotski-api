use axum::{
    extract::{Json, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response as AxumResponse},
    Extension, Json as JsonResponse,
};

use crate::errors::{domain::BoardError, http::HttpError};
use crate::models::{api::request::*, api::response::*, domain::game::Board};
use crate::repositories::board_states::*;
use crate::services::db::DbPool;

fn handle_repository_error(e: BoardStateRepositoryError) -> HttpError {
    match e {
        BoardStateRepositoryError::BoardError(e) => match e {
            BoardError::BlockIndexOutOfBounds | BoardError::BlockPlacementInvalid => {
                HttpError::BadRequest(e.to_string())
            }
            BoardError::BoardNotFound => HttpError::NotFound(e.to_string()),
        },
        BoardStateRepositoryError::DieselError(e) => HttpError::InternalServerError(e.to_string()),
    }
}

pub async fn new_board(axum::extract::Extension(pool): Extension<DbPool>) -> AxumResponse {
    create_board_state(pool)
        .map(|board_state| {
            (
                StatusCode::OK,
                JsonResponse(BuildingResponse::from(&board_state)),
            )
        })
        .map_err(handle_repository_error)
        .into_response()
}

pub async fn get_board(
    Extension(pool): Extension<DbPool>,
    Query(params): Query<BoardQueryParams>,
) -> AxumResponse {
    get_board_state(&params.id, pool)
        .map(|board_state| {
            (
                StatusCode::OK,
                JsonResponse(BuildingResponse::from(&board_state)),
            )
        })
        .map_err(handle_repository_error)
        .into_response()
}

pub async fn delete_board(
    Extension(pool): Extension<DbPool>,
    Query(params): Query<BoardQueryParams>,
) -> AxumResponse {
    delete_board_state(&params.id, pool)
        .map(|_| (StatusCode::OK, ()))
        .map_err(handle_repository_error)
        .into_response()
}

pub async fn add_block(
    Extension(pool): Extension<DbPool>,
    Query(params): Query<BoardQueryParams>,
    Json(payload): Json<AddBlockRequest>,
) -> AxumResponse {
    let update_fn = |board: &mut Board| board.add_block(payload.positioned_block);

    update_board_state(&params.id, update_fn, pool)
        .map(|board_state| {
            (
                StatusCode::OK,
                JsonResponse(BuildingResponse::from(&board_state)),
            )
        })
        .map_err(handle_repository_error)
        .into_response()
}

pub async fn alter_block(
    Extension(pool): Extension<DbPool>,
    Path(block_idx): Path<usize>,
    Query(params): Query<BoardQueryParams>,
    Json(payload): Json<AlterBlockRequest>,
) -> AxumResponse {
    let update_fn = |board: &mut Board| match payload.action {
        AlterBlockAction::ChangeBlock => {
            board.change_block(block_idx, payload.change_data.unwrap().block_id)
        }
        AlterBlockAction::MoveBlock => {
            let move_data = payload.move_data.unwrap();
            board.move_block(block_idx, move_data.row_diff, move_data.col_diff)
        }
    };

    update_board_state(&params.id, update_fn, pool)
        .map(|board_state| {
            (
                StatusCode::OK,
                JsonResponse(BuildingResponse::from(&board_state)),
            )
        })
        .map_err(handle_repository_error)
        .into_response()
}

pub async fn remove_block(
    Extension(pool): Extension<DbPool>,
    Path(block_idx): Path<usize>,
    Query(params): Query<BoardQueryParams>,
) -> AxumResponse {
    let update_fn = |board: &mut Board| board.remove_block(block_idx);

    update_board_state(&params.id, update_fn, pool)
        .map(|board_state| {
            (
                StatusCode::OK,
                JsonResponse(BuildingResponse::from(&board_state)),
            )
        })
        .map_err(handle_repository_error)
        .into_response()
}
