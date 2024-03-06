use diesel::prelude::*;

use crate::models::game::{board::Board, moves::FlatBoardMove};

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

#[derive(Debug, Insertable)]
#[diesel(table_name = super::schema::solutions)]
pub struct InsertableSolution {
    pub hash: i64,
    pub moves: Option<String>,
}

impl InsertableSolution {
    pub fn from(hash: u64, moves: Option<Vec<FlatBoardMove>>) -> Self {
        Self {
            hash: hash as i64,
            moves: moves.map(|moves| serde_json::to_string(&moves).unwrap()),
        }
    }
}

#[derive(Debug, Clone, Selectable, Queryable)]
#[diesel(table_name = super::schema::solutions)]
pub struct SelectableSolution {
    pub id: i32,
    pub hash: i64,
    pub moves: Option<String>,
}

impl SelectableSolution {
    pub fn get_moves(self) -> Option<Vec<FlatBoardMove>> {
        self.moves
            .map(|moves| serde_json::from_str(moves.as_str()).unwrap())
    }
}
