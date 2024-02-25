use diesel::prelude::*;

use crate::models::game::board::Board;

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = super::schema::boards)]
pub struct InsertableBoard {
    pub state: String,
    pub blocks: String,
    pub grid: String,
    pub moves: String,
}

impl InsertableBoard {
    pub fn from(board: &Board) -> Self {
        Self {
            state: serde_json::to_string(&board.state).unwrap(),
            blocks: serde_json::to_string(&board.blocks).unwrap(),
            grid: serde_json::to_string(&board.grid).unwrap(),
            moves: serde_json::to_string(&board.moves).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Selectable, Queryable)]
#[diesel(table_name = super::schema::boards)]
pub struct SelectableBoard {
    pub id: i32,
    pub state: String,
    pub blocks: String,
    pub grid: String,
    pub moves: String,
}

impl SelectableBoard {
    pub fn into_board(self) -> Board {
        Board::new(
            self.id,
            serde_json::from_str(self.state.as_str()).unwrap(),
            serde_json::from_str(self.blocks.as_str()).unwrap(),
            serde_json::from_str(self.grid.as_str()).unwrap(),
            serde_json::from_str(self.moves.as_str()).unwrap(),
        )
    }
}
