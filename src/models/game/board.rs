use serde::{Deserialize, Serialize};

use super::{
    blocks::Positioned as PositionedBlock,
    moves::{FlatBoardMove, FlatMove, Step},
};
use crate::{errors::board::Error as BoardError, models::game::utils::Position};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Building,
    ReadyToSolve,
    AlgoSolving,
    ManualSolving,
    Solved,
}

#[derive(Debug, Clone)]
pub struct Board {
    pub state: State,
    pub blocks: Vec<PositionedBlock>,
    pub filled: [[bool; Self::COLS as usize]; Self::ROWS as usize],
    pub moves: Vec<FlatBoardMove>,
    pub next_moves: Vec<Vec<FlatMove>>,
}

impl Default for Board {
    fn default() -> Self {
        Self::new(
            State::Building,
            vec![],
            [[false; Self::COLS as usize]; Self::ROWS as usize],
            vec![],
            vec![],
        )
    }
}

impl Board {
    pub const ROWS: u8 = 5;
    pub const COLS: u8 = 4;
    pub const NUM_EMPTY_CELLS: u8 = 2;

    const WINNING_BLOCK_ID: u8 = 4;
    const WINNING_ROW: u8 = 3;
    const WINNING_COL: u8 = 1;

    fn is_ready_to_solve(&self) -> bool {
        let num_winning_blocks = self.blocks.iter().fold(0, |acc, curr| {
            acc + u8::from(curr.block_id == Self::WINNING_BLOCK_ID)
        });

        if num_winning_blocks != 1 {
            return false;
        }

        let empty_cells = self.filled.iter().fold(0, |acc, row| {
            acc + row
                .iter()
                .fold(0, |acc, &is_filled| acc + u8::from(!is_filled))
        });

        empty_cells == Self::NUM_EMPTY_CELLS
    }

    fn updated_filled_range(&mut self, range: &Vec<(u8, u8)>, value: bool) {
        for (i, j) in range {
            self.filled[usize::from(*i)][usize::from(*j)] = value;
        }
    }

    fn is_range_empty(&self, range: &Vec<(u8, u8)>) -> bool {
        for (i, j) in range {
            if self.filled[usize::from(*i)][usize::from(*j)] {
                return false;
            }
        }

        true
    }

    fn is_step_valid_for_block(&self, block: &PositionedBlock, step: &Step) -> bool {
        match step {
            Step::Up => (block.min_position.col..=block.max_position.col).all(|col| {
                u8::try_from(i8::try_from(block.min_position.row).unwrap() - 1)
                    .ok()
                    .is_some_and(|row_above| {
                        Position::new(row_above, col).is_some_and(|new_position| {
                            !self.filled[usize::from(new_position.row)][usize::from(col)]
                        })
                    })
            }),
            Step::Down => (block.min_position.col..=block.max_position.col).all(|col| {
                Position::new(block.max_position.row + 1, col).is_some_and(|new_position| {
                    !self.filled[usize::from(new_position.row)][usize::from(col)]
                })
            }),
            Step::Left => (block.min_position.row..=block.max_position.row).all(|row| {
                u8::try_from(i8::try_from(block.min_position.col).unwrap() - 1)
                    .ok()
                    .is_some_and(|col_above| {
                        Position::new(row, col_above).is_some_and(|new_position| {
                            !self.filled[usize::from(row)][usize::from(new_position.col)]
                        })
                    })
            }),
            Step::Right => (block.min_position.row..=block.max_position.row).all(|row| {
                Position::new(row, block.max_position.col + 1).is_some_and(|new_position| {
                    !self.filled[usize::from(row)][usize::from(new_position.col)]
                })
            }),
        }
    }

