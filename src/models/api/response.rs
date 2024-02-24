use serde::Serialize;

use crate::models::{
    db::tables::SelectableBoard,
    game::{
        blocks::Positioned as PositionedBlock,
        board::{Board as Board_, State as BoardState},
        moves::{FlatBoardMove, FlatMove},
    },
};

#[derive(Debug, Serialize)]
pub struct Board {
    id: i32,
    state: BoardState,
    blocks: Vec<PositionedBlock>,
    grid: [[u8; Board_::COLS as usize]; Board_::ROWS as usize],
    next_moves: Vec<Vec<FlatMove>>,
}

impl Board {
    pub fn new(board_record: &SelectableBoard) -> Self {
        Self {
            id: board_record.id,
            state: serde_json::from_str(&board_record.state).unwrap(),
            blocks: serde_json::from_str(&board_record.blocks).unwrap(),
            grid: serde_json::from_str(&board_record.grid).unwrap(),
            next_moves: serde_json::from_str(&board_record.next_moves).unwrap(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Solved {
    moves: Vec<FlatBoardMove>,
}

impl Solved {
    pub fn new(moves: &[FlatBoardMove]) -> Self {
        Self {
            moves: moves.to_owned(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Solve {
    Solved(Solved),
    UnableToSolve,
}
