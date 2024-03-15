use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use super::board::Board;
use crate::errors::board::Error as BoardError;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Position {
    pub row: u8,
    pub col: u8,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.row, self.col)
    }
}

impl Position {
    pub const MAX_ROW: u8 = Board::ROWS - 1;
    pub const MAX_COL: u8 = Board::COLS - 1;

    pub fn new(row: u8, col: u8) -> Option<Self> {
        if row <= Self::MAX_ROW && col <= Self::MAX_COL {
            return Some(Self { row, col });
        }

        None
    }

    pub fn move_by(&mut self, row_diff: i8, col_diff: i8) -> Result<(), BoardError> {
        let new_row = u8::try_from(i8::try_from(self.row).unwrap() + row_diff)
            .map_err(|_| BoardError::BlockPlacementInvalid)?;
        let new_col = u8::try_from(i8::try_from(self.col).unwrap() + col_diff)
            .map_err(|_| BoardError::BlockPlacementInvalid)?;

        if new_row > Self::MAX_ROW || new_col > Self::MAX_COL {
            return Err(BoardError::BlockPlacementInvalid);
        }

        self.row = new_row;
        self.col = new_col;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_positions() {
        assert!(
            Position::new(0, 0).is_some()
                && Position::new(Position::MAX_ROW, Position::MAX_COL).is_some()
        );
    }

    #[test]
    fn invalid_positions() {
        assert!(
            Position::new(Position::MAX_ROW + 1, 0).is_none()
                && Position::new(0, Position::MAX_COL + 1).is_none()
        );
    }
}
