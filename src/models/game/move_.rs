use serde::{Deserialize, Serialize};

use super::board::Board;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Move {
    row_diff: i8,
    col_diff: i8,
}

impl Move {
    pub const UP_ONE: Self = Self {
        row_diff: -1,
        col_diff: 0,
    };
    pub const DOWN_ONE: Self = Self {
        row_diff: 1,
        col_diff: 0,
    };
    pub const LEFT_ONE: Self = Self {
        row_diff: 0,
        col_diff: -1,
    };
    pub const RIGHT_ONE: Self = Self {
        row_diff: 0,
        col_diff: 1,
    };

    pub const ALL_ONE_STEP_MOVES: [Self; 4] = [
        Self::UP_ONE,
        Self::DOWN_ONE,
        Self::LEFT_ONE,
        Self::RIGHT_ONE,
    ];

    pub fn new(row_diff: i8, col_diff: i8) -> Option<Self> {
        let diff = row_diff.abs() + col_diff.abs();

        if diff > Board::NUM_EMPTY_CELLS as i8 || diff == 0 {
            return None;
        }

        Some(Self { row_diff, col_diff })
    }

    pub fn row_diff(&self) -> i8 {
        self.row_diff
    }

    pub fn col_diff(&self) -> i8 {
        self.col_diff
    }

    pub fn to_array(&self) -> [i8; 2] {
        [self.row_diff, self.col_diff]
    }

    pub fn is_opposite(&self, other: &Self) -> bool {
        self.row_diff == -other.row_diff && self.col_diff == -other.col_diff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_moves() {
        assert!(
            Move::new(1, 1).is_some()
                && Move::new(-1, 1).is_some()
                && Move::new(1, -1).is_some()
                && Move::new(-1, -1).is_some()
        )
    }

    #[test]
    fn invalid_moves() {
        assert!(
            Move::new(-3, 0).is_none()
                && Move::new(0, -3).is_none()
                && Move::new(3, 0).is_none()
                && Move::new(0, 3).is_none()
                && Move::new(1, 2).is_none()
                && Move::new(-2, 1).is_none()
        );
    }

    #[test]
    fn move_is_opposite() {
        let move_one = Move::new(1, 0).unwrap();
        let move_two = Move::new(-1, 0).unwrap();

        assert!(move_one.is_opposite(&move_two));
    }
}
