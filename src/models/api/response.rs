use serde::Serialize;

use crate::models::{db::tables::BoardState, game::move_::Move};

#[derive(Serialize)]
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

#[derive(Serialize)]
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
