use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{moves::Step, utils::Position};
use crate::errors::board::Error as BoardError;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Block {
    OneByOne,
    OneByTwo,
    TwoByOne,
    TwoByTwo,
}

impl Block {
    pub fn rows(self) -> u8 {
        match self {
            Self::OneByOne | Self::OneByTwo => 1,
            Self::TwoByOne | Self::TwoByTwo => 2,
        }
    }

    pub fn cols(self) -> u8 {
        match self {
            Self::OneByOne | Self::TwoByOne => 1,
            Self::OneByTwo | Self::TwoByTwo => 2,
        }
    }

    pub fn size(self) -> u8 {
        self.rows() * self.cols()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = PositionedBlock)]
pub struct Positioned {
    pub block: Block,
    pub min_position: Position,
    pub max_position: Position,
    pub range: Vec<(u8, u8)>,
}

impl Display for Positioned {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Block({:?}@{})", self.block, self.min_position)
    }
}

impl Positioned {
    fn range(min_position: &Position, max_position: &Position) -> Vec<(u8, u8)> {
        (min_position.row..=max_position.row)
            .flat_map(move |i| (min_position.col..=max_position.col).map(move |j| (i, j)))
            .collect()
    }

    pub fn new(block: Block, min_row: u8, min_col: u8) -> Option<Self> {
        let min_position = Position::new(min_row, min_col)?;

        let max_position = Position::new(min_row + block.rows() - 1, min_col + block.cols() - 1)?;

        Some(Self {
            block,
            range: Self::range(&min_position, &max_position),
            min_position,
            max_position,
        })
    }

    pub fn move_by(&mut self, row_diff: i8, col_diff: i8) -> Result<(), BoardError> {
        let mut new_min_position = self.min_position.clone();
        let mut new_max_position = self.max_position.clone();

        new_min_position.move_by(row_diff, col_diff)?;
        new_max_position.move_by(row_diff, col_diff)?;

        self.range = Self::range(&new_min_position, &new_max_position);
        self.min_position = new_min_position;
        self.max_position = new_max_position;

        Ok(())
    }

    pub fn do_step(&mut self, step: &Step) -> Result<(), BoardError> {
        self.move_by(step.row_diff(), step.col_diff())
    }

    pub fn undo_step(&mut self, step: &Step) -> Result<(), BoardError> {
        self.do_step(&step.opposite())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::game::{moves::Step, utils::Position};

    #[test]
    fn valid_positioned_blocks() {
        assert!(
            Positioned::new(Block::OneByOne, 0, 0).is_some()
                && Positioned::new(Block::OneByOne, Position::MAX_ROW, Position::MAX_COL).is_some()
        );
    }

    #[test]
    fn invalid_positioned_blocks() {
        assert!(
            Positioned::new(Block::TwoByTwo, Position::MAX_ROW, Position::MAX_COL).is_none()
                && Positioned::new(Block::OneByOne, 0, Position::MAX_COL + 1).is_none()
        );
    }

    #[test]
    fn positioned_block_max_position() {
        let block_one = Positioned::new(Block::OneByOne, 0, 0).unwrap();
        let block_two = Positioned::new(Block::TwoByTwo, 0, 1).unwrap();

        assert!(
            block_one.max_position == Position::new(0, 0).unwrap()
                && block_two.max_position == Position::new(1, 2).unwrap()
        );
    }

    #[test]
    fn positioned_block_do_step() {
        let mut block_one = Positioned::new(Block::OneByOne, 0, 0).unwrap();
        let res = block_one.do_step(&Step::Down);

        assert!(res.is_ok());

        let block_two = Positioned::new(Block::OneByOne, 1, 0).unwrap();

        assert_eq!(block_one, block_two);
    }

    #[test]
    fn positioned_block_undo_step() {
        let mut block_two = Positioned::new(Block::OneByOne, 0, 1).unwrap();
        let res = block_two.undo_step(&Step::Right);

        assert!(res.is_ok());

        let block_one = Positioned::new(Block::OneByOne, 0, 0).unwrap();

        assert_eq!(block_one, block_two);
    }
}
