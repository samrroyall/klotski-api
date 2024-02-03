use serde::Serialize;

use crate::models::db::tables::BoardState;

#[derive(Serialize)]
pub struct BuildingResponse {
    id: String,
    blocks: String,
    ready_to_solve: bool,
}

impl BuildingResponse {
    pub fn from(board_state: &BoardState) -> Self {
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
    available_moves: String,
    is_solved: bool,
}
