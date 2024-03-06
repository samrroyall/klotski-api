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
    api::{request, response},
    game::{blocks::Positioned as PositionedBlock, board::Board, moves::FlatBoardMove},
};
use crate::repositories::boards::{
    create as create_board_state, delete as delete_board_state, get as get_board_state,
    update as update_board_state,
};
use crate::repositories::solutions::{create as create_solution, get as get_solution};
use crate::services::{db::Pool as DbPool, solver};
use crate::{
    errors::{board::Error as BoardError, http::Error as HttpError},
    services::randomizer,
};

fn update<F>(board_id: i32, update_fn: F, pool: &DbPool) -> Result<response::Board, HttpError>
where
    F: FnOnce(&mut Board) -> Result<(), BoardError>,
{
    let board = update_board_state(board_id, update_fn, pool)
        .map_err(handle_board_state_repository_error)?;

    Ok(response::Board::new(board))
}

fn new_inner(
    Extension(pool): Extension<DbPool>,
    json_extraction: Option<Json<request::NewBoard>>,
) -> Result<response::Board, HttpError> {
    let body = json_extraction.ok_or(handle_json_rejection())?.0;

    let board = create_board_state(&pool).map_err(handle_board_state_repository_error)?;

    match body {
        request::NewBoard::Empty => Ok(response::Board::new(board)),
        request::NewBoard::Random => update(
            board.id,
            |board: &mut Board| randomizer::randomize(board),
            &pool,
        ),
    }
}

#[debug_handler]
pub async fn new(
    pool_extension: Extension<DbPool>,
    json_extraction: Option<Json<request::NewBoard>>,
) -> Response {
    new_inner(pool_extension, json_extraction).into_response()
}

fn alter_inner(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BoardParams>>,
    json_extraction: Option<Json<request::AlterBoard>>,
) -> Result<response::Board, HttpError> {
    let params = path_extraction.ok_or(handle_path_rejection())?.0;
    let body = json_extraction.ok_or(handle_json_rejection())?.0;

    match body {
        request::AlterBoard::ChangeState(data) => update(
            params.board_id,
            |board: &mut Board| board.change_state(data.new_state),
            &pool,
        ),
        request::AlterBoard::UndoMove => update(
            params.board_id,
            |board: &mut Board| board.undo_move(),
            &pool,
        ),
        request::AlterBoard::Reset => {
            update(params.board_id, |board: &mut Board| board.reset(), &pool)
        }
    }
}

#[debug_handler]
pub async fn alter(
    pool_extension: Extension<DbPool>,
    path_extraction: Option<Path<request::BoardParams>>,
    json_extraction: Option<Json<request::AlterBoard>>,
) -> Response {
    alter_inner(pool_extension, path_extraction, json_extraction).into_response()
}

fn solve_inner(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BoardParams>>,
) -> Result<response::Solve, HttpError> {
    let params = path_extraction.ok_or(handle_path_rejection())?.0;

    let board =
        get_board_state(params.board_id, &pool).map_err(handle_board_state_repository_error)?;

    let maybe_moves: Option<Vec<FlatBoardMove>>;

    if let Ok(cached_solution) = get_solution(board.hash(), &pool) {
        maybe_moves = cached_solution;
    } else {
        maybe_moves = solver::solve(&board).map_err(handle_board_error)?;

        let _solution_cached = create_solution(board.hash(), maybe_moves.clone(), &pool).is_ok();
    }

    match maybe_moves {
        Some(moves) => Ok(response::Solve::Solved(response::Solved::new(moves))),
        None => Ok(response::Solve::UnableToSolve),
    }
}

#[debug_handler]
pub async fn solve(
    pool_extension: Extension<DbPool>,
    path_extraction: Option<Path<request::BoardParams>>,
) -> Response {
    solve_inner(pool_extension, path_extraction).into_response()
}

fn delete_inner(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BoardParams>>,
) -> Result<(), HttpError> {
    let params = path_extraction.ok_or(handle_path_rejection())?.0;

    delete_board_state(params.board_id, &pool).map_err(handle_board_state_repository_error)?;

    Ok(())
}

#[debug_handler]
pub async fn delete(
    pool_extension: Extension<DbPool>,
    path_extraction: Option<Path<request::BoardParams>>,
) -> Response {
    delete_inner(pool_extension, path_extraction).into_response()
}

fn add_block_inner(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BoardParams>>,
    json_extraction: Option<Json<request::AddBlock>>,
) -> Result<response::Board, HttpError> {
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
    path_extraction: Option<Path<request::BoardParams>>,
    json_extraction: Option<Json<request::AddBlock>>,
) -> Response {
    add_block_inner(pool_extension, path_extraction, json_extraction).into_response()
}

fn alter_block_inner(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BlockParams>>,
    json_extraction: Option<Json<request::AlterBlock>>,
) -> Result<response::Board, HttpError> {
    let params = path_extraction.ok_or(handle_path_rejection())?.0;
    let body = json_extraction.ok_or(handle_json_rejection())?.0;

    match body {
        request::AlterBlock::ChangeBlock(data) => update(
            params.board_id,
            |board: &mut Board| board.change_block(params.block_idx, data.new_block),
            &pool,
        ),
        request::AlterBlock::MoveBlock(data) => update(
            params.board_id,
            |board: &mut Board| board.move_block(params.block_idx, data.row_diff, data.col_diff),
            &pool,
        ),
    }
}

#[debug_handler]
pub async fn alter_block(
    pool_extension: Extension<DbPool>,
    path_extraction: Option<Path<request::BlockParams>>,
    json_extraction: Option<Json<request::AlterBlock>>,
) -> Response {
    alter_block_inner(pool_extension, path_extraction, json_extraction).into_response()
}

fn remove_block_inner(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BlockParams>>,
) -> Result<response::Board, HttpError> {
    let params = path_extraction.ok_or(handle_path_rejection())?.0;

    let update_fn = |board: &mut Board| board.remove_block(params.block_idx);

    update(params.board_id, update_fn, &pool)
}

#[debug_handler]
pub async fn remove_block(
    pool_extension: Extension<DbPool>,
    path_extraction: Option<Path<request::BlockParams>>,
) -> Response {
    remove_block_inner(pool_extension, path_extraction).into_response()
}
