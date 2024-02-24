use serde::Serialize;

use crate::models::{db::tables::SelectableBoard, game::moves::FlatBoardMove};

#[derive(Debug, Serialize)]
pub struct Board {
    id: i32,
    state: String,
    blocks: String,
    grid: String,
    next_moves: String,
}

impl Board {
    pub fn new(board_record: &SelectableBoard) -> Self {
        Self {
            id: board_record.id,
            state: board_record.state.clone(),
            blocks: board_record.blocks.clone(),
            grid: board_record.grid.clone(),
            next_moves: board_record.next_moves.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Solved {
    moves: String,
}

impl Solved {
    pub fn new(moves: &Vec<FlatBoardMove>) -> Self {
        Self {
            moves: serde_json::to_string(moves).unwrap(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Solve {
    Solved(Solved),
    UnableToSolve,
}
