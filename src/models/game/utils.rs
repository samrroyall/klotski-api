use serde::{Deserialize, Serialize};

use super::board::Board;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Dimensions {
    rows: u8,
    cols: u8,
}

impl Dimensions {
    pub const ONE_BY_ONE: Self = Self { rows: 1, cols: 1 };
    pub const ONE_BY_TWO: Self = Self { rows: 1, cols: 2 };
    pub const TWO_BY_ONE: Self = Self { rows: 2, cols: 1 };
    pub const TWO_BY_TWO: Self = Self { rows: 2, cols: 2 };

    pub fn rows(&self) -> u8 {
        self.rows
    }

    pub fn cols(&self) -> u8 {
        self.cols
    }
}

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