    fn get_next_moves_for_block(&self, block: &PositionedBlock) -> Vec<FlatMove> {
        let mut moves = vec![vec![]];

        let mut block = block.clone();

        for depth in 0..Self::NUM_EMPTY_CELLS {
            for i in 0..moves.len() {
                for step in &moves[i] {
                    block.do_step(step).unwrap();
                }

                for step in &Step::ALL {
                    if self.is_step_valid_for_block(&block, step) && block.do_step(step).is_ok() {
                        let mut new_move = moves[i].clone();
                        new_move.push(step.clone());

                        moves.push(new_move);

                        block.undo_step(step).unwrap();
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

    fn update_next_moves(&mut self) {
        self.next_moves = self
            .blocks
            .iter()
            .map(|block| {
                let mut moves = self.get_next_moves_for_block(block);
                moves.dedup();
                moves
            })
            .collect();
    }
}

impl Board {
    pub fn new(
        state: State,
        blocks: Vec<PositionedBlock>,
        filled: [[bool; Self::COLS as usize]; Self::ROWS as usize],
        moves: Vec<FlatBoardMove>,
        next_moves: Vec<Vec<FlatMove>>,
    ) -> Self {
        Self {
            state,
            blocks,
            filled,
            moves,
            next_moves,
        }
    }

    pub fn hash(&self) -> String {
        let mut block_id_matrix = [[0u8; Self::COLS as usize]; Self::ROWS as usize];

        for block in &self.blocks {
            for (i, j) in &block.range {
                block_id_matrix[usize::from(*i)][usize::from(*j)] = block.block_id;
            }
        }

        block_id_matrix
            .into_iter()
            .flat_map(|row| {
                row.into_iter()
                    .map(|cell| char::from_digit(u32::from(cell), 10).unwrap())
            })
            .collect()
    }

    pub fn change_state(&mut self, new_state: &State) -> Result<(), BoardError> {
        if &self.state == new_state {
            return Ok(());
        }

        match (&self.state, new_state) {
            (State::Building, State::ReadyToSolve) => {
                if !self.is_ready_to_solve() {
                    return Err(BoardError::BoardStateInvalid);
                }
            }
            (State::ReadyToSolve, State::Building | State::ManualSolving | State::AlgoSolving) => {}
            (State::ManualSolving | State::AlgoSolving, State::ReadyToSolve) => {
                if !self.moves.is_empty() {
                    return Err(BoardError::BoardStateInvalid);
                }
            }
            (State::ManualSolving | State::AlgoSolving, State::Solved) => {
                if !self.is_solved() {
                    return Err(BoardError::BoardStateInvalid);
                }
            }
            (State::Solved, State::AlgoSolving | State::ManualSolving) => {
                if self.is_solved() {
                    return Err(BoardError::BoardStateInvalid);
                }
            }
            _ => {
                return Err(BoardError::BoardStateInvalid);
            }
        }

        self.state = new_state.clone();

        Ok(())
    }

    pub fn is_solved(&self) -> bool {
        self.blocks.iter().any(|block| {
            block.block_id == Self::WINNING_BLOCK_ID
                && block.min_position.row == Self::WINNING_ROW
                && block.min_position.col == Self::WINNING_COL
        })
    }

    pub fn add_block(&mut self, block: PositionedBlock) -> Result<(), BoardError> {
        if self.state != State::Building {
            self.change_state(&State::Building)?;
        }

        if !self.is_range_empty(&block.range) {
            return Err(BoardError::BlockPlacementInvalid);
        }

        self.updated_filled_range(&block.range, true);

        self.blocks.push(block);

        if self.change_state(&State::ReadyToSolve).is_ok() {
            self.update_next_moves();
        }

        Ok(())
    }

    pub fn change_block(&mut self, block_idx: u8, new_block_id: u8) -> Result<(), BoardError> {
        if self.state != State::Building {
            self.change_state(&State::Building)?;
        }

        let block_idx = usize::from(block_idx);

        let block = self
            .blocks
            .get(block_idx)
            .cloned()
            .ok_or(BoardError::BlockIndexOutOfBounds)?;

        if block.block_id == new_block_id {
            return Ok(());
        }

        let new_block =
            PositionedBlock::new(new_block_id, block.min_position.row, block.min_position.col)
                .ok_or(BoardError::BlockPlacementInvalid)?;

        self.updated_filled_range(&block.range, false);

        if !self.is_range_empty(&new_block.range) {
            self.updated_filled_range(&block.range, true);

            return Err(BoardError::BlockPlacementInvalid);
        }

        self.updated_filled_range(&new_block.range, true);

        self.blocks[block_idx] = new_block;

        if self.change_state(&State::ReadyToSolve).is_ok() {
            self.update_next_moves();
        }

        Ok(())
    }

    pub fn remove_block(&mut self, block_idx: u8) -> Result<(), BoardError> {
        if self.state != State::Building {
            return Err(BoardError::BoardStateInvalid);
        }

        let block_idx = usize::from(block_idx);

        let block = self
            .blocks
            .get(block_idx)
            .cloned()
            .ok_or(BoardError::BlockIndexOutOfBounds)?;

        self.updated_filled_range(&block.range, false);

        self.blocks.swap_remove(block_idx);

        if self.state == State::ReadyToSolve && self.change_state(&State::Building).is_ok() {
            self.next_moves = vec![];
        }

        Ok(())
    }

    pub fn move_block(
        &mut self,
        block_idx: u8,
        row_diff: i8,
        col_diff: i8,
    ) -> Result<(), BoardError> {
        if self.state != State::ManualSolving && self.state != State::AlgoSolving {
            self.change_state(&State::ManualSolving)?;
        }

        let block_idx_usize = usize::from(block_idx);

        if self.state == State::ManualSolving {
            let is_valid_move = self
                .next_moves
                .get(block_idx_usize)
                .unwrap()
                .iter()
                .any(|move_| move_.row_diff == row_diff && move_.col_diff == col_diff);

            if !is_valid_move {
                return Err(BoardError::BlockPlacementInvalid);
            }
        }

        let mut block = self
            .blocks
            .get(block_idx_usize)
            .cloned()
            .ok_or(BoardError::BlockIndexOutOfBounds)?;

        self.updated_filled_range(&block.range, false);

        if block.move_by(row_diff, col_diff).is_err() {
            self.updated_filled_range(&block.range, true);

            return Err(BoardError::BlockPlacementInvalid);
        };

        self.updated_filled_range(&block.range, true);

        self.blocks[block_idx_usize] = block;

        self.moves.push(FlatBoardMove::new(
            block_idx,
            &FlatMove::new(row_diff, col_diff).unwrap(),
        ));

        if self.change_state(&State::Solved).is_ok() {
            self.next_moves = vec![];
        } else {
            self.update_next_moves();
        }

        Ok(())
    }

    pub fn undo_move(&mut self) -> Result<(), BoardError> {
        if self.state != State::ManualSolving && self.state != State::Solved {
            return Err(BoardError::BoardStateInvalid);
        }

        let opposite_move = self
            .moves
            .pop()
            .ok_or(BoardError::NoMovesToUndo)?
            .opposite();

        let block_idx = usize::from(opposite_move.block_idx);

        let mut block = self
            .blocks
            .get(block_idx)
            .cloned()
            .ok_or(BoardError::BlockIndexOutOfBounds)?;

        self.updated_filled_range(&block.range, false);

        if block
            .move_by(opposite_move.row_diff, opposite_move.col_diff)
            .is_err()
        {
            self.updated_filled_range(&block.range, true);

            return Err(BoardError::BlockPlacementInvalid);
        }

        self.updated_filled_range(&block.range, true);

        self.blocks[block_idx] = block;

        if self.state == State::Solved {
            let _board_is_no_longer_solved = self.change_state(&State::ManualSolving).is_ok();
        }

        self.update_next_moves();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updated_filled_range() {
        let mut board = Board::default();

        let block = PositionedBlock::new(1, 0, 0).unwrap();
        board.updated_filled_range(&block.range, true);

        assert!(board.filled[0][0]);

        board.updated_filled_range(&block.range, false);

        assert!(!board.filled[0][0]);
    }

    #[test]
    fn is_range_empty() {
        let mut board = Board::default();

        let block_one = PositionedBlock::new(1, 0, 0).unwrap();
        board.updated_filled_range(&block_one.range, true);

        let block_two = PositionedBlock::new(2, 1, 0).unwrap();

        assert!(!board.is_range_empty(&block_one.range));
        assert!(board.is_range_empty(&block_two.range));
    }

    #[test]
    fn is_step_valid_for_block() {
        let mut board = Board::default();

        let block_one = PositionedBlock::new(1, 0, 0).unwrap();
        board.updated_filled_range(&block_one.range, true);

        let block_two = PositionedBlock::new(2, 0, 1).unwrap();
        board.updated_filled_range(&block_two.range, true);

        assert!(!board.is_step_valid_for_block(&block_one, &Step::Left));
        assert!(!board.is_step_valid_for_block(&block_one, &Step::Right));
        assert!(!board.is_step_valid_for_block(&block_one, &Step::Up));
        assert!(board.is_step_valid_for_block(&block_one, &Step::Down));

        let block_three = PositionedBlock::new(1, 1, 0).unwrap();
        board.updated_filled_range(&block_three.range, true);

        assert!(!board.is_step_valid_for_block(&block_one, &Step::Down));

        assert_eq!(
            board.filled,
            [
                [true, true, true, false],
                [true, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
            ]
        )
    }

    #[test]
    fn get_next_moves_for_block_down_right() {
        let mut board = Board::default();

        let block_one = PositionedBlock::new(1, 0, 0).unwrap();
        board.updated_filled_range(&block_one.range, true);

        let block_two = PositionedBlock::new(1, 0, 1).unwrap();
        board.updated_filled_range(&block_two.range, true);

        let block_three = PositionedBlock::new(1, 1, 0).unwrap();
        board.updated_filled_range(&block_three.range, true);

        let block_one_moves = board.get_next_moves_for_block(&block_one);

        assert_eq!(block_one_moves.len(), 0);

        let block_two_moves = board.get_next_moves_for_block(&block_two);

        assert_eq!(
            board.filled,
            [
                [true, true, false, false],
                [true, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
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

        assert_eq!(block_two_moves.len(), expected_block_two_moves.len());

        for move_ in block_two_moves {
            assert!(expected_block_two_moves.contains(&move_));
        }
    }

    #[test]
    fn get_next_moves_for_block_up_left() {
        let mut board = Board::default();

        let block_one = PositionedBlock::new(1, 4, 3).unwrap();
        board.updated_filled_range(&block_one.range, true);

        let block_two = PositionedBlock::new(1, 4, 2).unwrap();
        board.updated_filled_range(&block_two.range, true);

        let block_three = PositionedBlock::new(1, 3, 3).unwrap();
        board.updated_filled_range(&block_three.range, true);

        let block_one_moves = board.get_next_moves_for_block(&block_one);

        assert_eq!(block_one_moves.len(), 0);

        let block_two_moves = board.get_next_moves_for_block(&block_two);

        assert_eq!(
            board.filled,
            [
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, true],
                [false, false, true, true],
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
    fn update_next_moves() {
        let blocks = vec![
            PositionedBlock::new(3, 0, 0).unwrap(),
            PositionedBlock::new(4, 0, 1).unwrap(),
            PositionedBlock::new(3, 0, 3).unwrap(),
            PositionedBlock::new(3, 2, 0).unwrap(),
            PositionedBlock::new(2, 2, 1).unwrap(),
            PositionedBlock::new(3, 2, 3).unwrap(),
            PositionedBlock::new(1, 3, 1).unwrap(),
            PositionedBlock::new(1, 3, 2).unwrap(),
            PositionedBlock::new(1, 4, 0).unwrap(),
            PositionedBlock::new(1, 4, 3).unwrap(),
        ];

        let mut board = Board::default();
        for block in blocks {
            board.add_block(block).unwrap();
        }

        assert_eq!(board.state, State::ReadyToSolve);

        assert!(board.change_state(&State::ManualSolving).is_ok());

        assert_eq!(
            board
                .next_moves
                .iter()
                .fold(0, |acc, moves| acc + moves.len()),
            8
        );

        let expected_moves = [
            vec![FlatMove::new(1, 0).unwrap(), FlatMove::new(1, 1).unwrap()],
            vec![FlatMove::new(1, 0).unwrap(), FlatMove::new(1, -1).unwrap()],
            vec![FlatMove::new(0, 1).unwrap(), FlatMove::new(0, 2).unwrap()],
            vec![FlatMove::new(0, -1).unwrap(), FlatMove::new(0, -2).unwrap()],
        ];

        for i in 0..board.next_moves.len() {
            if i < 6 {
                assert_eq!(board.next_moves[i].len(), 0);
            } else {
                assert_eq!(board.next_moves[i], expected_moves[i - 6]);
            }
        }
    }

    #[test]
    fn hash() {
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
            PositionedBlock::new(1, 4, 3).unwrap(),
        ];

        for block in blocks.iter() {
            board.updated_filled_range(&block.range, true);
            board.blocks.push(block.clone());
        }

        assert_eq!(board.hash(), String::from("34433443322332231001"),);
    }

    #[test]
    fn change_state() {
        let mut board = Board::default();

        assert!(board.change_state(&State::Building).is_ok());
        assert!(board.change_state(&State::AlgoSolving).is_err());
        assert!(board.change_state(&State::ManualSolving).is_err());

        let blocks = [
            PositionedBlock::new(3, 0, 0).unwrap(),
            PositionedBlock::new(3, 0, 3).unwrap(),
            PositionedBlock::new(4, 0, 1).unwrap(),
            PositionedBlock::new(3, 2, 0).unwrap(),
            PositionedBlock::new(2, 2, 1).unwrap(),
            PositionedBlock::new(3, 2, 3).unwrap(),
            PositionedBlock::new(2, 3, 1).unwrap(),
            PositionedBlock::new(1, 4, 0).unwrap(),
            PositionedBlock::new(1, 4, 3).unwrap(),
        ];

        for block in blocks.iter() {
            board.updated_filled_range(&block.range, true);
            board.blocks.push(block.clone());
        }

        assert!(board.change_state(&State::AlgoSolving).is_err());
        assert!(board.change_state(&State::ReadyToSolve).is_ok());
        assert!(board.change_state(&State::Building).is_ok());
        assert!(board.change_state(&State::ManualSolving).is_err());
        assert!(board.change_state(&State::ReadyToSolve).is_ok());
        assert!(board.change_state(&State::ManualSolving).is_ok());

        let move_ = FlatBoardMove::new(0, &FlatMove::new(1, 0).unwrap());
        board.moves.push(move_);

        assert!(board.change_state(&State::ReadyToSolve).is_err());
        assert!(board.change_state(&State::Building).is_err());
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
            board.updated_filled_range(&block.range, true);
            board.blocks.push(block.clone());

            assert!(!board.is_ready_to_solve());
        }

        board.updated_filled_range(&final_block.range, true);
        board.blocks.push(final_block);

        assert!(board.is_ready_to_solve());
    }

    #[test]
    fn is_solved() {
        let mut board = Board::default();
        let mut block = PositionedBlock::new(4, 2, 1).unwrap();
        board.blocks.push(block.clone());

        assert!(!board.is_solved());

        block.do_step(&Step::Down).unwrap();
        board.blocks[0] = block;

        assert!(board.is_solved())
    }

    #[test]
    fn add_block() {
        let mut board = Board::default();

        let block_one = PositionedBlock::new(2, 0, 0).unwrap();

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

        let block_two = PositionedBlock::new(2, 0, 1).unwrap();

        assert!(board.add_block(block_two).is_err());
    }

    #[test]
    fn remove_block() {
        let mut board = Board::default();

        let block_one = PositionedBlock::new(2, 0, 0).unwrap();
        board.updated_filled_range(&block_one.range, true);
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
        board.updated_filled_range(&block.range, true);
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
        board.updated_filled_range(&block_one.range, true);
        board.blocks.push(block_one);
        board.state = State::ManualSolving;
        board.next_moves = vec![vec![
            FlatMove::new(0, 1).unwrap(),
            FlatMove::new(0, 2).unwrap(),
            FlatMove::new(1, 1).unwrap(),
            FlatMove::new(1, 0).unwrap(),
            FlatMove::new(2, 0).unwrap(),
        ]];

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

        let block_two = PositionedBlock::new(4, 3, 2).unwrap();
        board.updated_filled_range(&block_two.range, true);
        board.blocks.push(block_two);
        board.next_moves.push(vec![
            FlatMove::new(-1, 0).unwrap(),
            FlatMove::new(-2, 0).unwrap(),
            FlatMove::new(-1, -1).unwrap(),
            FlatMove::new(0, -1).unwrap(),
            FlatMove::new(0, -2).unwrap(),
        ]);

        assert_eq!(
            board.filled,
            [
                [false, false, false, false],
                [true, false, false, false],
                [false, false, false, false],
                [false, false, true, true],
                [false, false, true, true],
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
            board.filled,
            [
                [false, false, false, false],
                [true, true, true, false],
                [false, true, true, false],
                [false, false, false, false],
                [false, false, false, false],
            ]
        );
    }

    #[test]
    fn undo_move() {
        let mut board = Board::default();

        let block = PositionedBlock::new(1, 2, 0).unwrap();
        board.updated_filled_range(&block.range, true);
        board.blocks.push(block);
        board.state = State::ManualSolving;
        board.moves = vec![
            FlatBoardMove::new(0, &FlatMove::new(0, 1).unwrap()),
            FlatBoardMove::new(0, &FlatMove::new(1, 0).unwrap()),
            FlatBoardMove::new(0, &FlatMove::new(0, -1).unwrap()),
            FlatBoardMove::new(0, &FlatMove::new(1, 0).unwrap()),
        ];
        board.next_moves = vec![vec![
            FlatMove::new(-1, 0).unwrap(),
            FlatMove::new(-2, 0).unwrap(),
            FlatMove::new(1, 0).unwrap(),
            FlatMove::new(2, 0).unwrap(),
            FlatMove::new(0, 1).unwrap(),
            FlatMove::new(0, 2).unwrap(),
            FlatMove::new(-1, 1).unwrap(),
            FlatMove::new(1, 1).unwrap(),
        ]];

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
}
