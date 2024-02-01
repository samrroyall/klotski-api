use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};

use crate::models::{
    api::{
        request::{AddBlockRequest, AlterBlockAction, AlterBlockRequest, QueryParams},
        response::BuildingResponse,
    },
    domain::game::Board,
};
use crate::repositories::board_states::{
    create_board_state, delete_board_state, get_board_state, update_board_state,
};
use crate::services::db::DbPool;

#[debug_handler]
pub async fn new_board(
    Extension(pool): Extension<DbPool>,
) -> (StatusCode, Result<Json<BuildingResponse>, String>) {
    match create_board_state(pool) {
        Ok(board_state) => (
            StatusCode::OK,
            Ok(Json(BuildingResponse::from(&board_state))),
        ),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Err(e.to_string())),
    }
}

#[debug_handler]
pub async fn get_board(
    Extension(pool): Extension<DbPool>,
    Query(params): Query<QueryParams>,
) -> (StatusCode, Result<Json<BuildingResponse>, String>) {
    match get_board_state(&params.id, pool) {
        Ok(board_state) => (
            StatusCode::OK,
            Ok(Json(BuildingResponse::from(&board_state))),
        ),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Err(e.to_string())),
    }
}

#[debug_handler]
pub async fn delete_board(
    Extension(pool): Extension<DbPool>,
    Query(params): Query<QueryParams>,
) -> (StatusCode, Result<(), String>) {
    match delete_board_state(&params.id, pool) {
        Ok(_) => (StatusCode::OK, Ok(())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Err(e.to_string())),
    }
}

#[debug_handler]
pub async fn add_block(
    Extension(pool): Extension<DbPool>,
    Query(params): Query<QueryParams>,
    Json(payload): Json<AddBlockRequest>,
) -> (StatusCode, Result<Json<BuildingResponse>, String>) {
    let update_fn = |board: &mut Board| board.add_block(payload.positioned_block);

    match update_board_state(&params.id, update_fn, pool) {
        Ok(board_state) => (
            StatusCode::OK,
            Ok(Json(BuildingResponse::from(&board_state))),
        ),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Err(e.to_string())),
    }
}

#[debug_handler]
pub async fn alter_block(
    Extension(pool): Extension<DbPool>,
    Path(block_idx): Path<usize>,
    Query(params): Query<QueryParams>,
    Json(payload): Json<AlterBlockRequest>,
) -> (StatusCode, Result<Json<BuildingResponse>, String>) {
    let update_fn = |board: &mut Board| match payload.action {
        AlterBlockAction::ChangeBlock => {
            let change_data = payload.change_data.unwrap();
            board.change_block(block_idx, change_data.block_id);
        }
        AlterBlockAction::MoveBlock => {
            let move_data = payload.move_data.unwrap();
            board.move_block(block_idx, move_data.row_diff, move_data.col_diff);
        }
    };

    match update_board_state(&params.id, update_fn, pool) {
        Ok(board_state) => (
            StatusCode::OK,
            Ok(Json(BuildingResponse::from(&board_state))),
        ),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Err(e.to_string())),
    }
}

#[debug_handler]
pub async fn remove_block(
    Extension(pool): Extension<DbPool>,
    Path(block_idx): Path<usize>,
    Query(params): Query<QueryParams>,
) -> (StatusCode, Result<Json<BuildingResponse>, String>) {
    let update_fn = |board: &mut Board| board.remove_block(block_idx);

    match update_board_state(&params.id, update_fn, pool) {
        Ok(board_state) => (
            StatusCode::OK,
            Ok(Json(BuildingResponse::from(&board_state))),
        ),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Err(e.to_string())),
    }
}
