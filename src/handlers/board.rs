use axum::{
    debug_handler,
    extract::{Json, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json as JsonResponse,
};

use super::utils::{
    handle_board_error, handle_board_state_repository_error, handle_json_rejection,
    handle_path_rejection,
};
use crate::errors::board::Error as BoardError;
use crate::models::{
    api::request::{
        AddBlock as AddBlockRequest, AlterBlock as AlterBlockRequest,
        AlterBoard as AlterBoardRequest, BlockParams, BoardParams,
    },
    api::response::{Board as BoardResponse, Solve as SolveResponse, Solved as SolvedResponse},
    game::{blocks::Positioned as PositionedBlock, board::Board},
};
use crate::repositories::board_states::{
    create as create_board_state, delete as delete_board_state, get as get_board_state,
    update_while_building as update_board_state_while_building,
    update_while_solving as update_board_state_while_solving,
};
use crate::services::{db::Pool as DbPool, solver::Solver};

#[debug_handler]
pub async fn new(Extension(pool): Extension<DbPool>) -> Response {
    create_board_state(&pool)
        .map(|board_state| {
            (
                StatusCode::OK,
                JsonResponse(BoardResponse::new(&board_state)),
            )
        })
        .map_err(handle_board_state_repository_error)
        .into_response()
}

#[debug_handler]
pub async fn alter(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<BoardParams>>,
    json_extraction: Option<Json<AlterBoardRequest>>,
) -> Response {
    if path_extraction.is_none() {
        return handle_path_rejection().into_response();
    }

    if json_extraction.is_none() {
        return handle_json_rejection().into_response();
    }

    let params = path_extraction.unwrap().0;
    let body = json_extraction.unwrap().0;

    match body {
        AlterBoardRequest::ChangeBoardState(data) => make_building_update(
            params.board_id,
            |board: &mut Board| board.change_state(data.new_state),
            &pool,
        ),
        AlterBoardRequest::UndoMove => make_solving_update(
            params.board_id,
            |board: &mut Board| board.undo_move(),
            &pool,
        ),
    }
}

#[debug_handler]
pub async fn solve(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<BoardParams>>,
) -> Response {
    if path_extraction.is_none() {
        return handle_path_rejection().into_response();
    }

    let params = path_extraction.unwrap().0;

    get_board_state(params.board_id, &pool)
        .map_err(handle_board_state_repository_error)
        .and_then(|board_state| {
            Solver::new(&mut board_state.to_board()).map_err(handle_board_error)
        })
        .map(|mut solver| {
            (
                StatusCode::OK,
                JsonResponse(match solver.solve() {
                    Some(moves) => SolveResponse::Solved(SolvedResponse::new(&moves)),
                    None => SolveResponse::UnableToSolve,
                }),
            )
        })
        .into_response()
}

#[debug_handler]
pub async fn delete(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<BoardParams>>,
) -> Response {
    if path_extraction.is_none() {
        return handle_path_rejection().into_response();
    }

    let params = path_extraction.unwrap().0;

    delete_board_state(params.board_id, &pool)
        .map(|()| (StatusCode::OK, ()))
        .map_err(handle_board_state_repository_error)
        .into_response()
}

fn make_building_update<F>(board_id: i32, update_fn: F, pool: &DbPool) -> Response
where
    F: FnOnce(&mut Board) -> Result<(), BoardError>,
{
    update_board_state_while_building(board_id, update_fn, pool)
        .map(|board_state| {
            (
                StatusCode::OK,
                JsonResponse(BoardResponse::new(&board_state)),
            )
        })
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

    if json_extraction.is_none() {
        return handle_json_rejection().into_response();
    }

    let params = path_extraction.unwrap().0;
    let body = json_extraction.unwrap().0;

    let update_fn = |board: &mut Board| {
        if let Some(block) = PositionedBlock::new(body.block_id, body.min_row, body.min_col) {
            return board.add_block(block);
        }
        Err(BoardError::BlockInvalid)
    };

    make_building_update(params.board_id, update_fn, &pool)
}

fn make_solving_update<F>(board_id: i32, update_fn: F, pool: &DbPool) -> Response
where
    F: FnOnce(&mut Board) -> Result<(), BoardError>,
{
    update_board_state_while_solving(board_id, update_fn, pool)
        .map(|(board_state, next_moves)| {
            (
                StatusCode::OK,
                JsonResponse(BoardResponse::new_with_next_moves(
                    &board_state,
                    &next_moves,
                )),
            )
        })
        .map_err(handle_board_state_repository_error)
        .into_response()
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

    if json_extraction.is_none() {
        return handle_json_rejection().into_response();
    }

    let params = path_extraction.unwrap().0;
    let body = json_extraction.unwrap().0;

    match body {
        AlterBlockRequest::ChangeBlock(data) => make_building_update(
            params.board_id,
            |board: &mut Board| board.change_block(params.block_idx, data.new_block_id),
            &pool,
        ),
        AlterBlockRequest::MoveBlock(data) => make_solving_update(
            params.board_id,
            |board: &mut Board| board.move_block(params.block_idx, data.steps.as_slice()),
            &pool,
        ),
    }
}

#[debug_handler]
pub async fn remove_block(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<BlockParams>>,
) -> Response {
    if path_extraction.is_none() {
        return handle_path_rejection().into_response();
    }

    let params = path_extraction.unwrap().0;

    let update_fn = |board: &mut Board| board.remove_block(params.block_idx);

    make_building_update(params.board_id, update_fn, &pool)
}
