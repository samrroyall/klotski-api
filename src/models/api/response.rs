use serde::Serialize;

use crate::models::{
    db::tables::BoardState,
    game::moves::{FlatBoardMove, FlatMove},
};

#[derive(Debug, Serialize)]
pub struct Board {
    id: i32,
    blocks: String,
    state: String,
    next_moves: Option<String>,
}

impl Board {
    pub fn new(board_state: &BoardState) -> Self {
        Self {
            id: board_state.id,
            blocks: board_state.blocks.clone(),
            state: board_state.state.clone(),
            next_moves: None,
        }
    }

    pub fn new_with_next_moves(board_state: &BoardState, next_moves: &Vec<Vec<FlatMove>>) -> Self {
        Self {
            id: board_state.id,
            blocks: board_state.blocks.clone(),
            state: board_state.state.clone(),
            next_moves: Some(serde_json::to_string(next_moves).unwrap()),
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
