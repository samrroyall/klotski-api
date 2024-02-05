use serde::{Deserialize, Serialize};

use super::block::PositionedBlock;
use crate::errors::game::BoardError;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub const ALL_SINGLE_MOVES: [Self; 4] = [
        Self::UP_ONE,
        Self::DOWN_ONE,
        Self::LEFT_ONE,
        Self::RIGHT_ONE,
    ];

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

    pub fn is_opposite(&self, other: &Self) -> bool {
        self.row_diff == -other.row_diff && self.col_diff == -other.col_diff
    }

    pub fn to_array(&self) -> [i8; 2] {
        [self.row_diff, self.col_diff]
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
        let [min_row, min_col] = positioned_block.min_position().to_array();
        let [max_row, max_col] = positioned_block.max_position().to_array();

        for i in min_row..=max_row {
            for j in min_col..=max_col {
                self.filled[i][j] = value;
            }
        }
    }

    fn is_placement_valid(&self, positioned_block: &PositionedBlock) -> bool {
        let [min_row, min_col] = positioned_block.min_position().to_array();
        let [max_row, max_col] = positioned_block.max_position().to_array();

        for i in min_row..=max_row {
            for j in min_col..=max_col {
                if self.filled[i][j] {
                    return false;
                }
            }
        }

        true
    }

    fn is_move_valid_for_block(&self, block: &mut PositionedBlock, move_: &Move) -> bool {
        if block.make_move(move_).is_ok() {
            let new_min_pos = block.min_position();
            let new_max_pos = block.max_position();

            let result = match move_.to_array() {
                [0, col_diff] => {
                    let new_col = if col_diff < 0 {
                        new_min_pos.col()
                    } else {
                        new_max_pos.row()
                    };

                    (new_min_pos.row()..=new_max_pos.row())
                        .map(|row| self.filled[row][new_col])
                        .all(|filled| !filled)
                }
                [row_diff, 0] => {
                    let new_row = if row_diff < 0 {
                        new_min_pos.row()
                    } else {
                        new_max_pos.row()
                    };

                    (new_min_pos.col()..=new_max_pos.col())
                        .map(|col| self.filled[new_row][col])
                        .all(|filled| !filled)
                }
                _ => false,
            };

            let _ = block.undo_move(move_);

            return result;
        }

        false
    }

    fn get_next_moves_for_block(&self, block: &PositionedBlock) -> Vec<Move> {
        let mut temp_block = block.clone();

        let mut next_moves = Move::ALL_SINGLE_MOVES
            .into_iter()
            .filter(|move_| self.is_move_valid_for_block(&mut temp_block, move_))
            .collect::<Vec<Move>>();

        let num_single_moves = next_moves.len();

        for i in 0..num_single_moves {
            let move_one = next_moves[i].clone();

            if temp_block.make_move(&move_one).is_ok() {
                let move_moves = Move::ALL_SINGLE_MOVES.into_iter().filter(|move_two| {
                    !move_one.is_opposite(move_two) && self.is_move_valid_for_block(&mut temp_block, move_two)
                });

                next_moves.extend(move_moves);

                let _ = temp_block.undo_move(&move_one);
            }
        }

        next_moves
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

    pub fn get_next_moves(&self) -> Vec<Vec<Move>> {
        self.blocks
            .iter()
            .map(|block| self.get_next_moves_for_block(block))
            .collect()
    }
}
