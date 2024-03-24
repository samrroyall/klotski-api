use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::models::game::{blocks::Block, board::State as BoardState};

#[derive(Debug, Deserialize, IntoParams)]
pub struct BoardParams {
    pub board_id: i32,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct RandomizeParams {
    pub randomize: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangeState {
    pub new_state: BoardState,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlterBoard {
    ChangeState(ChangeState),
    Reset,
    UndoMove,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct BlockParams {
    pub board_id: i32,
    pub block_idx: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddBlock {
    pub block: Block,
    pub min_row: u8,
    pub min_col: u8,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangeBlock {
    pub new_block: Block,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MoveBlock {
    pub row_diff: i8,
    pub col_diff: i8,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
// #[schema(as = AlterBlock)]
pub enum AlterBlock {
    ChangeBlock(ChangeBlock),
    MoveBlock(MoveBlock),
}
