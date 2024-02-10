use serde::{Deserialize, Serialize};

use super::board::Board;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
            Step::Left => 0,
            Step::Right => 0,
        }
    }

    pub fn col_diff(&self) -> i8 {
        match self {
            Step::Up => 0,
            Step::Down => 0,
            Step::Left => -1,
            Step::Right => 1,
        }
    }

    pub fn to_array(&self) -> [i8; 2] {
        [self.row_diff(), self.col_diff()]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    block_idx: usize,
    steps: Vec<Step>,
}

impl Move {
    pub fn new(block_idx: usize, steps: Vec<Step>) -> Option<Self> {
        if !steps.is_empty() && steps.len() <= Board::NUM_EMPTY_CELLS as usize {
            return Some(Self { block_idx, steps });
        }

        None
    }

    pub fn block_idx(&self) -> usize {
        self.block_idx
    }

    pub fn steps(&self) -> &Vec<Step> {
        &self.steps
    }

    pub fn is_opposite(&self, other: &Move) -> bool {
        if self.block_idx != other.block_idx() || self.steps.len() != other.steps().len() {
            return false;
        }

        for (step, other_step) in self.steps.iter().zip(other.steps().iter().rev()) {
            if other_step != &step.opposite() {
                return false;
            }
        }

        true
    }

    pub fn opposite(&self) -> Move {
        Move::new(
            self.block_idx,
            self.steps
                .iter()
                .rev()
                .map(|step| step.opposite())
                .collect(),
        )
        .unwrap()
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
    fn move_is_opposite() {
        let move_one = Move::new(0, vec![Step::Up, Step::Left]).unwrap();
        let move_two = Move::new(0, vec![Step::Right, Step::Down]).unwrap();
        let move_three = Move::new(0, vec![Step::Down, Step::Right]).unwrap();

        assert!(move_one.is_opposite(&move_two));
        assert!(!move_one.is_opposite(&move_three));
    }

    #[test]
    fn move_opposite() {
        let move_one = Move::new(0, vec![Step::Up, Step::Left]).unwrap();
        let move_two = Move::new(0, vec![Step::Right, Step::Down]).unwrap();
        let move_three = Move::new(0, vec![Step::Down, Step::Right]).unwrap();

        assert_eq!(move_one.opposite(), move_two);
        assert_ne!(move_one.opposite(), move_three);
    }
}
