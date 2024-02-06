use serde::Deserialize;

#[derive(Deserialize)]
pub struct BoardQueryParams {
    pub id: String,
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
    pub row_diff: i8,
    pub col_diff: i8,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlterBlockRequest {
    ChangeBlock(ChangeBlockRequest),
    MoveBlock(MoveBlockRequest),
}
