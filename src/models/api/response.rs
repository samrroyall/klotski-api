use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::models::game::{
    blocks::{Block, Positioned as PositionedBlock},
    board::{Board as Board_, State as BoardState},
    moves::{FlatBoardMove, FlatMove},
};

#[derive(Debug, Serialize)]
pub struct Board {
    id: i32,
    state: BoardState,
    blocks: Vec<PositionedBlock>,
    grid: [Option<Block>; (Board_::COLS * Board_::ROWS) as usize],
    next_moves: Vec<Vec<FlatMove>>,
}

impl Board {
    pub fn new(mut board: Board_) -> Self {
        let next_moves = board.get_next_moves();

        Self {
            id: board.id,
            state: board.state,
            blocks: board.blocks,
            grid: board.grid,
            next_moves,
        }
    }
}

impl IntoResponse for Board {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct Solved {
    moves: Vec<FlatBoardMove>,
}

impl Solved {
    pub fn new(moves: Vec<FlatBoardMove>) -> Self {
        Self { moves }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Solve {
    Solved(Solved),
    UnableToSolve,
}

impl IntoResponse for Solve {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}
