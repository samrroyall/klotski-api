use axum::{
    debug_handler,
    extract::{Json, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json as JsonResponse,
};

use super::utils::*;
use crate::errors::game::BoardError;
use crate::models::{
    api::request::*,
    api::response::*,
    game::{block::PositionedBlock, board::Board},
};
use crate::repositories::board_states::*;
use crate::services::db::DbPool;

fn update_board_while_building<F>(board_id: &String, update_fn: F, pool: DbPool) -> Response
where
    F: FnOnce(&mut Board) -> Result<(), BoardError>,
{
    update_board_state_building(board_id, update_fn, pool)
        .map(|board_state| {
            (
                StatusCode::OK,
                JsonResponse(BuildingResponse::new(&board_state)),
            )
        })
        .map_err(handle_board_state_repository_error)
        .into_response()
}

fn update_board_while_solving<F>(board_id: &String, update_fn: F, pool: DbPool) -> Response
where
    F: FnOnce(&mut Board) -> Result<(), BoardError>,
{
    update_board_state_solving(board_id, update_fn, pool)
        .map(|(board_state, next_moves)| {
            (
                StatusCode::OK,
                JsonResponse(SolvingResponse::new(&board_state, &next_moves)),
            )
        })
        .map_err(handle_board_state_repository_error)
        .into_response()
}

#[debug_handler]
pub async fn new_board(Extension(pool): Extension<DbPool>) -> Response {
    create_board_state(pool)
        .map(|board_state| {
            (
                StatusCode::OK,
                JsonResponse(BuildingResponse::new(&board_state)),
            )
        })
        .map_err(handle_board_state_repository_error)
        .into_response()
}

#[debug_handler]
pub async fn get_board(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<BoardParams>>,
) -> Response {
    if path_extraction.is_none() {
        return handle_path_rejection().into_response();
    }

    let BoardParams { board_id } = path_extraction.unwrap().0;

    get_board_state(&board_id, pool)
        .map(|board_state| {
            (
                StatusCode::OK,
                JsonResponse(BuildingResponse::new(&board_state)),
            )
        })
        .map_err(handle_board_state_repository_error)
        .into_response()
}

#[debug_handler]
pub async fn delete_board(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<BoardParams>>,
) -> Response {
    if path_extraction.is_none() {
        return handle_path_rejection().into_response();
    }

    let BoardParams { board_id } = path_extraction.unwrap().0;

    delete_board_state(&board_id, pool)
        .map(|_| (StatusCode::OK, ()))
        .map_err(handle_board_state_repository_error)
        .into_response()
}

#[debug_handler]
pub async fn add_block(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<BoardParams>>,
    json_extraction: Option<Json<AddBlockRequest>>,
) -> Response {
    if path_extraction.is_none() {
        return handle_path_rejection().into_response();
    }

    let BoardParams { board_id } = path_extraction.unwrap().0;

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

    update_board_while_building(&board_id, update_fn, pool)
}

fn change_block(board_id: &String, pool: DbPool, block_idx: usize, new_block_id: u8) -> Response {
    let update_fn = |board: &mut Board| board.change_block(block_idx, new_block_id);

    update_board_while_building(board_id, update_fn, pool)
}

fn move_block(
    board_id: &String,
    pool: DbPool,
    block_idx: usize,
    row_diff: i8,
    col_diff: i8,
) -> Response {
    let update_fn = |board: &mut Board| {
        if !board.is_ready_to_solve() {
            return Err(BoardError::BoardNotReady);
        }
        board.move_block(block_idx, row_diff, col_diff)
    };

    update_board_while_solving(board_id, update_fn, pool)
}

#[debug_handler]
pub async fn alter_block(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<BlockParams>>,
    json_extraction: Option<Json<AlterBlockRequest>>,
) -> Response {
    if path_extraction.is_none() {
        return handle_path_rejection().into_response();
    }

    let BlockParams {
        board_id,
        block_idx,
    } = path_extraction.unwrap().0;

    if json_extraction.is_none() {
        return handle_json_rejection().into_response();
    }

    let payload = json_extraction.unwrap().0;

    match payload {
        AlterBlockRequest::ChangeBlock(change_block_data) => {
            change_block(&board_id, pool, block_idx, change_block_data.new_block_id)
        }
        AlterBlockRequest::MoveBlock(move_block_data) => move_block(
            &board_id,
            pool,
            block_idx,
            move_block_data.row_diff,
            move_block_data.col_diff,
        ),
    }
}

#[debug_handler]
pub async fn undo_move(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<BoardParams>>,
) -> Response {
    if path_extraction.is_none() {
        return handle_path_rejection().into_response();
    }

    let BoardParams { board_id } = path_extraction.unwrap().0;

    let update_fn = |board: &mut Board| {
        if board.is_ready_to_solve() {
            return Err(BoardError::BoardNotReady);
        }
        board.undo_move()
    };

    update_board_while_solving(&board_id, update_fn, pool)
}

#[debug_handler]
pub async fn remove_block(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<BlockParams>>,
) -> Response {
    if path_extraction.is_none() {
        return handle_path_rejection().into_response();
    }

    let BlockParams {
        board_id,
        block_idx,
    } = path_extraction.unwrap().0;

    let update_fn = |board: &mut Board| board.remove_block(block_idx);

    update_board_while_building(&board_id, update_fn, pool)
}
