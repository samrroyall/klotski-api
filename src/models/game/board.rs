use serde::{Deserialize, Serialize};

use crate::errors::game::BoardError;

use super::block::PositionedBlock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    row_diff: i8,
    col_diff: i8,
}

impl Move {
    pub fn new(row_diff: i8, col_diff: i8) -> Option<Self> {
        if row_diff.abs() >= Board::ROWS as i8 || col_diff.abs() >= Board::COLS as i8 {
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
    pub const ROWS: usize = 5;
    pub const COLS: usize = 4;
    const WINNING_BLOCK_ID: u8 = 4;
    const WINNING_ROW: usize = 3;
    const WINNING_COL: usize = 1;
    const NUM_EMPTY_CELLS: u8 = 2;

    fn find_block(&self, target: &PositionedBlock) -> Option<usize> {
        self.blocks.iter().position(|curr| curr == target)
    }

    fn update_filled(&mut self, positioned_block: &PositionedBlock, value: bool) {
        let min_position = positioned_block.min_position();
        let max_position = positioned_block.max_position();

        for i in min_position.row()..=max_position.row() {
            for j in min_position.col()..=max_position.col() {
                self.filled[i][j] = value;
            }
        }
    }

    fn is_placement_valid(&self, positioned_block: &PositionedBlock) -> bool {
        let min_position = positioned_block.min_position();
        let max_position = positioned_block.max_position();

        for i in min_position.row()..=max_position.row() {
            for j in min_position.col()..=max_position.col() {
                if self.filled[i][j] {
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
            acc + (curr.block_id() == Self::WINNING_BLOCK_ID) as u8
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
        let winning_block = PositionedBlock::new(
            Self::WINNING_BLOCK_ID,
            Self::WINNING_ROW as u8,
            Self::WINNING_COL as u8,
        )
        .unwrap();

        self.find_block(&winning_block).is_some()
    }

    pub fn add_block(&mut self, positioned_block: PositionedBlock) -> Result<(), BoardError> {
        if !self.is_placement_valid(&positioned_block) {
            return Err(BoardError::BlockPlacementInvalid);
        }

        self.update_filled(&positioned_block, true);

        self.blocks.push(positioned_block);

        Ok(())
    }

    pub fn remove_block(&mut self, block_idx: usize) -> Result<(), BoardError> {
        if block_idx >= self.blocks.len() {
            return Err(BoardError::BlockIndexOutOfBounds);
        }

        let positioned_block = self.blocks[block_idx].clone();

        self.update_filled(&positioned_block, false);

        self.blocks.swap_remove(block_idx);

        Ok(())
    }

    pub fn change_block(&mut self, block_idx: usize, new_block_id: u8) -> Result<(), BoardError> {
        if block_idx >= self.blocks.len() {
            return Err(BoardError::BlockIndexOutOfBounds);
        }

        let positioned_block = self.blocks[block_idx].clone();

        if positioned_block.block_id() == new_block_id {
            return Ok(());
        }

        let min_position = positioned_block.min_position();

        let new_positioned_block = PositionedBlock::new(
            new_block_id,
            min_position.row() as u8,
            min_position.col() as u8,
        )
        .ok_or(BoardError::BlockPlacementInvalid)?;

        self.update_filled(&positioned_block, false);

        if !self.is_placement_valid(&new_positioned_block) {
            self.update_filled(&positioned_block, true);

            return Err(BoardError::BlockPlacementInvalid);
        }

        self.update_filled(&new_positioned_block, true);

        self.blocks[block_idx] = new_positioned_block;

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

        let move_ = Move::new(row_diff, col_diff).ok_or(BoardError::BlockPlacementInvalid)?;

        if move_.row_diff() == 0 && move_.col_diff() == 0 {
            return Ok(());
        }

        let mut positioned_block = self.blocks[block_idx].clone();

        self.update_filled(&positioned_block, false);

        positioned_block.make_move(&move_)?;

        if !self.is_placement_valid(&positioned_block) {
            self.update_filled(&positioned_block, true);

            return Err(BoardError::BlockPlacementInvalid);
        }

        self.update_filled(&positioned_block, true);

        self.blocks[block_idx] = positioned_block;

        Ok(())
    }
}
