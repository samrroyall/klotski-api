use serde::Deserialize;

use crate::models::game::{blocks::Block, board::State as BoardState};

#[derive(Debug, Deserialize)]
pub struct BoardParams {
    pub board_id: i32,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NewBoard {
    Empty,
    Random,
}

#[derive(Debug, Deserialize)]
pub struct ChangeState {
    pub new_state: BoardState,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlterBoard {
    ChangeState(ChangeState),
    Reset,
    UndoMove,
}

#[derive(Debug, Deserialize)]
pub struct BlockParams {
    pub board_id: i32,
    pub block_idx: usize,
}

#[derive(Debug, Deserialize)]
pub struct AddBlock {
    pub block: Block,
    pub min_row: u8,
    pub min_col: u8,
}

#[derive(Debug, Deserialize)]
pub struct ChangeBlock {
    pub new_block: Block,
}

#[derive(Debug, Deserialize)]
pub struct MoveBlock {
    pub row_diff: i8,
    pub col_diff: i8,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlterBlock {
    ChangeBlock(ChangeBlock),
    MoveBlock(MoveBlock),
}
