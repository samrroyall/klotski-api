use std::{
    collections::hash_map::DefaultHasher,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
    blocks::{Block, Positioned as PositionedBlock},
    moves::{FlatBoardMove, FlatMove, Step},
};
use crate::{errors::board::Error as BoardError, models::game::utils::Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[schema(as = BoardState)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Building,
    ReadyToSolve,
    Solving,
    Solved,
}

#[derive(Debug, Clone)]
pub struct Board {
    pub id: i32,
    pub state: State,
    pub blocks: Vec<PositionedBlock>,
    pub grid: [Option<Block>; (Self::ROWS * Self::COLS) as usize],
    pub moves: Vec<FlatBoardMove>,
}

impl Default for Board {
    fn default() -> Self {
        Self::new(
            0,
            State::Building,
            vec![],
            [None; (Self::COLS * Self::ROWS) as usize],
            vec![],
        )
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Board(ID:{}, State:{:?}, Blocks:[", self.id, self.state)?;
        for (i, block) in self.blocks.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{block}")?;
        }
        write!(f, "])")
    }
}

impl Board {
    pub const ROWS: u8 = 5;
    pub const COLS: u8 = 4;
    pub const MIN_EMPTY_CELLS: u8 = 2;

    const WINNING_BLOCK: Block = Block::TwoByTwo;
    const WINNING_ROW: u8 = 3;
    const WINNING_COL: u8 = 1;

    fn num_cells_free(&self) -> usize {
        self.grid.iter().filter(|cell| cell.is_none()).count() - usize::from(Self::MIN_EMPTY_CELLS)
    }

    fn is_ready_to_solve(&self) -> bool {
        1 == self
            .blocks
            .iter()
            .filter(|positioned_block| positioned_block.block == Self::WINNING_BLOCK)
            .count()
            && 0 == self.num_cells_free()
    }

    fn update_grid_range(&mut self, range: &[(u8, u8)], value: Option<Block>) {
        range
            .iter()
            .for_each(|(i, j)| self.grid[usize::from(i * Self::COLS + j)] = value);
    }

    fn is_range_empty(&self, range: &[(u8, u8)]) -> bool {
        range
            .iter()
            .all(|(i, j)| self.grid[usize::from(i * Self::COLS + j)].is_none())
    }

    fn is_step_valid_for_block(&self, block: &PositionedBlock, step: &Step) -> bool {
        match step {
            Step::Up => (block.min_position.col..=block.max_position.col).all(|col| {
                u8::try_from(i8::try_from(block.min_position.row).unwrap() - 1)
                    .ok()
                    .is_some_and(|row_above| {
                        Position::new(row_above, col).is_some_and(|new_position| {
                            self.grid[usize::from(new_position.row * Self::COLS + col)].is_none()
                        })
                    })
            }),
            Step::Down => (block.min_position.col..=block.max_position.col).all(|col| {
                Position::new(block.max_position.row + 1, col).is_some_and(|new_position| {
                    self.grid[usize::from(new_position.row * Self::COLS + col)].is_none()
                })
            }),
            Step::Left => (block.min_position.row..=block.max_position.row).all(|row| {
                u8::try_from(i8::try_from(block.min_position.col).unwrap() - 1)
                    .ok()
                    .is_some_and(|col_above| {
                        Position::new(row, col_above).is_some_and(|new_position| {
                            self.grid[usize::from(row * Self::COLS + new_position.col)].is_none()
                        })
                    })
            }),
            Step::Right => (block.min_position.row..=block.max_position.row).all(|row| {
                Position::new(row, block.max_position.col + 1).is_some_and(|new_position| {
                    self.grid[usize::from(row * Self::COLS + new_position.col)].is_none()
                })
            }),
        }
    }

    fn get_next_moves_for_block(&self, block: &PositionedBlock) -> Vec<FlatMove> {
        let mut moves = vec![vec![]];

        let mut block = block.clone();

        for depth in 0..Self::MIN_EMPTY_CELLS {
            for i in 0..moves.len() {
                for step in &moves[i] {
                    block.do_step(step).unwrap();
                }

                let opposite_last_move = &moves[i].last().map(Step::opposite);

                for ref next_step in Step::ALL {
                    if opposite_last_move.is_some()
                        && opposite_last_move.as_ref() == Some(next_step)
                    {
                        continue;
                    }

                    if self.is_step_valid_for_block(&block, next_step)
                        && block.do_step(next_step).is_ok()
                    {
                        let mut new_move = moves[i].clone();
                        new_move.push(next_step.clone());

                        moves.push(new_move);

                        block.undo_step(next_step).unwrap();
                    }
                }

                for step in moves[i].iter().rev() {
                    block.undo_step(step).unwrap();
                }
            }

            if depth == 0 {
                moves.remove(0);
            }
        }

        moves
            .into_iter()
            .map(|move_| FlatMove::from_steps(move_.as_slice()))
            .collect()
    }
}

impl Board {
    pub fn new(
        id: i32,
        state: State,
        blocks: Vec<PositionedBlock>,
        grid: [Option<Block>; (Self::COLS * Self::ROWS) as usize],
        moves: Vec<FlatBoardMove>,
    ) -> Self {
        Self {
            id,
            state,
            blocks,
            grid,
            moves,
        }
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.grid.hash(&mut hasher);
        hasher.finish()
    }

    pub fn change_state(&mut self, new_state: State) -> Result<(), BoardError> {
        if self.state == new_state {
            return Ok(());
        }

        match (self.state, new_state) {
            (State::Building, State::ReadyToSolve) => {
                if !self.is_ready_to_solve() {
                    return Err(BoardError::BoardStateInvalid);
                }
            }
            (State::ReadyToSolve, State::Building | State::Solving) => {}
            (State::Solving, State::ReadyToSolve) => {
                if !self.moves.is_empty() {
                    return Err(BoardError::BoardStateInvalid);
                }
            }
            (State::Solving, State::Solved) => {
                if !self.is_solved() {
                    return Err(BoardError::BoardStateInvalid);
                }
            }
            (State::Solved, State::Solving) => {
                if self.is_solved() {
                    return Err(BoardError::BoardStateInvalid);
                }
            }
            _ => {
                return Err(BoardError::BoardStateInvalid);
            }
        }

        self.state = new_state;

        Ok(())
    }

    pub fn is_solved(&self) -> bool {
        self.blocks.iter().any(|block| {
            block.block == Self::WINNING_BLOCK
                && block.min_position.row == Self::WINNING_ROW
                && block.min_position.col == Self::WINNING_COL
        })
    }

    pub fn add_block(&mut self, positioned_block: PositionedBlock) -> Result<(), BoardError> {
        if self.state != State::Building {
            self.change_state(State::Building)?;
        }

        if !self.is_range_empty(&positioned_block.range) {
            return Err(BoardError::BlockPlacementInvalid);
        }

        if self.num_cells_free() < usize::from(positioned_block.block.size()) {
            return Err(BoardError::BlockPlacementInvalid);
        }

        self.update_grid_range(&positioned_block.range, Some(positioned_block.block));

        self.blocks.push(positioned_block);

        let _is_ready_to_solve = self.change_state(State::ReadyToSolve).is_ok();

        Ok(())
    }

    pub fn change_block(&mut self, block_idx: usize, new_block: Block) -> Result<(), BoardError> {
        if self.state != State::Building {
            self.change_state(State::Building)?;
        }

        let positioned_block = self
            .blocks
            .get(block_idx)
            .cloned()
            .ok_or(BoardError::BlockIndexOutOfBounds)?;

        if positioned_block.block == new_block {
            return Ok(());
        }

        let old_size = positioned_block.block.size();
        let new_size = new_block.size();

        if new_size > old_size && self.num_cells_free() < usize::from(new_size - old_size) {
            return Err(BoardError::BlockPlacementInvalid);
        }

        let new_positioned_block = PositionedBlock::new(
            new_block,
            positioned_block.min_position.row,
            positioned_block.min_position.col,
        )
        .ok_or(BoardError::BlockPlacementInvalid)?;

        self.update_grid_range(&positioned_block.range, None);

        if !self.is_range_empty(&new_positioned_block.range) {
            self.update_grid_range(&positioned_block.range, Some(positioned_block.block));

            return Err(BoardError::BlockPlacementInvalid);
        }

        self.update_grid_range(
            &new_positioned_block.range,
            Some(new_positioned_block.block),
        );

        self.blocks[block_idx] = new_positioned_block;

        let _is_ready_to_solve = self.change_state(State::ReadyToSolve).is_ok();

        Ok(())
    }

    pub fn get_next_moves(&mut self) -> Vec<Vec<FlatMove>> {
        self.blocks
            .iter()
            .map(|block| {
                let mut moves = self.get_next_moves_for_block(block);
                moves.dedup();
                moves
            })
            .collect()
    }

    pub fn remove_block(&mut self, block_idx: usize) -> Result<(), BoardError> {
        if self.state != State::Building {
            self.change_state(State::Building)?;
        }

        let positioned_block = self
            .blocks
            .get(block_idx)
            .cloned()
            .ok_or(BoardError::BlockIndexOutOfBounds)?;

        self.update_grid_range(&positioned_block.range, None);

        self.blocks.swap_remove(block_idx);

        let _is_not_ready_to_solve = self.change_state(State::Building).is_ok();

        Ok(())
    }

    pub fn move_block_unchecked(&mut self, block_idx: usize, row_diff: i8, col_diff: i8) {
        let mut positioned_block = self.blocks.get(block_idx).cloned().unwrap();

        self.update_grid_range(&positioned_block.range, None);

        positioned_block.move_by(row_diff, col_diff).unwrap();

        self.update_grid_range(&positioned_block.range, Some(positioned_block.block));

        self.blocks[block_idx] = positioned_block;

        self.moves.push(FlatBoardMove::new(
            block_idx,
            &FlatMove::new(row_diff, col_diff).unwrap(),
        ));

        let _is_solved = self.change_state(State::Solved).is_ok();
    }

    pub fn move_block(
        &mut self,
        block_idx: usize,
        row_diff: i8,
        col_diff: i8,
    ) -> Result<(), BoardError> {
        if self.state != State::Solving {
            self.change_state(State::Solving)?;
        }

        let next_moves = self.get_next_moves();

        let is_valid_move = next_moves
            .get(block_idx)
            .unwrap()
            .iter()
            .any(|move_| move_.row_diff == row_diff && move_.col_diff == col_diff);

        if !is_valid_move {
            return Err(BoardError::BlockPlacementInvalid);
        }

        let mut positioned_block = self
            .blocks
            .get(block_idx)
            .cloned()
            .ok_or(BoardError::BlockIndexOutOfBounds)?;

        self.update_grid_range(&positioned_block.range, None);

        if positioned_block.move_by(row_diff, col_diff).is_err() {
            self.update_grid_range(&positioned_block.range, Some(positioned_block.block));

            return Err(BoardError::BlockPlacementInvalid);
        };

        self.update_grid_range(&positioned_block.range, Some(positioned_block.block));

        self.blocks[block_idx] = positioned_block;

        self.moves.push(FlatBoardMove::new(
            block_idx,
            &FlatMove::new(row_diff, col_diff).unwrap(),
        ));

        let _is_solved = self.change_state(State::Solved).is_ok();

        Ok(())
    }

    pub fn undo_move_unchecked(&mut self) {
        let opposite_move = self.moves.pop().unwrap().opposite();

        let mut block = self.blocks.get(opposite_move.block_idx).cloned().unwrap();

        self.update_grid_range(&block.range, None);

        block
            .move_by(opposite_move.row_diff, opposite_move.col_diff)
            .unwrap();

        self.update_grid_range(&block.range, Some(block.block));

        self.blocks[opposite_move.block_idx] = block;

        let _is_not_solved = self.change_state(State::Solving).is_ok();
    }

    pub fn undo_move(&mut self) -> Result<(), BoardError> {
        if ![State::Solving, State::Solved].contains(&self.state) {
            return Err(BoardError::BoardStateInvalid);
        }

        let opposite_move = self
            .moves
            .pop()
            .ok_or(BoardError::NoMovesToUndo)?
            .opposite();

        let mut block = self
            .blocks
            .get(opposite_move.block_idx)
            .cloned()
            .ok_or(BoardError::BlockIndexOutOfBounds)?;

        self.update_grid_range(&block.range, None);

        if block
            .move_by(opposite_move.row_diff, opposite_move.col_diff)
            .is_err()
        {
            self.update_grid_range(&block.range, Some(block.block));

            return Err(BoardError::BlockPlacementInvalid);
        }

        self.update_grid_range(&block.range, Some(block.block));

        self.blocks[opposite_move.block_idx] = block;

        let _is_not_solved = self.change_state(State::Solving).is_ok();

        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), BoardError> {
        if ![State::Solving, State::Solved].contains(&self.state) {
            return Err(BoardError::BoardStateInvalid);
        }

        while !self.moves.is_empty() {
            self.undo_move()?;
        }

        let _board_is_ready_to_solve = self.change_state(State::ReadyToSolve).is_ok();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_grid_range() {
        let mut board = Board::default();

        let block = PositionedBlock::new(Block::OneByOne, 0, 0).unwrap();
        board.update_grid_range(&block.range, Some(block.block));

        assert_eq!(board.grid[0], Some(block.block));

        board.update_grid_range(&block.range, None);

        assert_eq!(board.grid[0], None);
    }

    #[test]
    fn is_range_empty() {
        let mut board = Board::default();

        let block_one = PositionedBlock::new(Block::OneByOne, 0, 0).unwrap();
        board.update_grid_range(&block_one.range, Some(block_one.block));

        let block_two = PositionedBlock::new(Block::OneByTwo, 1, 0).unwrap();

        assert!(!board.is_range_empty(&block_one.range));
        assert!(board.is_range_empty(&block_two.range));
    }

    #[test]
    fn is_step_valid_for_block() {
        let mut board = Board::default();

        let block_one = PositionedBlock::new(Block::OneByOne, 0, 0).unwrap();
        board.update_grid_range(&block_one.range, Some(block_one.block));

        let block_two = PositionedBlock::new(Block::OneByTwo, 0, 1).unwrap();
        board.update_grid_range(&block_two.range, Some(block_two.block));

        assert!(!board.is_step_valid_for_block(&block_one, &Step::Left));
        assert!(!board.is_step_valid_for_block(&block_one, &Step::Right));
        assert!(!board.is_step_valid_for_block(&block_one, &Step::Up));
        assert!(board.is_step_valid_for_block(&block_one, &Step::Down));

        assert!(!board.is_step_valid_for_block(&block_two, &Step::Left));
        assert!(!board.is_step_valid_for_block(&block_two, &Step::Up));
        assert!(board.is_step_valid_for_block(&block_two, &Step::Right));
        assert!(board.is_step_valid_for_block(&block_two, &Step::Down));

        let block_three = PositionedBlock::new(Block::OneByOne, 1, 0).unwrap();
        board.update_grid_range(&block_three.range, Some(block_three.block));

        assert!(!board.is_step_valid_for_block(&block_one, &Step::Down));

        assert!(!board.is_step_valid_for_block(&block_three, &Step::Up));
        assert!(!board.is_step_valid_for_block(&block_three, &Step::Left));
        assert!(board.is_step_valid_for_block(&block_three, &Step::Right));
        assert!(board.is_step_valid_for_block(&block_three, &Step::Down));

        assert_eq!(
            board.grid,
            [
                Some(Block::OneByOne),
                Some(Block::OneByTwo),
                Some(Block::OneByTwo),
                None,
                Some(Block::OneByOne),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ]
        )
    }

    #[test]
    fn get_next_moves_for_block_down_right() {
        let mut board = Board::default();

        let block_one = PositionedBlock::new(Block::OneByOne, 0, 0).unwrap();
        board.update_grid_range(&block_one.range, Some(block_one.block));

        let block_two = PositionedBlock::new(Block::OneByOne, 0, 1).unwrap();
        board.update_grid_range(&block_two.range, Some(block_two.block));

        let block_three = PositionedBlock::new(Block::OneByOne, 1, 0).unwrap();
        board.update_grid_range(&block_three.range, Some(block_three.block));

        let block_one_moves = board.get_next_moves_for_block(&block_one);

        assert_eq!(block_one_moves.len(), 0);

        let block_two_moves = board.get_next_moves_for_block(&block_two);

        assert_eq!(
            board.grid,
            [
                Some(Block::OneByOne),
                Some(Block::OneByOne),
                None,
                None,
                Some(Block::OneByOne),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ]
        );

        let expected_block_two_moves = [
            FlatMove::new(0, 1).unwrap(),
            FlatMove::new(0, 2).unwrap(),
            FlatMove::new(1, 1).unwrap(),
            FlatMove::new(1, 1).unwrap(),
            FlatMove::new(1, 0).unwrap(),
            FlatMove::new(2, 0).unwrap(),
        ];

        println!("{:?}", block_two_moves);

        assert_eq!(block_two_moves.len(), expected_block_two_moves.len());

        for move_ in block_two_moves {
            assert!(expected_block_two_moves.contains(&move_));
        }
    }

    #[test]
    fn get_next_moves_for_block_up_left() {
        let mut board = Board::default();

        let block_one = PositionedBlock::new(Block::OneByOne, 4, 3).unwrap();
        board.update_grid_range(&block_one.range, Some(block_one.block));

        let block_two = PositionedBlock::new(Block::OneByOne, 4, 2).unwrap();
        board.update_grid_range(&block_two.range, Some(block_two.block));

        let block_three = PositionedBlock::new(Block::OneByOne, 3, 3).unwrap();
        board.update_grid_range(&block_three.range, Some(block_three.block));

        let block_one_moves = board.get_next_moves_for_block(&block_one);

        assert_eq!(block_one_moves.len(), 0);

        let block_two_moves = board.get_next_moves_for_block(&block_two);

        assert_eq!(
            board.grid,
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(Block::OneByOne),
                None,
                None,
                Some(Block::OneByOne),
                Some(Block::OneByOne),
            ]
        );

        let expected_block_two_moves = [
            FlatMove::new(0, -1).unwrap(),
            FlatMove::new(0, -2).unwrap(),
            FlatMove::new(-1, -1).unwrap(),
            FlatMove::new(-1, -1).unwrap(),
            FlatMove::new(-1, 0).unwrap(),
            FlatMove::new(-2, 0).unwrap(),
        ];

        assert_eq!(block_two_moves.len(), expected_block_two_moves.len());

        for move_ in block_two_moves {
            assert!(expected_block_two_moves.contains(&move_));
        }
    }

    #[test]
    fn get_next_moves() {
        let blocks = vec![
            PositionedBlock::new(Block::TwoByOne, 0, 0).unwrap(),
            PositionedBlock::new(Block::TwoByTwo, 0, 1).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 0, 3).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 0).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 2, 1).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 3).unwrap(),
            PositionedBlock::new(Block::OneByOne, 3, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 3, 2).unwrap(),
            PositionedBlock::new(Block::OneByOne, 4, 0).unwrap(),
            PositionedBlock::new(Block::OneByOne, 4, 3).unwrap(),
        ];

        let mut board = Board::default();
        for block in blocks {
            board.add_block(block).unwrap();
        }

        assert_eq!(board.state, State::ReadyToSolve);

        assert!(board.change_state(State::Solving).is_ok());

        let next_moves = board.get_next_moves();

        assert_eq!(next_moves.iter().fold(0, |acc, moves| acc + moves.len()), 8);

        let expected_moves = [
            vec![FlatMove::new(1, 0).unwrap(), FlatMove::new(1, 1).unwrap()],
            vec![FlatMove::new(1, 0).unwrap(), FlatMove::new(1, -1).unwrap()],
            vec![FlatMove::new(0, 1).unwrap(), FlatMove::new(0, 2).unwrap()],
            vec![FlatMove::new(0, -1).unwrap(), FlatMove::new(0, -2).unwrap()],
        ];

        for i in 0..next_moves.len() {
            if i < 6 {
                assert_eq!(next_moves[i].len(), 0);
            } else {
                assert_eq!(next_moves[i], expected_moves[i - 6]);
            }
        }
    }

    #[test]
    fn hash() {
        let mut board = Board::default();
        let blocks = [
            PositionedBlock::new(Block::TwoByOne, 0, 0).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 0, 3).unwrap(),
            PositionedBlock::new(Block::TwoByTwo, 0, 1).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 0).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 2, 1).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 3).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 3, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 4, 0).unwrap(),
            PositionedBlock::new(Block::OneByOne, 4, 3).unwrap(),
        ];

        for block in blocks.iter() {
            board.update_grid_range(&block.range, Some(block.block));
            board.blocks.push(block.clone());
        }

        assert_eq!(board.hash(), 9403663965540605277);
    }

    #[test]
    fn change_state() {
        let mut board = Board::default();

        assert!(board.change_state(State::Building).is_ok());
        assert!(board.change_state(State::Solving).is_err());

        let blocks = [
            PositionedBlock::new(Block::TwoByOne, 0, 0).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 0, 3).unwrap(),
            PositionedBlock::new(Block::TwoByTwo, 0, 1).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 0).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 2, 1).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 3).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 3, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 4, 0).unwrap(),
            PositionedBlock::new(Block::OneByOne, 4, 3).unwrap(),
        ];

        for block in blocks.iter() {
            board.update_grid_range(&block.range, Some(block.block));
            board.blocks.push(block.clone());
        }

        assert!(board.change_state(State::Solving).is_err());
        assert!(board.change_state(State::ReadyToSolve).is_ok());
        assert!(board.change_state(State::Building).is_ok());
        assert!(board.change_state(State::Solving).is_err());
        assert!(board.change_state(State::ReadyToSolve).is_ok());
        assert!(board.change_state(State::Solving).is_ok());

        let move_ = FlatBoardMove::new(0, &FlatMove::new(1, 0).unwrap());
        board.moves.push(move_);

        assert!(board.change_state(State::ReadyToSolve).is_err());
        assert!(board.change_state(State::Building).is_err());
    }

    #[test]
    fn is_ready_to_solve() {
        let mut board = Board::default();
        let blocks = [
            PositionedBlock::new(Block::TwoByOne, 0, 0).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 0, 3).unwrap(),
            PositionedBlock::new(Block::TwoByTwo, 0, 1).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 0).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 2, 1).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 3).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 3, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 4, 0).unwrap(),
        ];
        let final_block = PositionedBlock::new(Block::OneByOne, 4, 3).unwrap();

        for block in blocks.iter() {
            board.update_grid_range(&block.range, Some(block.block));
            board.blocks.push(block.clone());

            assert!(!board.is_ready_to_solve());
        }

        board.update_grid_range(&final_block.range, Some(final_block.block));
        board.blocks.push(final_block);

        assert!(board.is_ready_to_solve());
    }

    #[test]
    fn is_solved() {
        let mut board = Board::default();
        let mut block = PositionedBlock::new(Block::TwoByTwo, 2, 1).unwrap();
        board.blocks.push(block.clone());

        assert!(!board.is_solved());

        block.do_step(&Step::Down).unwrap();
        board.blocks[0] = block;

        assert!(board.is_solved())
    }

    #[test]
    fn add_block() {
        let mut board = Board::default();

        let block_one = PositionedBlock::new(Block::OneByTwo, 0, 0).unwrap();

        assert!(board.add_block(block_one).is_ok());
        assert_eq!(board.blocks.len(), 1);
        assert_eq!(
            board.grid,
            [
                Some(Block::OneByTwo),
                Some(Block::OneByTwo),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None
            ]
        );

        let block_two = PositionedBlock::new(Block::OneByTwo, 0, 1).unwrap();

        assert!(board.add_block(block_two).is_err());
    }

    #[test]
    fn add_block_not_enough_cells_free() {
        let mut board = Board::default();

        let blocks = [
            PositionedBlock::new(Block::TwoByOne, 0, 0).unwrap(),
            PositionedBlock::new(Block::TwoByTwo, 0, 1).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 0, 3).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 0).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 2, 1).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 3).unwrap(),
            PositionedBlock::new(Block::OneByOne, 3, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 3, 2).unwrap(),
            PositionedBlock::new(Block::OneByOne, 4, 3).unwrap(),
        ];

        let last_block = PositionedBlock::new(Block::OneByTwo, 4, 0).unwrap();

        for block in blocks.into_iter() {
            assert!(board.add_block(block).is_ok());
        }

        assert_eq!(
            board.add_block(last_block),
            Err(BoardError::BlockPlacementInvalid)
        );
    }

    #[test]
    fn remove_block() {
        let mut board = Board::default();

        let block_one = PositionedBlock::new(Block::OneByTwo, 0, 0).unwrap();
        board.update_grid_range(&block_one.range, Some(block_one.block));
        board.blocks.push(block_one.clone());

        assert!(board.remove_block(0).is_ok());
        assert_eq!(board.blocks.len(), 0);
        assert_eq!(board.grid, [None; 20]);
        assert!(board.remove_block(0).is_err());
    }

    #[test]
    fn change_block() {
        let mut board = Board::default();

        let block = PositionedBlock::new(Block::OneByTwo, 0, 0).unwrap();
        board.update_grid_range(&block.range, Some(block.block));
        board.blocks.push(block);

        assert!(board.change_block(0, Block::OneByOne).is_ok());
        assert_eq!(
            board.grid,
            [
                Some(Block::OneByOne),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ]
        );
        assert!(board.change_block(1, Block::OneByOne).is_err());
    }

    #[test]
    fn change_block_not_enough_cells_free() {
        let mut board = Board::default();

        let blocks = [
            PositionedBlock::new(Block::TwoByOne, 0, 0).unwrap(),
            PositionedBlock::new(Block::TwoByTwo, 0, 1).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 0, 3).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 0).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 2, 1).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 3).unwrap(),
            PositionedBlock::new(Block::OneByOne, 3, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 3, 2).unwrap(),
            PositionedBlock::new(Block::OneByOne, 4, 0).unwrap(),
            PositionedBlock::new(Block::OneByOne, 4, 3).unwrap(),
        ];

        for block in blocks.iter() {
            board.update_grid_range(&block.range, Some(block.block));
            board.blocks.push(block.clone());
        }

        assert_eq!(
            board.change_block(8, Block::OneByTwo),
            Err(BoardError::BlockPlacementInvalid)
        );
    }

    #[test]
    fn move_block_unchecked() {
        let mut board = Board::default();

        let block_one = PositionedBlock::new(Block::OneByOne, 0, 0).unwrap();
        board.update_grid_range(&block_one.range, Some(block_one.block));
        board.blocks.push(block_one);
        board.state = State::Solving;

        board.move_block_unchecked(0, 0, 1);

        assert_eq!(
            board.grid,
            [
                None,
                Some(Block::OneByOne),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ]
        );

        board.move_block_unchecked(0, 1, 0);
        board.move_block_unchecked(0, 0, -1);

        let block_two = PositionedBlock::new(Block::TwoByTwo, 3, 2).unwrap();
        board.update_grid_range(&block_two.range, Some(block_two.block));
        board.blocks.push(block_two);

        assert_eq!(
            board.grid,
            [
                None,
                None,
                None,
                None,
                Some(Block::OneByOne),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(Block::TwoByTwo),
                Some(Block::TwoByTwo),
                None,
                None,
                Some(Block::TwoByTwo),
                Some(Block::TwoByTwo),
            ]
        );

        board.move_block_unchecked(1, 0, -2);
        board.move_block_unchecked(1, -1, 1);
        board.move_block_unchecked(1, -1, 0);
        board.move_block_unchecked(1, 0, -1);

        assert_eq!(
            board.grid,
            [
                None,
                None,
                None,
                None,
                Some(Block::TwoByTwo),
                Some(Block::TwoByTwo),
                None,
                None,
                Some(Block::TwoByTwo),
                Some(Block::TwoByTwo),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ]
        );
    }

    #[test]
    fn move_block() {
        let mut board = Board::default();

        let block_one = PositionedBlock::new(Block::OneByOne, 0, 0).unwrap();
        board.update_grid_range(&block_one.range, Some(block_one.block));
        board.blocks.push(block_one);
        board.state = State::Solving;

        assert!(board.move_block(0, 0, 1).is_ok());

        assert_eq!(
            board.grid,
            [
                None,
                Some(Block::OneByOne),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ]
        );

        assert_eq!(
            board.move_block(0, -1, 0),
            Err(BoardError::BlockPlacementInvalid)
        );
        assert!(board.move_block(0, 0, -1).is_ok());
        assert!(board.move_block(0, 1, 0).is_ok());
        assert_eq!(
            board.move_block(0, 0, -1),
            Err(BoardError::BlockPlacementInvalid)
        );

        let block_two = PositionedBlock::new(Block::TwoByTwo, 3, 2).unwrap();
        board.update_grid_range(&block_two.range, Some(block_two.block));
        board.blocks.push(block_two);

        assert_eq!(
            board.grid,
            [
                None,
                None,
                None,
                None,
                Some(Block::OneByOne),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(Block::TwoByTwo),
                Some(Block::TwoByTwo),
                None,
                None,
                Some(Block::TwoByTwo),
                Some(Block::TwoByTwo),
            ]
        );

        assert_eq!(
            board.move_block(1, 0, 1),
            Err(BoardError::BlockPlacementInvalid)
        );
        assert_eq!(
            board.move_block(1, 1, 0),
            Err(BoardError::BlockPlacementInvalid)
        );
        assert!(board.move_block(1, 0, -2).is_ok());
        assert!(board.move_block(1, -1, 1).is_ok());
        assert!(board.move_block(1, -1, 0).is_ok());
        assert_eq!(
            board.move_block(1, 0, -1),
            Err(BoardError::BlockPlacementInvalid)
        );

        assert_eq!(
            board.grid,
            [
                None,
                None,
                None,
                None,
                Some(Block::OneByOne),
                Some(Block::TwoByTwo),
                Some(Block::TwoByTwo),
                None,
                None,
                Some(Block::TwoByTwo),
                Some(Block::TwoByTwo),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ]
        );
    }

    #[test]
    #[should_panic]
    fn undo_move_unchecked() {
        let mut board = Board::default();

        board.undo_move_unchecked();
    }

    #[test]
    fn undo_move() {
        let mut board = Board::default();

        let block = PositionedBlock::new(Block::OneByOne, 2, 0).unwrap();
        board.update_grid_range(&block.range, Some(block.block));
        board.blocks.push(block);
        board.state = State::Solving;
        board.moves = vec![
            FlatBoardMove::new(0, &FlatMove::new(0, 1).unwrap()),
            FlatBoardMove::new(0, &FlatMove::new(1, 0).unwrap()),
            FlatBoardMove::new(0, &FlatMove::new(0, -1).unwrap()),
            FlatBoardMove::new(0, &FlatMove::new(1, 0).unwrap()),
        ];

        assert!(board.undo_move().is_ok());
        assert_eq!(board.moves.len(), 3);
        assert_eq!(
            board.grid,
            [
                None,
                None,
                None,
                None,
                Some(Block::OneByOne),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ]
        );

        assert!(board.undo_move().is_ok());
        assert_eq!(board.moves.len(), 2);
        assert_eq!(
            board.grid,
            [
                None,
                None,
                None,
                None,
                None,
                Some(Block::OneByOne),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ]
        );

        assert!(board.undo_move().is_ok());
        assert_eq!(board.moves.len(), 1);
        assert_eq!(
            board.grid,
            [
                None,
                Some(Block::OneByOne),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ]
        );

        assert!(board.undo_move().is_ok());
        assert_eq!(board.moves.len(), 0);
        assert_eq!(
            board.grid,
            [
                Some(Block::OneByOne),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ]
        );

        assert!(board.undo_move().is_err());
    }

    #[test]
    fn reset() {
        let mut board = Board::default();

        let block = PositionedBlock::new(Block::OneByOne, 2, 0).unwrap();
        board.update_grid_range(&block.range, Some(block.block));
        board.blocks.push(block);

        assert!(board.reset().is_err());

        board.state = State::Solving;
        board.moves = vec![
            FlatBoardMove::new(0, &FlatMove::new(0, 1).unwrap()),
            FlatBoardMove::new(0, &FlatMove::new(1, 0).unwrap()),
            FlatBoardMove::new(0, &FlatMove::new(0, -1).unwrap()),
            FlatBoardMove::new(0, &FlatMove::new(1, 0).unwrap()),
        ];

        assert!(board.reset().is_ok());
        assert_eq!(board.moves.len(), 0);
    }
}
