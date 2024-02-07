use serde::{Deserialize, Serialize};

use super::{block::PositionedBlock, move_::Move};
use crate::errors::game::BoardError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardMove {
    block_idx: usize,
    move_: Move,
}

impl BoardMove {
    pub fn new(block_idx: usize, move_: Move) -> Self {
        Self { block_idx, move_ }
    }

    pub fn block_idx(&self) -> usize {
        self.block_idx
    }

    pub fn move_(&self) -> Move {
        self.move_.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    blocks: Vec<PositionedBlock>,
    moves: Vec<BoardMove>,
    filled: [[bool; Self::COLS]; Self::ROWS],
}

impl Default for Board {
    fn default() -> Self {
        Self::new(vec![], vec![], &[[false; Self::COLS]; Self::ROWS])
    }
}

impl Board {
    pub const ROWS: usize = 5;
    pub const COLS: usize = 4;
    pub const NUM_EMPTY_CELLS: u8 = 2;

    const WINNING_BLOCK_ID: u8 = 4;
    const WINNING_ROW: usize = 3;
    const WINNING_COL: usize = 1;

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

    fn is_move_valid_for_block(&mut self, block: &PositionedBlock, move_: &Move) -> bool {
        let mut temp_block = block.clone();

        self.update_filled(&temp_block, false);

        if temp_block.make_move(move_).is_ok() {
            let res = self.is_placement_valid(&temp_block);

            let _ = temp_block.undo_move(move_);

            self.update_filled(&temp_block, true);

            return res;
        }

        false
    }

