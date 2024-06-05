use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
    pub block_idx: usize,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct FlatMove {
    pub row_diff: i8,
    pub col_diff: i8,
}

impl FlatMove {
    const MAX_DIFF: u8 = Board::MIN_EMPTY_CELLS;

    pub fn new(row_diff: i8, col_diff: i8) -> Option<Self> {
        if u8::try_from(row_diff.abs() + col_diff.abs()).unwrap() <= Self::MAX_DIFF {
            return Some(Self { row_diff, col_diff });
        }

        None
    }

    pub fn from_steps(steps: &[Step]) -> Self {
        Self {
            row_diff: steps.iter().fold(0, |acc, step| acc + step.row_diff()),
            col_diff: steps.iter().fold(0, |acc, step| acc + step.col_diff()),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct FlatBoardMove {
    pub block_idx: usize,
    pub row_diff: i8,
    pub col_diff: i8,
}

impl FlatBoardMove {
    pub fn new(block_idx: usize, move_: &FlatMove) -> Self {
        Self {
            block_idx,
            row_diff: move_.row_diff,
            col_diff: move_.col_diff,
        }
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
    fn flat_move() {
        let flat_move_one = FlatMove::from_steps(&[Step::Up, Step::Left]);

        assert_eq!(flat_move_one.row_diff, -1);
        assert_eq!(flat_move_one.col_diff, -1);

        let flat_move_two = FlatMove::from_steps(&[Step::Left, Step::Up]);

        assert_eq!(flat_move_two.row_diff, -1);
        assert_eq!(flat_move_two.col_diff, -1);

        assert_eq!(flat_move_one, flat_move_two);
    }

    #[test]
    fn flat_board_move() {
        let flat_move_one = FlatMove::from_steps(&[Step::Up, Step::Left]);
        let flat_board_move_one = FlatBoardMove::new(0, &flat_move_one);

        assert_eq!(flat_board_move_one.row_diff, -1);
        assert_eq!(flat_board_move_one.col_diff, -1);

        let flat_move_two = FlatMove::from_steps(&[Step::Left, Step::Up]);
        let flat_board_move_two = FlatBoardMove::new(0, &flat_move_two);

        assert_eq!(flat_board_move_two.row_diff, -1);
        assert_eq!(flat_board_move_two.col_diff, -1);

        assert_eq!(flat_board_move_one, flat_board_move_two);

        let flat_move_three = FlatMove::from_steps(&[Step::Down, Step::Right]);
        let flat_board_move_three = FlatBoardMove::new(0, &flat_move_three);

        assert_eq!(flat_board_move_three.opposite(), flat_board_move_one);
        assert_eq!(flat_board_move_three.opposite(), flat_board_move_two);
    }
}
