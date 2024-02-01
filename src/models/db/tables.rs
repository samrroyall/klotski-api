use diesel::prelude::*;
use serde_json::to_string;

use crate::models::domain::game::Board;

#[derive(AsChangeset, Insertable, Selectable, Queryable, Debug, Clone)]
#[diesel(table_name = super::schema::board_states)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BoardState {
    pub id: String,
    pub is_ready_to_solve: bool,
    pub is_solved: bool,
    pub blocks: String,
    pub filled: String,
    pub moves: String,
}

impl BoardState {
    pub fn from(new_id: &String, board: &Board) -> Self {
        Self {
            id: String::from(new_id),
            is_ready_to_solve: board.is_ready_to_solve(),
            is_solved: board.is_solved(),
            blocks: to_string(&board.blocks()).unwrap(),
            filled: to_string(&board.filled()).unwrap(),
            moves: to_string(&board.moves()).unwrap(),
        }
    }

    pub fn to_board(&self) -> Board {
        Board::from(
            serde_json::from_str(&self.blocks).unwrap(),
            serde_json::from_str(&self.moves).unwrap(),
            &serde_json::from_str(&self.filled).unwrap(),
        )
    }
}
