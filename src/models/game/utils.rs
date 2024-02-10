use serde::{Deserialize, Serialize};

use super::board::Board;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Position {
    row: usize,
    col: usize,
}

impl Position {
    pub fn new(row: i8, col: i8) -> Option<Self> {
        if row < 0 || row >= Board::ROWS as i8 || col < 0 || col >= Board::COLS as i8 {
            return None;
        }

        Some(Self {
            row: row as usize,
            col: col as usize,
        })
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn to_array(&self) -> [usize; 2] {
        [self.row, self.col]
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
                && Position::new(Board::ROWS as i8 - 1, Board::COLS as i8 - 1).is_some()
        );
    }

    #[test]
    fn invalid_positions() {
        assert!(
            Position::new(-1, 0).is_none()
                && Position::new(0, -1).is_none()
                && Position::new(Board::ROWS as i8, 0).is_none()
                && Position::new(0, Board::COLS as i8).is_none()
        );
    }
}
