use serde::{Deserialize, Serialize};

use super::board::Board;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Step {
    Up,
    Down,
    Left,
    Right,
}

impl Step {
    pub const ALL: [Step; 4] = [Step::Up, Step::Down, Step::Left, Step::Right];

    pub fn row_diff(&self) -> i8 {
        match self {
            Step::Up => -1,
            Step::Down => 1,
            Step::Left | Step::Right => 0,
        }
    }

    pub fn col_diff(&self) -> i8 {
        match self {
            Step::Left => -1,
            Step::Right => 1,
            Step::Up | Step::Down => 0,
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Step::Up => Step::Down,
            Step::Down => Step::Up,
            Step::Left => Step::Right,
            Step::Right => Step::Left,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move {
    pub block_idx: u8,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlatMove {
    pub row_diff: i8,
    pub col_diff: i8,
}

impl FlatMove {
    pub fn new(row_diff: i8, col_diff: i8) -> Option<Self> {
        if row_diff.abs() + col_diff.abs() > i8::try_from(Board::NUM_EMPTY_CELLS).unwrap() {
            return None;
        }

        Some(Self { row_diff, col_diff })
    }

    pub fn from_steps(steps: &[Step]) -> Self {
        Self {
            row_diff: steps.iter().fold(0, |acc, step| acc + step.row_diff()),
            col_diff: steps.iter().fold(0, |acc, step| acc + step.col_diff()),
        }
    }

    pub fn is_opposite(&self, other: &Self) -> bool {
        self.row_diff == -other.row_diff && self.col_diff == -other.col_diff
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlatBoardMove {
    pub block_idx: u8,
    pub row_diff: i8,
    pub col_diff: i8,
}

impl FlatBoardMove {
    pub fn new(block_idx: u8, move_: &FlatMove) -> Self {
        Self {
            block_idx,
            row_diff: move_.row_diff,
            col_diff: move_.col_diff,
        }
    }

    pub fn is_opposite(&self, other: &Self) -> bool {
        self.block_idx == other.block_idx
            && self.row_diff == -other.row_diff
            && self.col_diff == -other.col_diff
    }

    pub fn opposite(&self) -> Self {
        Self {
            block_idx: self.block_idx,
            row_diff: -self.row_diff,
            col_diff: -self.col_diff,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_is_opposite() {
        assert_eq!(Step::Left.opposite(), Step::Right);
        assert_eq!(Step::Up.opposite(), Step::Down);
        assert_ne!(Step::Up.opposite(), Step::Right);
        assert_ne!(Step::Down.opposite(), Step::Left);
    }

    #[test]
    fn flat_move() {
        let flat_move_one = FlatMove::from_steps(&[Step::Up, Step::Left]);

        assert_eq!(flat_move_one.row_diff, -1);
        assert_eq!(flat_move_one.col_diff, -1);

        let flat_move_two = FlatMove::from_steps(&[Step::Left, Step::Up]);

        assert_eq!(flat_move_two.row_diff, -1);
        assert_eq!(flat_move_two.col_diff, -1);

        assert_eq!(flat_move_one, flat_move_two);
    }
}
