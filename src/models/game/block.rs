use serde::{Deserialize, Serialize};

use crate::errors::game::BoardError;

use super::{move_::Step, utils::Position};

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
            Self::OneByOne => 1,
            Self::OneByTwo => 1,
            Self::TwoByOne => 2,
            Self::TwoByTwo => 2,
        }
    }

    pub fn cols(&self) -> u8 {
        match self {
            Self::OneByOne => 1,
            Self::OneByTwo => 2,
            Self::TwoByOne => 1,
            Self::TwoByTwo => 2,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct PositionedBlock {
    block_id: u8,
    min_position: Position,
    max_position: Position,
}

impl PositionedBlock {
    pub fn new(block_id: u8, min_row: u8, min_col: u8) -> Option<Self> {
        let block = Block::from_id(block_id)?;

        let min_position = Position::new(min_row as i8, min_col as i8)?;

        let max_position = Position::new(
            (min_row + block.rows()) as i8 - 1,
            (min_col + block.cols()) as i8 - 1,
        )?;

        Some(Self {
            block_id,
            min_position,
            max_position,
        })
    }

    pub fn from_positioned_block(block: &PositionedBlock) -> Option<Self> {
        let [min_row, min_col] = block.min_position().to_array();

        Self::new(block.block_id(), min_row as u8, min_col as u8)
    }

    pub fn block_id(&self) -> u8 {
        self.block_id
    }

    pub fn min_position(&self) -> Position {
        self.min_position.clone()
    }

    pub fn max_position(&self) -> Position {
        self.max_position.clone()
    }

    pub fn do_step(&mut self, step: &Step) -> Result<(), BoardError> {
        let [row_diff, col_diff] = step.to_array();

        let [min_row, min_col] = self.min_position.to_array();

        let [max_row, max_col] = self.max_position.to_array();

        self.min_position = Position::new(min_row as i8 + row_diff, min_col as i8 + col_diff)
            .ok_or(BoardError::BlockPlacementInvalid)?;
        self.max_position = Position::new(max_row as i8 + row_diff, max_col as i8 + col_diff)
            .ok_or(BoardError::BlockPlacementInvalid)?;

        Ok(())
    }

    pub fn range(&self) -> Vec<(usize, usize)> {
        (self.min_position.row()..=self.max_position.row())
            .flat_map(move |i| {
                (self.min_position.col()..=self.max_position.col()).map(move |j| (i, j))
            })
            .collect()
    }

    pub fn undo_step(&mut self, step: &Step) -> Result<(), BoardError> {
        self.do_step(&step.opposite())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::game::{board::Board, move_::Step};

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
            PositionedBlock::new(1, 0, 0).is_some()
                && PositionedBlock::new(1, Board::ROWS as u8 - 1, Board::COLS as u8 - 1).is_some()
        );
    }

    #[test]
    fn invalid_positioned_blocks() {
        assert!(
            PositionedBlock::new(4, Board::ROWS as u8 - 1, Board::COLS as u8 - 1).is_none()
                && PositionedBlock::new(1, 0, Board::COLS as u8).is_none()
        );
    }

    #[test]
    fn positioned_block_max_position() {
        let block_one = PositionedBlock::new(1, 0, 0).unwrap();
        let block_two = PositionedBlock::new(4, 0, 1).unwrap();

        assert!(
            block_one.max_position() == Position::new(0, 0).unwrap()
                && block_two.max_position() == Position::new(1, 2).unwrap()
        );
    }

    #[test]
    fn positioned_block_from() {
        let block_one = PositionedBlock::new(1, 0, 0).unwrap();
        let block_two = PositionedBlock::from_positioned_block(&block_one).unwrap();

        assert!(block_one == block_two);
    }

    #[test]
    fn positioned_block_do_step() {
        let mut block_one = PositionedBlock::new(1, 0, 0).unwrap();
        let res = block_one.do_step(&Step::Down);

        assert!(res.is_ok());

        let block_two = PositionedBlock::new(1, 1, 0).unwrap();

        assert_eq!(block_one, block_two);
    }

    #[test]
    fn positioned_block_undo_step() {
        let mut block_two = PositionedBlock::new(1, 0, 1).unwrap();
        let res = block_two.undo_step(&Step::Right);

        assert!(res.is_ok());

        let block_one = PositionedBlock::new(1, 0, 0).unwrap();

        assert_eq!(block_one, block_two);
    }
}
