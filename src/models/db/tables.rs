use diesel::prelude::*;

use crate::models::game::board::Board;

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = super::schema::board_states)]
pub struct InsertableBoardState {
    pub state: String,
    pub blocks: String,
    pub filled: String,
    pub moves: String,
}

impl InsertableBoardState {
    pub fn from(board: &Board) -> Self {
        Self {
            state: board.state.to_string(),
            blocks: serde_json::to_string(&board.blocks).unwrap(),
            filled: serde_json::to_string(&board.filled).unwrap(),
            moves: serde_json::to_string(&board.moves).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Selectable, Queryable)]
#[diesel(table_name = super::schema::board_states)]
pub struct BoardState {
    pub id: i32,
    pub state: String,
    pub blocks: String,
    pub filled: String,
    pub moves: String,
}

impl BoardState {
    pub fn to_board(&self) -> Board {
        Board::new(
            serde_json::from_str(&self.state).unwrap(),
            serde_json::from_str(&self.blocks).unwrap(),
            serde_json::from_str(&self.moves).unwrap(),
            serde_json::from_str(&self.filled).unwrap(),
        )
    }
}
