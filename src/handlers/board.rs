use axum::{
    debug_handler,
    extract::{Json, Path},
    response::{IntoResponse, Response},
    Extension,
};

use crate::errors::{
    board::Error as BoardError, handler::Error as HandlerError, http::Error as HttpError,
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
use crate::services::{db::Pool as DbPool, randomizer, solver};

fn update<F>(board_id: i32, update_fn: F, pool: &DbPool) -> Result<Response, HttpError>
where
    F: FnOnce(&mut Board) -> Result<(), BoardError>,
{
    let board = update_board_state(board_id, update_fn, pool)?;

    Ok(response::Board::new(board).into_response())
}

fn random(pool: &DbPool) -> Result<Response, HttpError> {
    let board = create_board_state(pool)?;

    let response = update(
        board.id,
        |board: &mut Board| randomizer::randomize(board),
        pool,
    );

    match solver::solve(&board) {
        Ok(None) | Err(_) => {
            delete_board_state(board.id, pool)?;
            random(pool)
        }
        Ok(_) => response,
    }
}

#[debug_handler]
pub async fn new(
    Extension(pool): Extension<DbPool>,
    json_extraction: Option<Json<request::NewBoard>>,
) -> Result<Response, HttpError> {
    let body = json_extraction.ok_or(HandlerError::InvalidBody)?.0;

    match body {
        request::NewBoard::Empty => {
            let board = create_board_state(&pool)?;
            Ok(response::Board::new(board).into_response())
        }
        request::NewBoard::Random => random(&pool),
    }
}

#[debug_handler]
pub async fn alter(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BoardParams>>,
    json_extraction: Option<Json<request::AlterBoard>>,
) -> Result<Response, HttpError> {
    let params = path_extraction.ok_or(HandlerError::InvalidPath)?.0;
    let body = json_extraction.ok_or(HandlerError::InvalidBody)?.0;

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
pub async fn solve(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BoardParams>>,
) -> Result<Response, HttpError> {
    let params = path_extraction.ok_or(HandlerError::InvalidPath)?.0;

    let board = get_board_state(params.board_id, &pool)?;

    let maybe_moves: Option<Vec<FlatBoardMove>>;

    if let Ok(cached_solution) = get_solution(board.hash(), &pool) {
        maybe_moves = cached_solution;
    } else {
        maybe_moves = solver::solve(&board)?;

        let _solution_cached = create_solution(board.hash(), maybe_moves.clone(), &pool).is_ok();
    }

    let result = match maybe_moves {
        Some(moves) => response::Solve::Solved(response::Solved::new(moves)),
        None => response::Solve::UnableToSolve,
    };

    Ok(result.into_response())
}

#[debug_handler]
pub async fn delete(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BoardParams>>,
) -> Result<Response, HttpError> {
    let params = path_extraction.ok_or(HandlerError::InvalidPath)?.0;

    delete_board_state(params.board_id, &pool)?;

    Ok(().into_response())
}

#[debug_handler]
pub async fn add_block(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BoardParams>>,
    json_extraction: Option<Json<request::AddBlock>>,
) -> Result<Response, HttpError> {
    let params = path_extraction.ok_or(HandlerError::InvalidPath)?.0;
    let body = json_extraction.ok_or(HandlerError::InvalidBody)?.0;

    let new_block = PositionedBlock::new(body.block, body.min_row, body.min_col)
        .ok_or(BoardError::BlockInvalid)?;

    let update_fn = |board: &mut Board| board.add_block(new_block);

    update(params.board_id, update_fn, &pool)
}

#[debug_handler]
pub async fn alter_block(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BlockParams>>,
    json_extraction: Option<Json<request::AlterBlock>>,
) -> Result<Response, HttpError> {
    let params = path_extraction.ok_or(HandlerError::InvalidPath)?.0;
    let body = json_extraction.ok_or(HandlerError::InvalidBody)?.0;

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
pub async fn remove_block(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BlockParams>>,
) -> Result<Response, HttpError> {
    let params = path_extraction.ok_or(HandlerError::InvalidPath)?.0;

    let update_fn = |board: &mut Board| board.remove_block(params.block_idx);

    update(params.board_id, update_fn, &pool)
}
