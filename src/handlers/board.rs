use axum::{
    debug_handler,
    extract::{Json, Path},
    response::{IntoResponse, Response},
    Extension,
};

use crate::errors::{handler::Error as HandlerError, http::Error as HttpError};
use crate::models::{
    api::{request, response},
    game::{board::Board, moves::FlatBoardMove},
};
use crate::repositories::boards::{
    create as create_board, delete as delete_board, get as get_board, update as update_board,
};
use crate::repositories::solutions::{create as create_solution, get as get_solution};
use crate::services::{db::Pool as DbPool, randomizer, solver};

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

    let mut board = create_board(&pool)?;

    tracing::info!("Empty board {} successfully created", board);

    if let request::NewBoard::Random = body {
        let randomized_board = update_board(board.id, randomizer::randomize, &pool)?;

        tracing::info!("Board {} successfully randomized", board.id);

        board = randomized_board;
    }

    Ok(response::Board::new(board).into_response())
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

    let board = match body {
        request::AlterBoard::ChangeState(data) => {
            tracing::info!(
                "Changing state of board {} to {:?}",
                params.board_id,
                data.new_state
            );

            update_board(
                params.board_id,
                |board| board.change_state(data.new_state),
                &pool,
            )
        }
        request::AlterBoard::UndoMove => {
            tracing::info!("Undoing last move for board with id {}", params.board_id);

            update_board(params.board_id, Board::undo_move, &pool)
        }
        request::AlterBoard::Reset => {
            tracing::info!("Resetting board with id {}", params.board_id);

            update_board(params.board_id, Board::reset, &pool)
        }
    }?;

    tracing::info!("Successfully altered board with id {}", params.board_id);

    Ok(response::Board::new(board).into_response())
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
    let board = get_board(params.board_id, &pool)?;

    let maybe_moves: Option<Vec<FlatBoardMove>>;

    if let Ok(cached_solution) = get_solution(board.hash(), &pool) {
        tracing::info!("Returning cached solution for board {}", board);

        maybe_moves = cached_solution;
    } else {
        tracing::info!(
            "No cached solution found for board {}. Attempting to find solution",
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

        response::Solution::Solved(response::Solved::new(moves))
    } else {
        tracing::info!("There is no valid solution for board {}", board);

        response::Solution::UnableToSolve
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

    delete_board(params.board_id, &pool)?;

    tracing::info!("Successfully deleted board with id {}", params.board_id);

    Ok(().into_response())
}
