use serde::{Deserialize, Serialize};

use crate::errors::game::BoardError;

use super::{
    board::Move,
    utils::{Dimensions, Position},
};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
struct Block {
    id: u8,
    dimensions: Dimensions,
}

impl Block {
    pub const VALID_IDS: [u8; 4] = [1, 2, 3, 4];

    const ONE_BY_ONE: Self = Self {
        id: 1,
        dimensions: Dimensions::ONE_BY_ONE,
    };
    const ONE_BY_TWO: Self = Self {
        id: 2,
        dimensions: Dimensions::ONE_BY_TWO,
    };
    const TWO_BY_ONE: Self = Self {
        id: 3,
        dimensions: Dimensions::TWO_BY_ONE,
    };
    const TWO_BY_TWO: Self = Self {
        id: 4,
        dimensions: Dimensions::TWO_BY_TWO,
    };

    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            1 => Some(Self::ONE_BY_ONE),
            2 => Some(Self::ONE_BY_TWO),
            3 => Some(Self::TWO_BY_ONE),
            4 => Some(Self::TWO_BY_TWO),
            _ => None,
        }
    }

    pub fn dimensions(&self) -> Dimensions {
        self.dimensions.clone()
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
        if !Block::VALID_IDS.contains(&block_id) {
            return None;
        }

        if let Some(min_position) = Position::new(min_row as i8, min_col as i8) {
            let dimensions = Block::from_id(block_id).unwrap().dimensions();

            let new_row = (min_position.row() as u8 + dimensions.rows()) as i8 - 1;
            let new_col = (min_position.col() as u8 + dimensions.cols()) as i8 - 1;

            if let Some(max_position) = Position::new(new_row, new_col) {
                return Some(Self {
                    block_id,
                    min_position,
                    max_position,
                });
            }
        }

        None
    }

    pub fn from_positioned_block(other_positioned_block: &PositionedBlock) -> Option<Self> {
        Self::new(
            other_positioned_block.block_id(),
            other_positioned_block.min_position().row() as u8,
            other_positioned_block.min_position().col() as u8,
        )
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

    pub fn make_move(&mut self, move_: &Move) -> Result<(), BoardError> {
        let new_min_position = Position::new(
            self.min_position.row() as i8 + move_.row_diff(),
            self.min_position.col() as i8 + move_.col_diff(),
        )
        .ok_or(BoardError::BlockPlacementInvalid)?;

        let new_max_position = Position::new(
            self.max_position.row() as i8 + move_.row_diff(),
            self.max_position.col() as i8 + move_.col_diff(),
        )
        .ok_or(BoardError::BlockPlacementInvalid)?;

        self.min_position = new_min_position;
        self.max_position = new_max_position;

        Ok(())
    }

    pub fn undo_move(&mut self, move_: &Move) -> Result<(), BoardError> {
        let move_ = Move::new(-move_.row_diff(), -move_.col_diff())
            .ok_or(BoardError::BlockPlacementInvalid)?;

        self.make_move(&move_)
    }
}