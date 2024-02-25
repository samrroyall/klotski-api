use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::errors::board::Error as BoardError;
use crate::models::game::{
    board::{Board, State as BoardState},
    moves::FlatBoardMove,
};

pub struct Solver {
    start_board: Board,
    seen: Arc<Mutex<HashSet<u64>>>,
}

impl Solver {
    const NUM_THREADS: usize = 4;

    fn process_sub_level(
        batch_size: usize,
        queue: &Arc<Mutex<VecDeque<Board>>>,
        seen: &Arc<Mutex<HashSet<u64>>>,
    ) -> Option<Board> {
        for _ in 0..batch_size {
            let mut board = queue.lock().unwrap().pop_front().unwrap();

            if board.state == BoardState::Solved {
                return Some(board);
            }

            let next_moves = board.get_next_moves();

            for (block_idx, moves) in next_moves.into_iter().enumerate() {
                for move_ in moves {
                    board.move_block_unchecked(block_idx, move_.row_diff, move_.col_diff);

                    if seen.lock().unwrap().insert(board.hash()) {
                        queue.lock().unwrap().push_back(board.clone());
                    }

                    board.undo_move_unchecked();
                }
            }
        }

        None
    }

    fn parallel_bfs(&mut self) -> Option<Board> {
        let root = self.start_board.clone();

        if root.state == BoardState::Solved {
            return Some(root);
        }

        let queue: Arc<Mutex<VecDeque<Board>>> = Arc::new(Mutex::new(VecDeque::new()));

        queue.lock().unwrap().push_back(root);

        while !queue.lock().unwrap().is_empty() {
            let mut level_size = queue.lock().unwrap().len();

            let batch_size = (level_size + Self::NUM_THREADS - 1) / Self::NUM_THREADS;

            let mut handles = vec![];

            for _ in 0..Self::NUM_THREADS {
                let curr_batch_size = batch_size.min(level_size);

                let queue_clone = Arc::clone(&queue);
                let seen_clone = Arc::clone(&self.seen);

                let handle = thread::spawn(move || {
                    Solver::process_sub_level(curr_batch_size, &queue_clone, &seen_clone)
                });

                level_size -= curr_batch_size;

                handles.push(handle);
            }

            for handle in handles {
                if let Some(solved_board) = handle.join().unwrap() {
                    return Some(solved_board);
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
            seen: Arc::new(Mutex::new(HashSet::<u64>::new())),
        })
    }

    pub fn solve(&mut self) -> Option<Vec<FlatBoardMove>> {
        self.parallel_bfs().map(|solved_board| solved_board.moves)
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
