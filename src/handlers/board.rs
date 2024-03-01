use axum::{
    debug_handler,
    extract::{Json, Path},
    response::{IntoResponse, Response},
    Extension,
};

use super::utils::{
    handle_board_error, handle_board_state_repository_error, handle_json_rejection,
    handle_path_rejection,
};
use crate::models::{
    api::request::{
        AddBlock as AddBlockRequest, AlterBlock as AlterBlockRequest,
        AlterBoard as AlterBoardRequest, BlockParams, BoardParams, NewBoard as NewBoardRequest,
    },
    api::response::{Board as BoardResponse, Solve as SolveResponse, Solved as SolvedResponse},
    game::{blocks::Positioned as PositionedBlock, board::Board},
};
use crate::repositories::board_states::{
    create as create_board_state, delete as delete_board_state, get as get_board_state,
    update as update_board_state,
};
use crate::services::{db::Pool as DbPool, solver};
use crate::{
    errors::{board::Error as BoardError, http::Error as HttpError},
    services::randomizer,
};

fn new_inner(
    Extension(pool): Extension<DbPool>,
    json_extraction: Option<Json<NewBoardRequest>>,
) -> Result<BoardResponse, HttpError> {
    let body = json_extraction.ok_or(handle_json_rejection())?.0;

    let mut board = create_board_state(&pool).map_err(handle_board_state_repository_error)?;

    if let NewBoardRequest::Random = body {
        randomizer::randomize(&mut board);
    }

    Ok(BoardResponse::new(board))
}

#[debug_handler]
pub async fn new(
    pool_extension: Extension<DbPool>,
    json_extraction: Option<Json<NewBoardRequest>>,
) -> Response {
    new_inner(pool_extension, json_extraction).into_response()
}

fn update<F>(board_id: i32, update_fn: F, pool: &DbPool) -> Result<BoardResponse, HttpError>
where
    F: FnOnce(&mut Board) -> Result<(), BoardError>,
{
    let board = update_board_state(board_id, update_fn, pool)
        .map_err(handle_board_state_repository_error)?;

    Ok(BoardResponse::new(board))
}

fn alter_inner(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<BoardParams>>,
    json_extraction: Option<Json<AlterBoardRequest>>,
) -> Result<BoardResponse, HttpError> {
    let params = path_extraction.ok_or(handle_path_rejection())?.0;
    let body = json_extraction.ok_or(handle_json_rejection())?.0;

    match body {
        AlterBoardRequest::ChangeState(data) => update(
            params.board_id,
            |board: &mut Board| board.change_state(&data.new_state),
            &pool,
        ),
        AlterBoardRequest::UndoMove => update(
            params.board_id,
            |board: &mut Board| board.undo_move(),
            &pool,
        ),
        AlterBoardRequest::Reset => {
            update(params.board_id, |board: &mut Board| board.reset(), &pool)
        }
    }
}

#[debug_handler]
pub async fn alter(
    pool_extension: Extension<DbPool>,
    path_extraction: Option<Path<BoardParams>>,
    json_extraction: Option<Json<AlterBoardRequest>>,
) -> Response {
    alter_inner(pool_extension, path_extraction, json_extraction).into_response()
}

fn solve_inner(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<BoardParams>>,
) -> Result<SolveResponse, HttpError> {
    let params = path_extraction.ok_or(handle_path_rejection())?.0;

    let board =
        get_board_state(params.board_id, &pool).map_err(handle_board_state_repository_error)?;

    let maybe_moves = solver::solve(&board).map_err(handle_board_error)?;

    match maybe_moves {
        Some(moves) => Ok(SolveResponse::Solved(SolvedResponse::new(moves))),
        None => Ok(SolveResponse::UnableToSolve),
    }
}

#[debug_handler]
pub async fn solve(
    pool_extension: Extension<DbPool>,
    path_extraction: Option<Path<BoardParams>>,
) -> Response {
    solve_inner(pool_extension, path_extraction).into_response()
}

fn delete_inner(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<BoardParams>>,
) -> Result<(), HttpError> {
    let params = path_extraction.ok_or(handle_path_rejection())?.0;

    delete_board_state(params.board_id, &pool).map_err(handle_board_state_repository_error)?;

    Ok(())
}

#[debug_handler]
pub async fn delete(
    pool_extension: Extension<DbPool>,
    path_extraction: Option<Path<BoardParams>>,
) -> Response {
    delete_inner(pool_extension, path_extraction).into_response()
}

fn add_block_inner(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<BoardParams>>,
    json_extraction: Option<Json<AddBlockRequest>>,
) -> Result<BoardResponse, HttpError> {
    let params = path_extraction.ok_or(handle_path_rejection())?.0;
    let body = json_extraction.ok_or(handle_json_rejection())?.0;

    let new_block = PositionedBlock::new(body.block, body.min_row, body.min_col)
        .ok_or(handle_board_error(BoardError::BlockInvalid))?;

    let update_fn = |board: &mut Board| board.add_block(new_block);

    update(params.board_id, update_fn, &pool)
}

#[debug_handler]
pub async fn add_block(
    pool_extension: Extension<DbPool>,
    path_extraction: Option<Path<BoardParams>>,
    json_extraction: Option<Json<AddBlockRequest>>,
) -> Response {
    add_block_inner(pool_extension, path_extraction, json_extraction).into_response()
}

fn alter_block_inner(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<BlockParams>>,
    json_extraction: Option<Json<AlterBlockRequest>>,
) -> Result<BoardResponse, HttpError> {
    let params = path_extraction.ok_or(handle_path_rejection())?.0;
    let body = json_extraction.ok_or(handle_json_rejection())?.0;

    match body {
        AlterBlockRequest::ChangeBlock(data) => update(
            params.board_id,
            |board: &mut Board| board.change_block(params.block_idx, data.new_block),
            &pool,
        ),
        AlterBlockRequest::MoveBlock(data) => update(
            params.board_id,
            |board: &mut Board| board.move_block(params.block_idx, data.row_diff, data.col_diff),
            &pool,
        ),
    }
}

#[debug_handler]
pub async fn alter_block(
    pool_extension: Extension<DbPool>,
    path_extraction: Option<Path<BlockParams>>,
    json_extraction: Option<Json<AlterBlockRequest>>,
) -> Response {
    alter_block_inner(pool_extension, path_extraction, json_extraction).into_response()
}

fn remove_block_inner(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<BlockParams>>,
) -> Result<BoardResponse, HttpError> {
    let params = path_extraction.ok_or(handle_path_rejection())?.0;

    let update_fn = |board: &mut Board| board.remove_block(params.block_idx);

    update(params.board_id, update_fn, &pool)
}

#[debug_handler]
pub async fn remove_block(
    pool_extension: Extension<DbPool>,
    path_extraction: Option<Path<BlockParams>>,
) -> Response {
    remove_block_inner(pool_extension, path_extraction).into_response()
}
