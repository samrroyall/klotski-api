use serde::Deserialize;

#[derive(Deserialize)]
pub struct BoardQueryParams {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct AddBlockRequest {
    pub block_id: u8,
    pub row: u8,
    pub col: u8,
}

#[derive(Debug, Deserialize)]
pub enum AlterBlockAction {
    ChangeBlock(u8),
    MoveBlock(i8, i8),
}

#[derive(Debug, Deserialize)]
pub struct AlterBlockRequest {
    pub action: AlterBlockAction,
}
