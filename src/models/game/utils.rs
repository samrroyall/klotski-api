use serde::{Deserialize, Serialize};

use super::board::Board;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    pub fn new(row: usize, col: usize) -> Option<Self> {
        if row < Board::ROWS && col < Board::COLS {
            Some(Self { row, col })
        } else {
            None
        }
    }

    pub fn move_by(&self, row_diff: i8, col_diff: i8) -> Option<Self> {
        Position::new(
            usize::try_from(i8::try_from(self.row).unwrap() + row_diff).ok()?,
            usize::try_from(i8::try_from(self.col).unwrap() + col_diff).ok()?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::game::board::Board;

    #[test]
    fn valid_positions() {
        assert!(
            Position::new(0, 0).is_some()
                && Position::new(Board::ROWS - 1, Board::COLS - 1).is_some()
        );
    }

    #[test]
    fn invalid_positions() {
        assert!(Position::new(Board::ROWS, 0).is_none() && Position::new(0, Board::COLS).is_none());
    }
}
