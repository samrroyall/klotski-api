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
    game::blocks::Positioned as PositionedBlock,
};
use crate::repositories::boards::update as update_board;
use crate::services::db::Pool as DbPool;

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
pub async fn add(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BoardParams>>,
    json_extraction: Option<Json<request::AddBlock>>,
) -> Result<Response, HttpError> {
    tracing::info!("Handling request to add block to board");

    let params = path_extraction.ok_or(HandlerError::Path)?.0;
    let body = json_extraction.ok_or(HandlerError::Body)?.0;

    tracing::info!(
        "Attempting to add {:?} block to board with id {}",
        body.block,
        params.board_id
    );

    let new_block = PositionedBlock::new(body.block, body.min_row, body.min_col)
        .ok_or(BoardError::BlockInvalid)?;

    let board = update_board(params.board_id, |board| board.add_block(new_block), &pool)?;

    tracing::info!(
        "Successfully added {:?} block to board with id {}",
        body.block,
        params.board_id
    );

    Ok(response::Board::new(board).into_response())
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
pub async fn alter(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BlockParams>>,
    json_extraction: Option<Json<request::AlterBlock>>,
) -> Result<Response, HttpError> {
    tracing::info!("Handling request to alter block in board");

    let params = path_extraction.ok_or(HandlerError::Path)?.0;
    let body = json_extraction.ok_or(HandlerError::Body)?.0;

    let board = match body {
        request::AlterBlock::ChangeBlock(data) => {
            tracing::info!(
                "Changing block at index {} in board with id {} to {:?}",
                params.block_idx,
                params.board_id,
                data.new_block
            );

            update_board(
                params.board_id,
                |board| board.change_block(params.block_idx, data.new_block),
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

            update_board(
                params.board_id,
                |board| board.move_block(params.block_idx, data.row_diff, data.col_diff),
                &pool,
            )
        }
    }?;

    tracing::info!(
        "Successfully altered block in board with id {}",
        params.board_id
    );

    Ok(response::Board::new(board).into_response())
}

#[utoipa::path(
    delete,
    tag = "Block Operations",
    operation_id = "remove_block",
    path = "/board/{board_id}/block/{block_idx}",
    params(
        ("value" = request::BlockParams, Query,)
    ),
    responses(
        (status = OK, description = "Success", body = Board),
        (status = BAD_REQUEST, description = "Invalid parameters"),
        (status = FORBIDDEN, description = "Action not allowed"),
        (status = NOT_FOUND, description = "Board not found"),
        (status = INTERNAL_SERVER_ERROR, description = "Unhandled exception"),
    ),
)]
#[debug_handler]
pub async fn remove(
    Extension(pool): Extension<DbPool>,
    path_extraction: Option<Path<request::BlockParams>>,
) -> Result<Response, HttpError> {
    tracing::info!("Handling request to remove block from board");

    let params = path_extraction.ok_or(HandlerError::Path)?.0;

    tracing::info!(
        "Attempting to remove block at index {} from board with id {}",
        params.block_idx,
        params.board_id
    );

    let board = update_board(
        params.board_id,
        |board| board.remove_block(params.block_idx),
        &pool,
    )?;

    tracing::info!(
        "Successfully removed block at index {} from board with id {}",
        params.block_idx,
        params.board_id
    );

    Ok(response::Board::new(board).into_response())
}
