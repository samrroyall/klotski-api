use axum::{
    debug_handler,
    extract::{Json, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json as JsonResponse,
};

use super::utils::{
    handle_board_state_repository_error as handle_repository_error, handle_json_rejection,
    handle_path_rejection, handle_query_rejection,
};
use crate::errors::game::BoardError;
use crate::models::{
    api::request::*,
    api::response::*,
    game::{block::PositionedBlock, board::Board},
};
use crate::repositories::board_states::*;
use crate::services::db::DbPool;

#[debug_handler]
pub async fn new_board(Extension(pool): Extension<DbPool>) -> Response {
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

#[debug_handler]
pub async fn get_board(
    Extension(pool): Extension<DbPool>,
    query_extraction: Option<Query<BoardQueryParams>>,
) -> Response {
    if query_extraction.is_none() {
        return handle_query_rejection().into_response();
    }

    let query_params = query_extraction.unwrap().0;

    get_board_state(&query_params.id, pool)
        .map(|board_state| {
            (
                StatusCode::OK,
                JsonResponse(BuildingResponse::from(&board_state)),
            )
        })
        .map_err(handle_repository_error)
        .into_response()
}

#[debug_handler]
pub async fn delete_board(
    Extension(pool): Extension<DbPool>,
    query_extraction: Option<Query<BoardQueryParams>>,
) -> Response {
    if query_extraction.is_none() {
        return handle_query_rejection().into_response();
    }

    let query_params = query_extraction.unwrap().0;

    delete_board_state(&query_params.id, pool)
        .map(|_| (StatusCode::OK, ()))
        .map_err(handle_repository_error)
        .into_response()
}

#[debug_handler]
pub async fn add_block(
    Extension(pool): Extension<DbPool>,
    query_extraction: Option<Query<BoardQueryParams>>,
    json_extraction: Option<Json<AddBlockRequest>>,
) -> Response {
    if query_extraction.is_none() {
        return handle_query_rejection().into_response();
    }

    let query_params = query_extraction.unwrap().0;

    if json_extraction.is_none() {
        return handle_json_rejection().into_response();
    }

    let AddBlockRequest {
        block_id,
        min_row,
        min_col,
    } = json_extraction.unwrap().0;

    let update_fn = |board: &mut Board| {
        if let Some(block) = PositionedBlock::new(block_id, min_row, min_col) {
            return board.add_block(block);
        }
        Err(BoardError::BlockInvalid)
    };

    update_board_state(&query_params.id, update_fn, pool)
        .map(|board_state| {
            (
                StatusCode::OK,
                JsonResponse(BuildingResponse::from(&board_state)),
            )
        })
        .map_err(handle_repository_error)
        .into_response()
}

#[debug_handler]
pub async fn alter_block(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<usize>>,
    query_extraction: Option<Query<BoardQueryParams>>,
    json_extraction: Option<Json<AlterBlockRequest>>,
) -> Response {
    if path_extraction.is_none() {
        return handle_path_rejection().into_response();
    }

    let block_idx = path_extraction.unwrap().0;

    if query_extraction.is_none() {
        return handle_query_rejection().into_response();
    }

    let query_params = query_extraction.unwrap().0;

    if json_extraction.is_none() {
        return handle_json_rejection().into_response();
    }

    let payload = json_extraction.unwrap().0;

    let update_fn = |board: &mut Board| match payload {
        AlterBlockRequest::ChangeBlock(change_block_data) => {
            board.change_block(block_idx, change_block_data.new_block_id)
        }
        AlterBlockRequest::MoveBlock(move_block_data) => board.move_block(
            block_idx,
            move_block_data.row_diff,
            move_block_data.col_diff,
        ),
    };

    update_board_state(&query_params.id, update_fn, pool)
        .map(|board_state| {
            (
                StatusCode::OK,
                JsonResponse(BuildingResponse::from(&board_state)),
            )
        })
        .map_err(handle_repository_error)
        .into_response()
}

#[debug_handler]
pub async fn remove_block(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<usize>>,
    query_extraction: Option<Query<BoardQueryParams>>,
) -> Response {
    if path_extraction.is_none() {
        return handle_path_rejection().into_response();
    }

    let block_idx = path_extraction.unwrap().0;

    if query_extraction.is_none() {
        return handle_query_rejection().into_response();
    }

    let query_params = query_extraction.unwrap().0;

    let update_fn = |board: &mut Board| board.remove_block(block_idx);

    update_board_state(&query_params.id, update_fn, pool)
        .map(|board_state| {
            (
                StatusCode::OK,
                JsonResponse(BuildingResponse::from(&board_state)),
            )
        })
        .map_err(handle_repository_error)
        .into_response()
}
