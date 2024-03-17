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

    tracing::info!("Board {} successfully updated", board);

    Ok(response::Board::new(board).into_response())
}

#[utoipa::path(
    post,
    tag = "Board Operations",
    operation_id = "create_board",
    path = "/board",
    request_body(content = NewBoard),
    responses(
        (status = OK, description = "Success", body = Board),
        (status = BAD_REQUEST, description = "Invalid parameters"),
        (status = FORBIDDEN, description = "Action not allowed"),
        (status = NOT_FOUND, description = "Board not found"),
        (status = INTERNAL_SERVER_ERROR, description = "Unhandled exception"),
    ),
)]
#[debug_handler]
pub async fn new(
    Extension(pool): Extension<DbPool>,
    json_extraction: Option<Json<request::NewBoard>>,
) -> Result<Response, HttpError> {
    tracing::info!("Handling request to create a new board");

    let body = json_extraction.ok_or(HandlerError::InvalidBody)?.0;

    let board = create_board_state(&pool)?;

    tracing::info!("Empty board {} successfully created", board);

    match body {
        request::NewBoard::Empty => Ok(response::Board::new(board).into_response()),
        request::NewBoard::Random => {
            let randomized_board = update(
                board.id,
                |board: &mut Board| randomizer::randomize(board),
                &pool,
            )?;

            tracing::info!("Board {} successfully randomized", board.id);

            Ok(randomized_board)
        }
    }
}

#[utoipa::path(
    put,
    tag = "Board Operations",
    operation_id = "alter_board",
    path = "/board/{board_id}",
    params(request::BoardParams),
    request_body(content = AlterBoard),
    responses(
        (status = OK, description = "Success", body = Board),
        (status = BAD_REQUEST, description = "Invalid parameters"),
        (status = FORBIDDEN, description = "Action not allowed"),
        (status = NOT_FOUND, description = "Board not found"),
        (status = INTERNAL_SERVER_ERROR, description = "Unhandled exception"),
    ),
)]
#[debug_handler]
pub async fn alter(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BoardParams>>,
    json_extraction: Option<Json<request::AlterBoard>>,
) -> Result<Response, HttpError> {
    tracing::info!("Handling request to alter board");

    let params = path_extraction.ok_or(HandlerError::InvalidPath)?.0;
    let body = json_extraction.ok_or(HandlerError::InvalidBody)?.0;

    let result = match body {
        request::AlterBoard::ChangeState(data) => {
            tracing::info!(
                "Changing state of board {} to {:?}",
                params.board_id,
                data.new_state
            );

            update(
                params.board_id,
                |board: &mut Board| board.change_state(data.new_state),
                &pool,
            )
        }
        request::AlterBoard::UndoMove => {
            tracing::info!("Undoing last move for board with id {}", params.board_id);

            update(
                params.board_id,
                |board: &mut Board| board.undo_move(),
                &pool,
            )
        }
        request::AlterBoard::Reset => {
            tracing::info!("Resetting board with id {}", params.board_id);

            update(params.board_id, |board: &mut Board| board.reset(), &pool)
        }
    }?;

    tracing::info!("Successfully altered board with id {}", params.board_id);

    Ok(result)
}

#[utoipa::path(
    post,
    tag = "Board Operations",
    operation_id = "solve_board",
    path = "/board/{board_id}/solve",
    params(request::BoardParams),
    responses(
        (status = OK, description = "Success", body = Solve),
        (status = BAD_REQUEST, description = "Invalid parameters"),
        (status = FORBIDDEN, description = "Action not allowed"),
        (status = NOT_FOUND, description = "Board not found"),
        (status = INTERNAL_SERVER_ERROR, description = "Unhandled exception"),
    ),
)]
#[debug_handler]
pub async fn solve(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BoardParams>>,
) -> Result<Response, HttpError> {
    tracing::info!("Handling request to solve board");

    let params = path_extraction.ok_or(HandlerError::InvalidPath)?.0;
    let board = get_board_state(params.board_id, &pool)?;

    let maybe_moves: Option<Vec<FlatBoardMove>>;

    if let Ok(cached_solution) = get_solution(board.hash(), &pool) {
        tracing::info!("Returning cached solution for board {}", board);

        maybe_moves = cached_solution;
    } else {
        tracing::info!(
            "No cached solution found for board {}. Attemping to find solution",
            board
        );

        maybe_moves = solver::solve(&board)?;

        let _solution_cached = create_solution(board.hash(), maybe_moves.clone(), &pool).is_ok();
    }

    let result = if let Some(moves) = maybe_moves {
        tracing::info!(
            "Solution of length {} found for board {}",
            moves.len(),
            board
        );

        response::Solve::Solved(response::Solved::new(moves))
    } else {
        tracing::info!("There is no valid solution for board {}", board);

        response::Solve::UnableToSolve
    };

    Ok(result.into_response())
}

