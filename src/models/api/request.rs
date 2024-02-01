use serde::Deserialize;

use crate::models::domain::game::PositionedBlock;

#[derive(Deserialize)]
pub struct QueryParams {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct AddBlockRequest {
    pub positioned_block: PositionedBlock,
}

#[derive(Debug, Deserialize)]
pub struct ChangeBlockData {
    pub block_id: u8,
}

#[derive(Debug, Deserialize)]
pub struct MoveBlockData {
    pub row_diff: i8,
    pub col_diff: i8,
}

#[derive(Debug, Deserialize)]
pub enum AlterBlockAction {
    ChangeBlock,
    MoveBlock,
}

#[derive(Debug, Deserialize)]
pub struct AlterBlockRequest {
    pub action: AlterBlockAction,
    pub change_data: Option<ChangeBlockData>,
    pub move_data: Option<MoveBlockData>,
}
