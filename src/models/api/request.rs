use serde::Deserialize;

use crate::models::game::move_::Step;

#[derive(Debug, Deserialize)]
pub struct BoardParams {
    pub board_id: String,
}

#[derive(Debug, Deserialize)]
pub struct BlockParams {
    pub board_id: String,
    pub block_idx: usize,
}

#[derive(Debug, Deserialize)]
pub struct AddBlockRequest {
    pub block_id: u8,
    pub min_row: u8,
    pub min_col: u8,
}

#[derive(Debug, Deserialize)]
pub struct ChangeBlockRequest {
    pub new_block_id: u8,
}

#[derive(Debug, Deserialize)]
pub struct MoveBlockRequest {
    pub steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlterBlockRequest {
    ChangeBlock(ChangeBlockRequest),
    MoveBlock(MoveBlockRequest),
}
