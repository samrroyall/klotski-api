use serde::{Deserialize, Serialize};

use crate::errors::board::Error as BoardError;

use super::{moves::Step, utils::Position};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
enum Block {
    OneByOne,
    OneByTwo,
    TwoByOne,
    TwoByTwo,
}

impl Block {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            1 => Some(Self::OneByOne),
            2 => Some(Self::OneByTwo),
            3 => Some(Self::TwoByOne),
            4 => Some(Self::TwoByTwo),
            _ => None,
        }
    }

    pub fn rows(&self) -> u8 {
        match self {
            Self::OneByOne | Self::OneByTwo => 1,
            Self::TwoByOne | Self::TwoByTwo => 2,
        }
    }

    pub fn cols(&self) -> u8 {
        match self {
            Self::OneByOne | Self::TwoByOne => 1,
            Self::OneByTwo | Self::TwoByTwo => 2,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Positioned {
    pub block_id: u8,
    pub min_position: Position,
    pub max_position: Position,
}

impl Positioned {
    pub fn new(block_id: u8, min_row: usize, min_col: usize) -> Option<Self> {
        let block = Block::from_id(block_id)?;

        let min_position = Position::new(min_row, min_col)?;

        let max_position = Position::new(
            min_row + usize::from(block.rows() - 1),
            min_col + usize::from(block.cols() - 1),
        )?;

        Some(Self {
            block_id,
            min_position,
            max_position,
        })
    }

    pub fn range(&self) -> Vec<(usize, usize)> {
        (self.min_position.row..=self.max_position.row)
            .flat_map(move |i| (self.min_position.col..=self.max_position.col).map(move |j| (i, j)))
            .collect()
    }

    pub fn do_step(&mut self, step: &Step) -> Result<(), BoardError> {
        let row_diff = step.row_diff();
        let col_diff = step.col_diff();

        let new_min_position = self
            .min_position
            .move_by(row_diff, col_diff)
            .ok_or(BoardError::BlockIndexOutOfBounds)?;
        let new_max_position = self
            .max_position
            .move_by(row_diff, col_diff)
            .ok_or(BoardError::BlockIndexOutOfBounds)?;

        self.min_position = new_min_position;
        self.max_position = new_max_position;

        Ok(())
    }

    pub fn undo_step(&mut self, step: &Step) -> Result<(), BoardError> {
        self.do_step(&step.opposite())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::game::{board::Board, moves::Step};

    #[test]
    fn valid_blocks() {
        assert!(
            Block::from_id(1).is_some()
                && Block::from_id(2).is_some()
                && Block::from_id(3).is_some()
                && Block::from_id(4).is_some()
        );
    }

    #[test]
    fn invalid_block() {
        assert!(Block::from_id(5).is_none());
    }

    #[test]
    fn valid_positioned_blocks() {
        assert!(
            Positioned::new(1, 0, 0).is_some()
                && Positioned::new(1, Board::ROWS - 1, Board::COLS - 1).is_some()
        );
    }

    #[test]
    fn invalid_positioned_blocks() {
        assert!(
            Positioned::new(4, Board::ROWS - 1, Board::COLS - 1).is_none()
                && Positioned::new(1, 0, Board::COLS).is_none()
        );
    }

    #[test]
    fn positioned_block_max_position() {
        let block_one = Positioned::new(1, 0, 0).unwrap();
        let block_two = Positioned::new(4, 0, 1).unwrap();

        assert!(
            block_one.max_position == Position::new(0, 0).unwrap()
                && block_two.max_position == Position::new(1, 2).unwrap()
        );
    }

    #[test]
    fn positioned_block_do_step() {
        let mut block_one = Positioned::new(1, 0, 0).unwrap();
        let res = block_one.do_step(&Step::Down);

        assert!(res.is_ok());

        let block_two = Positioned::new(1, 1, 0).unwrap();

        assert_eq!(block_one, block_two);
    }

    #[test]
    fn positioned_block_undo_step() {
        let mut block_two = Positioned::new(1, 0, 1).unwrap();
        let res = block_two.undo_step(&Step::Right);

        assert!(res.is_ok());

        let block_one = Positioned::new(1, 0, 0).unwrap();

        assert_eq!(block_one, block_two);
    }
}