    fn get_next_moves_for_block(&mut self, block: &PositionedBlock) -> Vec<Move> {
        let mut next_moves = vec![];

        for row_diff in -2..=2 {
            for col_diff in -2..=2 {
                if let Some(move_) = Move::new(row_diff, col_diff) {
                    if self.is_move_valid_for_block(block, &move_) {
                        next_moves.push(move_);
                    }
                }
            }
        }

        next_moves
    }
}

impl Board {
    pub fn new(
        blocks: Vec<PositionedBlock>,
        moves: Vec<BoardMove>,
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

    pub fn moves(&self) -> Vec<BoardMove> {
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

        let mut temp_block = self.blocks[block_idx].clone();

        self.update_filled(&temp_block, false);

        if temp_block.make_move(&move_).is_ok() && self.is_placement_valid(&temp_block) {
            self.update_filled(&temp_block, true);

            self.blocks[block_idx] = temp_block;

            self.moves.push(BoardMove::new(block_idx, move_));

            return Ok(());
        }

        let original_block = self.blocks[block_idx].clone();

        self.update_filled(&original_block, true);

        Err(BoardError::BlockPlacementInvalid)
    }

    pub fn undo_move(&mut self) -> Result<(), BoardError> {
        if self.moves.is_empty() {
            return Err(BoardError::NoMovesToUndo);
        }

        let BoardMove { block_idx, move_ } = self.moves.pop().unwrap();
        let [row_diff, col_diff] = move_.to_array();

        if let Err(e) = self.move_block(block_idx, -row_diff, -col_diff) {
            self.moves.push(BoardMove::new(block_idx, move_));

            return Err(e);
        }

        let _ = self.moves.pop();

        Ok(())
    }

    pub fn get_next_moves(&mut self) -> Vec<Vec<Move>> {
        let blocks = self.blocks.clone();

        blocks
            .iter()
            .map(|block| self.get_next_moves_for_block(block))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn find_block() {
        let mut board = Board::default();
        let block = PositionedBlock::new(1, 0, 0).unwrap();
        let other_block = PositionedBlock::new(2, 0, 0).unwrap();
        board.blocks.push(block.clone());

        assert_eq!(board.find_block(&block), Some(0));
        assert_eq!(board.find_block(&other_block), None);
    }

    #[test]
    fn update_filled() {
        let mut board = Board::default();
        let block = PositionedBlock::new(1, 0, 0).unwrap();
        board.update_filled(&block, true);

        assert!(board.filled[0][0]);

        board.update_filled(&block, false);

        assert!(!board.filled[0][0]);
    }

    #[test]
    fn is_placement_valid() {
        let mut board = Board::default();
        let block_one = PositionedBlock::new(1, 0, 0).unwrap();
        let block_two = PositionedBlock::new(2, 1, 0).unwrap();
        board.update_filled(&block_one, true);

        assert!(!board.is_placement_valid(&block_one));
        assert!(board.is_placement_valid(&block_two));
    }

    #[test]
    fn is_move_valid_for_block() {
        let mut board = Board::default();
        let block_one = PositionedBlock::new(1, 0, 0).unwrap();
        let block_two = PositionedBlock::new(1, 0, 1).unwrap();
        board.update_filled(&block_one, true);

        assert!(
            !board.is_move_valid_for_block(&block_one, &Move::LEFT_ONE)
                && !board.is_move_valid_for_block(&block_one, &Move::UP_ONE)
                && board.is_move_valid_for_block(&block_one, &Move::RIGHT_ONE)
                && board.is_move_valid_for_block(&block_one, &Move::DOWN_ONE)
        );

        board.update_filled(&block_two, true);

        assert!(
            !board.is_move_valid_for_block(&block_one, &Move::LEFT_ONE)
                && !board.is_move_valid_for_block(&block_one, &Move::RIGHT_ONE)
                && !board.is_move_valid_for_block(&block_one, &Move::UP_ONE)
                && board.is_move_valid_for_block(&block_one, &Move::DOWN_ONE)
                && !board.is_move_valid_for_block(&block_two, &Move::LEFT_ONE)
                && board.is_move_valid_for_block(&block_two, &Move::RIGHT_ONE)
                && !board.is_move_valid_for_block(&block_two, &Move::UP_ONE)
                && board.is_move_valid_for_block(&block_two, &Move::DOWN_ONE)
        );
    }

    #[test]
    fn get_next_moves_for_block() {
        let mut board = Board::default();
        let block = PositionedBlock::new(1, 0, 0).unwrap();
        board.update_filled(&block, true);

        let move_set = HashSet::from([
            Move::new(1, 0).unwrap(),
            Move::new(2, 0).unwrap(),
            Move::new(1, 1).unwrap(),
            Move::new(0, 1).unwrap(),
            Move::new(0, 2).unwrap(),
        ]);

        let next_moves = board.get_next_moves_for_block(&block);

        assert_eq!(next_moves.len(), 5);

        for move_ in next_moves.iter() {
            assert!(move_set.contains(move_));
        }
    }

    #[test]
    fn is_ready_to_solve() {
        let mut board = Board::default();
        let blocks = [
            PositionedBlock::new(3, 0, 0).unwrap(),
            PositionedBlock::new(3, 0, 3).unwrap(),
            PositionedBlock::new(4, 0, 1).unwrap(),
            PositionedBlock::new(3, 2, 0).unwrap(),
            PositionedBlock::new(2, 2, 1).unwrap(),
            PositionedBlock::new(3, 2, 3).unwrap(),
            PositionedBlock::new(2, 3, 1).unwrap(),
            PositionedBlock::new(1, 4, 0).unwrap(),
        ];
        let final_block = PositionedBlock::new(1, 4, 3).unwrap();

        for block in blocks.iter() {
            board.update_filled(block, true);
            board.blocks.push(block.clone());

            assert!(!board.is_ready_to_solve());
        }

        board.update_filled(&final_block, true);
        board.blocks.push(final_block);

        assert!(board.is_ready_to_solve());
    }

    #[test]
    fn is_solved() {
        let mut board = Board::default();
        let mut block = PositionedBlock::new(4, 2, 1).unwrap();
        board.blocks.push(block.clone());

        assert!(!board.is_solved());

        let _ = block.make_move(&Move::DOWN_ONE);
        board.blocks[0] = block;

        assert!(board.is_solved())
    }

    #[test]
    fn add_block() {
        let mut board = Board::default();
        let block_one = PositionedBlock::new(2, 0, 0).unwrap();
        let block_two = PositionedBlock::new(2, 0, 1).unwrap();

        assert!(board.add_block(block_one).is_ok());
        assert_eq!(board.blocks.len(), 1);
        assert_eq!(
            board.filled,
            [
                [true, true, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false]
            ]
        );
        assert!(board.add_block(block_two).is_err());
    }

    #[test]
    fn remove_block() {
        let mut board = Board::default();
        let block_one = PositionedBlock::new(2, 0, 0).unwrap();
        board.update_filled(&block_one, true);
        board.blocks.push(block_one.clone());

        assert!(board.remove_block(0).is_ok());
        assert_eq!(board.blocks.len(), 0);
        assert_eq!(board.filled, [[false; 4]; 5]);
        assert!(board.remove_block(0).is_err());
    }

    #[test]
    fn change_block() {
        let mut board = Board::default();
        let block = PositionedBlock::new(2, 0, 0).unwrap();
        board.update_filled(&block, true);
        board.blocks.push(block);

        assert!(board.change_block(0, 1).is_ok());
        assert_eq!(
            board.filled,
            [
                [true, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
            ]
        );
        assert!(board.change_block(1, 1).is_err());
    }

    #[test]
    fn move_block() {
        let mut board = Board::default();
        let block_one = PositionedBlock::new(1, 0, 0).unwrap();
        board.update_filled(&block_one, true);
        board.blocks.push(block_one);

        assert!(board.move_block(0, 0, 1).is_ok());
        assert_eq!(
            board.filled,
            [
                [false, true, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
            ]
        );
        assert!(board.move_block(0, -1, 0).is_err());
        assert!(board.move_block(0, 0, -2).is_err());

        let block_two = PositionedBlock::new(4, 3, 2).unwrap();
        board.update_filled(&block_two, true);
        board.blocks.push(block_two);

        assert_eq!(
            board.filled,
            [
                [false, true, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, true, true],
                [false, false, true, true],
            ]
        );
        assert!(board.move_block(1, 0, 1).is_err());
        assert!(board.move_block(1, 1, 0).is_err());
    }

    #[test]
    fn undo_move() {
        let mut board = Board::default();
        let block = PositionedBlock::new(1, 2, 0).unwrap();
        board.update_filled(&block, true);
        board.blocks.push(block);
        board.moves = vec![
            BoardMove::new(0, Move::RIGHT_ONE),
            BoardMove::new(0, Move::DOWN_ONE),
            BoardMove::new(0, Move::LEFT_ONE),
            BoardMove::new(0, Move::DOWN_ONE),
        ];

        assert!(board.undo_move().is_ok());
        assert_eq!(board.moves.len(), 3);
        assert_eq!(
            board.filled,
            [
                [false, false, false, false],
                [true, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
            ]
        );

        assert!(board.undo_move().is_ok());
        assert_eq!(board.moves.len(), 2);
        assert_eq!(
            board.filled,
            [
                [false, false, false, false],
                [false, true, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
            ]
        );

        assert!(board.undo_move().is_ok());
        assert_eq!(board.moves.len(), 1);
        assert_eq!(
            board.filled,
            [
                [false, true, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
            ]
        );

        assert!(board.undo_move().is_ok());
        assert_eq!(board.moves.len(), 0);
        assert_eq!(
            board.filled,
            [
                [true, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
            ]
        );

        assert!(board.undo_move().is_err());
    }

    #[test]
    fn get_next_moves() {
        let mut board = Board::default();
        let block_one = PositionedBlock::new(1, 0, 0).unwrap();
        let block_two = PositionedBlock::new(1, 4, 3).unwrap();
        board.update_filled(&block_one, true);
        board.blocks.push(block_one);
        board.update_filled(&block_two, true);
        board.blocks.push(block_two);

        let block_one_move_set = HashSet::from([
            Move::new(1, 0).unwrap(),
            Move::new(2, 0).unwrap(),
            Move::new(0, 1).unwrap(),
            Move::new(0, 2).unwrap(),
            Move::new(1, 1).unwrap(),
        ]);

        let block_two_move_set = HashSet::from([
            Move::new(-1, 0).unwrap(),
            Move::new(-2, 0).unwrap(),
            Move::new(0, -1).unwrap(),
            Move::new(0, -2).unwrap(),
            Move::new(-1, -1).unwrap(),
        ]);

        let next_moves = board.get_next_moves();

        assert_eq!(next_moves.len(), 2);
        assert_eq!(next_moves[0].len(), 5);

        for move_ in next_moves[0].iter() {
            assert!(block_one_move_set.contains(move_));
        }

        assert_eq!(next_moves[1].len(), 5);

        for move_ in next_moves[1].iter() {
            assert!(block_two_move_set.contains(move_));
        }
    }
}
