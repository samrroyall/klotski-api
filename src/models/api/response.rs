use serde::Serialize;

use crate::models::{
    db::tables::BoardState,
    game::{board::BoardMove, move_::Move},
};

#[derive(Debug, Serialize)]
pub struct BuildingResponse {
    id: String,
    blocks: String,
    ready_to_solve: bool,
}

impl BuildingResponse {
    pub fn new(board_state: &BoardState) -> Self {
        Self {
            id: board_state.id.clone(),
            blocks: board_state.blocks.clone(),
            ready_to_solve: board_state.is_ready_to_solve,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SolvingResponse {
    id: String,
    blocks: String,
    next_moves: String,
    is_solved: bool,
}

impl SolvingResponse {
    pub fn new(board_state: &BoardState, next_moves: &Vec<Vec<Move>>) -> Self {
        Self {
            id: board_state.id.clone(),
            blocks: board_state.blocks.clone(),
            next_moves: serde_json::to_string(next_moves).unwrap(),
            is_solved: board_state.is_solved,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SolvedResponse {
    moves: String,
}

impl SolvedResponse {
    pub fn new(moves: &Vec<BoardMove>) -> Self {
        Self {
            moves: serde_json::to_string(moves).unwrap(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SolveResponse {
    Solved(SolvedResponse),
    UnableToSolve,
}
