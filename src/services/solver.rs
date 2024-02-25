use std::collections::{HashSet, VecDeque};

use crate::errors::board::Error as BoardError;
use crate::models::game::{
    board::{Board, State as BoardState},
    moves::FlatBoardMove,
};

pub struct Solver {
    start_board: Board,
    seen: HashSet<u64>,
}

impl Solver {
    fn bfs(&mut self) -> Option<Board> {
        let mut queue: VecDeque<Board> = VecDeque::from([self.start_board.clone()]);

        while !queue.is_empty() {
            let queue_size = queue.len();

            for _ in 0..queue_size {
                let mut curr_board = queue.pop_front().unwrap();

                if curr_board.state == BoardState::Solved {
                    return Some(curr_board);
                }

                let next_moves = curr_board.get_next_moves();

                for (block_idx, moves) in next_moves.into_iter().enumerate() {
                    for move_ in moves {
                        curr_board.move_block_unchecked(block_idx, move_.row_diff, move_.col_diff);

                        let hash = curr_board.hash();

                        if !self.seen.contains(&hash) {
                            self.seen.insert(hash);

                            queue.push_back(curr_board.clone());
                        }

                        curr_board.undo_move().unwrap();
                    }
                }
            }
        }

        None
    }
}

impl Solver {
    pub fn new(board: &Board) -> Result<Self, BoardError> {
        let mut start_board = board.clone();

        start_board.change_state(&BoardState::Solving)?;

        start_board.moves.clear();

        let _board_is_already_solved = start_board.change_state(&BoardState::Solved).is_ok();

        Ok(Self {
            start_board,
            seen: HashSet::<u64>::new(),
        })
    }

    pub fn solve(&mut self) -> Option<Vec<FlatBoardMove>> {
        self.bfs().map(|solved_board| solved_board.moves)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::game::{
        blocks::{Block, Positioned as PositionedBlock},
        board::Board,
    };

    #[test]
    fn test_not_ready_board() {
        let mut board = Board::default();

        assert!(Solver::new(&mut board).is_err());
    }

    fn test_board_is_optimal(blocks: &[PositionedBlock], expected_moves: usize) {
        let mut board = Board::default();

        for block in blocks.iter() {
            board.add_block(block.clone()).unwrap();
        }

        let mut solver = Solver::new(&mut board).unwrap();
        let moves = solver.solve().unwrap();

        assert_eq!(moves.len(), expected_moves);
    }

    fn test_solution_works(blocks: &[PositionedBlock]) {
        let mut board = Board::default();

        for block in blocks.iter() {
            board.add_block(block.clone()).unwrap();
        }

        let mut solver = Solver::new(&mut board).unwrap();
        let moves = solver.solve().unwrap();

        for move_ in moves.iter() {
            board
                .move_block(move_.block_idx, move_.row_diff, move_.col_diff)
                .unwrap();
        }

        assert!(board.is_solved());
    }

    #[test]
    fn test_solved_board() {
        let blocks = [
            PositionedBlock::new(Block::OneByTwo, 0, 0).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 0, 2).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 1, 0).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 1, 2).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 2, 0).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 2, 2).unwrap(),
            PositionedBlock::new(Block::OneByOne, 3, 0).unwrap(),
            PositionedBlock::new(Block::TwoByTwo, 3, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 3, 3).unwrap(),
        ];

        test_board_is_optimal(&blocks, 0);
    }

    #[test]
    fn test_classic_board_solution_works() {
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

        test_solution_works(&blocks);
    }

    #[test]
    fn test_classic_board_is_optimal() {
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

        test_board_is_optimal(&blocks, 81);
    }

    #[test]
    fn test_easy_board_solution_works() {
        let blocks = [
            PositionedBlock::new(Block::OneByOne, 0, 0).unwrap(),
            PositionedBlock::new(Block::TwoByTwo, 0, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 0, 3).unwrap(),
            PositionedBlock::new(Block::OneByOne, 1, 0).unwrap(),
            PositionedBlock::new(Block::OneByOne, 1, 3).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 0).unwrap(),
            PositionedBlock::new(Block::OneByOne, 2, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 2, 2).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 3).unwrap(),
            PositionedBlock::new(Block::OneByOne, 3, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 3, 2).unwrap(),
            PositionedBlock::new(Block::OneByOne, 4, 0).unwrap(),
            PositionedBlock::new(Block::OneByOne, 4, 3).unwrap(),
        ];

        test_solution_works(&blocks);
    }

    #[test]
    fn test_easy_board_is_optimal() {
        let blocks = [
            PositionedBlock::new(Block::OneByOne, 0, 0).unwrap(),
            PositionedBlock::new(Block::TwoByTwo, 0, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 0, 3).unwrap(),
            PositionedBlock::new(Block::OneByOne, 1, 0).unwrap(),
            PositionedBlock::new(Block::OneByOne, 1, 3).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 0).unwrap(),
            PositionedBlock::new(Block::OneByOne, 2, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 2, 2).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 3).unwrap(),
            PositionedBlock::new(Block::OneByOne, 3, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 3, 2).unwrap(),
            PositionedBlock::new(Block::OneByOne, 4, 0).unwrap(),
            PositionedBlock::new(Block::OneByOne, 4, 3).unwrap(),
        ];

        test_board_is_optimal(&blocks, 17);
    }

    #[test]
    fn test_medium_board_solution_works() {
        let blocks = [
            PositionedBlock::new(Block::OneByOne, 0, 0).unwrap(),
            PositionedBlock::new(Block::TwoByTwo, 0, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 0, 3).unwrap(),
            PositionedBlock::new(Block::OneByOne, 1, 0).unwrap(),
            PositionedBlock::new(Block::OneByOne, 1, 3).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 0).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 1).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 2, 2).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 3, 2).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 4, 1).unwrap(),
        ];

        test_solution_works(&blocks);
    }

    #[test]
    fn test_medium_board_is_optimal() {
        let blocks = [
            PositionedBlock::new(Block::OneByOne, 0, 0).unwrap(),
            PositionedBlock::new(Block::TwoByTwo, 0, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 0, 3).unwrap(),
            PositionedBlock::new(Block::OneByOne, 1, 0).unwrap(),
            PositionedBlock::new(Block::OneByOne, 1, 3).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 0).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 2, 1).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 2, 2).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 3, 2).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 4, 1).unwrap(),
        ];

        test_board_is_optimal(&blocks, 40);
    }

    #[test]
    fn test_hard_board_solution_works() {
        let blocks = [
            PositionedBlock::new(Block::OneByOne, 0, 0).unwrap(),
            PositionedBlock::new(Block::TwoByTwo, 0, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 0, 3).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 1, 0).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 1, 3).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 2, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 3, 0).unwrap(),
            PositionedBlock::new(Block::OneByOne, 3, 3).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 3, 1).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 4, 1).unwrap(),
        ];

        test_solution_works(&blocks);
    }

    #[test]
    fn test_hard_board_is_optimal() {
        let blocks = [
            PositionedBlock::new(Block::OneByOne, 0, 0).unwrap(),
            PositionedBlock::new(Block::TwoByTwo, 0, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 0, 3).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 1, 0).unwrap(),
            PositionedBlock::new(Block::TwoByOne, 1, 3).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 2, 1).unwrap(),
            PositionedBlock::new(Block::OneByOne, 3, 0).unwrap(),
            PositionedBlock::new(Block::OneByOne, 3, 3).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 3, 1).unwrap(),
            PositionedBlock::new(Block::OneByTwo, 4, 1).unwrap(),
        ];

        test_board_is_optimal(&blocks, 120);
    }
}
