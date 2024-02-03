use serde::{Deserialize, Serialize};

use crate::errors::domain::BoardError;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
struct Dimensions {
    rows: u8,
    cols: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Block {
    id: u8,
    dimensions: Dimensions,
}

impl Block {
    const ONE_BY_ONE: Self = Self {
        id: 1,
        dimensions: Dimensions { rows: 1, cols: 1 },
    };

    const ONE_BY_TWO: Self = Self {
        id: 2,
        dimensions: Dimensions { rows: 1, cols: 2 },
    };

    const TWO_BY_ONE: Self = Self {
        id: 3,
        dimensions: Dimensions { rows: 2, cols: 1 },
    };

    const TWO_BY_TWO: Self = Self {
        id: 4,
        dimensions: Dimensions { rows: 2, cols: 2 },
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
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Position {
    row: usize,
    col: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct PositionedBlock {
    block_id: u8,
    position: Position, // top-left cell coordinates
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    block_idx: usize,
    row_diff: i8,
    col_diff: i8,
}

#[derive(Debug, Clone)]
pub struct Board {
    blocks: Vec<PositionedBlock>,
    moves: Vec<Move>,
    filled: [[bool; Self::COLS]; Self::ROWS],
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    const ROWS: usize = 5;
    const COLS: usize = 4;
    const WINNING_BLOCK_ID: u8 = 4;
    const NUM_EMPTY_CELLS: u8 = 2;
    const WINNING_POSITION: Position = Position { row: 3, col: 1 };

    fn find_block(&self, target: &PositionedBlock) -> Option<usize> {
        self.blocks.iter().position(|curr| curr == target)
    }

    fn update_filled(&mut self, position: &Position, dimensions: &Dimensions, value: bool) {
        for i in 0..(dimensions.rows as usize) {
            for j in 0..(dimensions.cols as usize) {
                self.filled[position.row + i][position.col + j] = value;
            }
        }
    }

    fn is_placement_valid(&self, block_id: u8, position: &Position) -> bool {
        let dimensions = Block::from_id(block_id).unwrap().dimensions;

        for i in 0..(dimensions.rows as usize) {
            for j in 0..(dimensions.cols as usize) {
                if position.row + i >= Self::ROWS
                    || position.col + j >= Self::COLS
                    || self.filled[position.row + i][position.row + j]
                {
                    return false;
                }
            }
        }

        true
    }
}

impl Board {
    pub fn new() -> Self {
        Self {
            blocks: vec![],
            moves: vec![],
            filled: [[false; Self::COLS]; Self::ROWS],
        }
    }

    pub fn from(
        blocks: Vec<PositionedBlock>,
        moves: Vec<Move>,
        filled: &[[bool; Self::COLS]; Self::ROWS],
    ) -> Self {
        Self {
            blocks: blocks.to_owned(),
            moves: moves.to_owned(),
            filled: filled.to_owned(),
        }
    }

    pub fn blocks(&self) -> Vec<PositionedBlock> {
        self.blocks.clone()
    }

    pub fn filled(&self) -> [[bool; Self::COLS]; Self::ROWS] {
        self.filled
    }

    pub fn moves(&self) -> Vec<Move> {
        self.moves.clone()
    }

    pub fn is_ready_to_solve(&self) -> bool {
        let num_winning_blocks = self.blocks.iter().fold(0, |acc, curr| {
            acc + (curr.block_id == Self::WINNING_BLOCK_ID) as u8
        });

        if num_winning_blocks != 1 {
            return false;
        }

        let empty_cells = self.filled.iter().fold(0, |acc, row| {
            acc + row.iter().fold(0, |acc, &filled| acc + !filled as u8)
        });

        empty_cells == Self::NUM_EMPTY_CELLS
    }

    pub fn is_solved(&self) -> bool {
        let winning_block = PositionedBlock {
            block_id: Self::WINNING_BLOCK_ID,
            position: Position {
                row: Self::WINNING_POSITION.row,
                col: Self::WINNING_POSITION.col,
            },
        };

        self.find_block(&winning_block).is_some()
    }

    pub fn add_block(&mut self, positioned_block: PositionedBlock) -> Result<(), BoardError> {
        let block = Block::from_id(positioned_block.block_id).unwrap();

        if !self.is_placement_valid(positioned_block.block_id, &positioned_block.position) {
            return Err(BoardError::BlockPlacementInvalid);
        }

        self.update_filled(&positioned_block.position, &block.dimensions, true);

        self.blocks.push(positioned_block);

        Ok(())
    }

    pub fn remove_block(&mut self, block_idx: usize) -> Result<(), BoardError> {
        if block_idx >= self.blocks.len() {
            return Err(BoardError::BlockIndexOutOfBounds);
        }

        let PositionedBlock { block_id, position } = self.blocks[block_idx].clone();

        let dimensions = Block::from_id(block_id).unwrap().dimensions;

        self.update_filled(&position, &dimensions, false);

        self.blocks.swap_remove(block_idx);

        Ok(())
    }

    pub fn change_block(&mut self, block_idx: usize, new_block_id: u8) -> Result<(), BoardError> {
        if block_idx >= self.blocks.len() {
            return Err(BoardError::BlockIndexOutOfBounds);
        }

        let PositionedBlock {
            block_id: old_block_id,
            position,
        } = self.blocks[block_idx].clone();

        if old_block_id == new_block_id {
            return Ok(());
        }

        let old_dimensions = Block::from_id(old_block_id).unwrap().dimensions;

        self.update_filled(&position, &old_dimensions, false);

        if !self.is_placement_valid(new_block_id, &position) {
            self.update_filled(&position, &old_dimensions, true);

            return Err(BoardError::BlockPlacementInvalid);
        }

        let new_dimensions = Block::from_id(new_block_id).unwrap().dimensions;

        self.update_filled(&position, &new_dimensions, true);

        self.blocks[block_idx] = PositionedBlock {
            block_id: new_block_id,
            position,
        };

        Ok(())
    }

    pub fn move_block(
        &mut self,
        block_idx: usize,
        row_diff: i8,
        col_diff: i8,
    ) -> Result<(), BoardError> {
        if block_idx >= self.blocks.len() {
            return Err(BoardError::BlockIndexOutOfBounds);
        }

        if row_diff == 0 && col_diff == 0 {
            return Ok(());
        }

        let PositionedBlock {
            block_id,
            position: old_position,
        } = self.blocks[block_idx].clone();

        let dimensions = Block::from_id(block_id).unwrap().dimensions;

        self.update_filled(&old_position, &dimensions, false);

        if old_position.row == 0 && row_diff < 0 || old_position.col == 0 && col_diff < 0 {
            return Err(BoardError::BlockPlacementInvalid);
        }

        let new_position = Position {
            row: (old_position.row as i8 + row_diff) as usize,
            col: (old_position.col as i8 + col_diff) as usize,
        };

        if !self.is_placement_valid(block_id, &new_position) {
            self.update_filled(&old_position, &dimensions, true);

            return Err(BoardError::BlockPlacementInvalid);
        }

        self.update_filled(&new_position, &dimensions, true);

        self.blocks[block_idx] = PositionedBlock {
            block_id,
            position: new_position,
        };

        Ok(())
    }
}
