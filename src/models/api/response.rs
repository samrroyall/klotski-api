use serde::Serialize;

use crate::models::{
    db::tables::BoardState,
    game::moves::{FlatBoardMove, FlatMove},
};

#[derive(Debug, Serialize)]
pub struct Building {
    id: String,
    blocks: String,
    ready_to_solve: bool,
}

impl Building {
    pub fn new(board_state: &BoardState) -> Self {
        Self {
            id: board_state.id.clone(),
            blocks: board_state.blocks.clone(),
            ready_to_solve: board_state.is_ready_to_solve,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Solving {
    id: String,
    blocks: String,
    next_moves: String,
    is_solved: bool,
}

impl Solving {
    pub fn new(board_state: &BoardState, next_moves: &Vec<Vec<FlatMove>>) -> Self {
        Self {
            id: board_state.id.clone(),
            blocks: board_state.blocks.clone(),
            next_moves: serde_json::to_string(next_moves).unwrap(),
            is_solved: board_state.is_solved,
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