#[utoipa::path(
    delete,
    tag = "Board Operations",
    operation_id = "delete_board",
    path = "/board/{board_id}",
    params(request::BoardParams),
    responses(
        (status = OK, description = "Success", body = Board),
        (status = BAD_REQUEST, description = "Invalid parameters"),
        (status = NOT_FOUND, description = "Board not found"),
        (status = INTERNAL_SERVER_ERROR, description = "Unhandled exception"),
    ),
)]
#[debug_handler]
pub async fn delete(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BoardParams>>,
) -> Result<Response, HttpError> {
    tracing::info!("Handling request to delete board");

    let params = path_extraction.ok_or(HandlerError::InvalidPath)?.0;

    delete_board_state(params.board_id, &pool)?;

    tracing::info!("Successfully deleted board with id {}", params.board_id);

    Ok(().into_response())
}

#[utoipa::path(
    post,
    tag = "Block Operations",
    operation_id = "add_block",
    path = "/board/{board_id}/block",
    params(request::BoardParams),
    request_body(content = AddBlock),
    responses(
        (status = OK, description = "Success", body = Board),
        (status = BAD_REQUEST, description = "Invalid parameters"),
        (status = FORBIDDEN, description = "Action not allowed"),
        (status = NOT_FOUND, description = "Board not found"),
        (status = INTERNAL_SERVER_ERROR, description = "Unhandled exception"),
    ),
)]
#[debug_handler]
pub async fn add_block(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BoardParams>>,
    json_extraction: Option<Json<request::AddBlock>>,
) -> Result<Response, HttpError> {
    tracing::info!("Handling request to add block to board");

    let params = path_extraction.ok_or(HandlerError::InvalidPath)?.0;
    let body = json_extraction.ok_or(HandlerError::InvalidBody)?.0;

    tracing::info!(
        "Attempting to add {:?} block to board with id {}",
        body.block,
        params.board_id
    );

    let new_block = PositionedBlock::new(body.block, body.min_row, body.min_col)
        .ok_or(BoardError::BlockInvalid)?;

    let update_fn = |board: &mut Board| board.add_block(new_block);

    let result = update(params.board_id, update_fn, &pool)?;

    tracing::info!(
        "Successfully added {:?} block to board with id {}",
        body.block,
        params.board_id
    );

    Ok(result)
}

#[utoipa::path(
    put,
    tag = "Block Operations",
    operation_id = "alter_block",
    path = "/board/{board_id}/block/{block_idx}",
    params(request::BlockParams),
    request_body(content = AlterBlock),
    responses(
        (status = OK, description = "Success", body = Board),
        (status = BAD_REQUEST, description = "Invalid parameters"),
        (status = FORBIDDEN, description = "Action not allowed"),
        (status = NOT_FOUND, description = "Board not found"),
        (status = INTERNAL_SERVER_ERROR, description = "Unhandled exception"),
    ),
)]
#[debug_handler]
pub async fn alter_block(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BlockParams>>,
    json_extraction: Option<Json<request::AlterBlock>>,
) -> Result<Response, HttpError> {
    tracing::info!("Handling request to alter block in board");

    let params = path_extraction.ok_or(HandlerError::InvalidPath)?.0;
    let body = json_extraction.ok_or(HandlerError::InvalidBody)?.0;

    let result = match body {
        request::AlterBlock::ChangeBlock(data) => {
            tracing::info!(
                "Changing block at index {} in board with id {} to {:?}",
                params.block_idx,
                params.board_id,
                data.new_block
            );

            update(
                params.board_id,
                |board: &mut Board| board.change_block(params.block_idx, data.new_block),
                &pool,
            )
        }
        request::AlterBlock::MoveBlock(data) => {
            tracing::info!(
                "Moving block at index {} in board with id {} by ({},{})",
                params.block_idx,
                params.board_id,
                data.row_diff,
                data.col_diff
            );

            update(
                params.board_id,
                |board: &mut Board| {
                    board.move_block(params.block_idx, data.row_diff, data.col_diff)
                },
                &pool,
            )
        }
    }?;

    tracing::info!(
        "Successfully altered block in board with id {}",
        params.board_id
    );

    Ok(result)
}

#[utoipa::path(
    delete,
    tag = "Block Operations",
    operation_id = "remove_block",
    path = "/board/{board_id}/block/{block_idx}",
    params(request::BlockParams),
    responses(
        (status = OK, description = "Success", body = Board),
        (status = BAD_REQUEST, description = "Invalid parameters"),
        (status = FORBIDDEN, description = "Action not allowed"),
        (status = NOT_FOUND, description = "Board not found"),
        (status = INTERNAL_SERVER_ERROR, description = "Unhandled exception"),
    ),
)]
#[debug_handler]
pub async fn remove_block(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BlockParams>>,
) -> Result<Response, HttpError> {
    tracing::info!("Handling request to remove block from board");

    let params = path_extraction.ok_or(HandlerError::InvalidPath)?.0;

    let update_fn = |board: &mut Board| board.remove_block(params.block_idx);

    tracing::info!(
        "Attempting to remove block at index {} from board with id {}",
        params.block_idx,
        params.board_id
    );

    let result = update(params.board_id, update_fn, &pool)?;

    tracing::info!(
        "Successfully removed block at index {} from board with id {}",
        params.block_idx,
        params.board_id
    );

    Ok(result)
}
