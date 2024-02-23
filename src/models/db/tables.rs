use diesel::prelude::*;

use crate::models::game::board::Board;

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = super::schema::boards)]
pub struct InsertableBoard {
    pub state: String,
    pub blocks: String,
    pub filled: String,
    pub moves: String,
    pub next_moves: String,
}

impl InsertableBoard {
    pub fn from(board: &Board) -> Self {
        Self {
            state: serde_json::to_string(&board.state).unwrap(),
            blocks: serde_json::to_string(&board.blocks).unwrap(),
            filled: serde_json::to_string(&board.filled).unwrap(),
            moves: serde_json::to_string(&board.moves).unwrap(),
            next_moves: serde_json::to_string(&board.next_moves).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Selectable, Queryable)]
#[diesel(table_name = super::schema::boards)]
pub struct SelectableBoard {
    pub id: i32,
    pub state: String,
    pub blocks: String,
    pub filled: String,
    pub moves: String,
    pub next_moves: String,
}

impl SelectableBoard {
    pub fn to_board(&self) -> Board {
        Board::new(
            serde_json::from_str(&self.state).unwrap(),
            serde_json::from_str(&self.blocks).unwrap(),
            serde_json::from_str(&self.filled).unwrap(),
            serde_json::from_str(&self.moves).unwrap(),
            serde_json::from_str(&self.next_moves).unwrap(),
        )
    }
}
