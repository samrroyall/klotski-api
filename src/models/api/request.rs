use serde::Deserialize;

use crate::models::game::moves::Step;

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
pub struct AddBlock {
    pub block_id: u8,
    pub min_row: usize,
    pub min_col: usize,
}

#[derive(Debug, Deserialize)]
pub struct ChangeBlock {
    pub new_block_id: u8,
}

#[derive(Debug, Deserialize)]
pub struct MoveBlock {
    pub steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlterBlock {
    ChangeBlock(ChangeBlock),
    MoveBlock(MoveBlock),
}
