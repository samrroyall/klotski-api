use std::collections::{HashSet, VecDeque};

use crate::errors::board::Error as BoardError;
use crate::models::game::{
    board::{Board, State as BoardState},
    moves::FlatBoardMove,
};

pub struct Solver {
    start_board: Board,
    seen: HashSet<String>,
}

impl Solver {
    fn bfs(&mut self) -> Result<Option<Board>, BoardError> {
        let root = self.start_board.clone();

        if root.is_solved() {
            return Ok(Some(root));
        }

        let mut queue: VecDeque<Board> = VecDeque::from([root]);

        while !queue.is_empty() {
            let queue_size = queue.len();

            for _ in 0..queue_size {
                let curr_board = queue.pop_front().unwrap();

                for (block_idx, moves) in curr_board.next_moves.iter().enumerate() {
                    for move_ in moves {
                        let mut child_board = curr_board.clone();

                        child_board.move_block(block_idx, move_.row_diff, move_.col_diff)?;

                        if child_board.state == BoardState::Solved {
                            return Ok(Some(child_board));
                        }

                        let hash = child_board.hash();

                        if self.seen.contains(&hash) {
                            continue;
                        }

                        self.seen.insert(hash);

                        queue.push_back(child_board);
                    }
                }
            }
        }

        Ok(None)
    }
}

impl Solver {
    pub fn new(start_board: &mut Board) -> Result<Self, BoardError> {
        start_board.change_state(&BoardState::Solving)?;

        Ok(Self {
            start_board: start_board.clone(),
            seen: HashSet::<String>::new(),
        })
    }

    pub fn solve(&mut self) -> Result<Option<Vec<FlatBoardMove>>, BoardError> {
        let maybe_solved_board = self.bfs()?;

        if let Some(solved_board) = maybe_solved_board {
            return Ok(Some(solved_board.moves));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::game::{blocks::Positioned as PositionedBlock, board::Board};

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
        let moves = solver.solve().unwrap().unwrap();

        assert_eq!(moves.len(), expected_moves);
    }

    fn test_solution_works(blocks: &[PositionedBlock]) {
        let mut board = Board::default();

        for block in blocks.iter() {
            board.add_block(block.clone()).unwrap();
        }

        let mut solver = Solver::new(&mut board).unwrap();
        let moves = solver.solve().unwrap().unwrap();

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
            PositionedBlock::new(2, 0, 0).unwrap(),
            PositionedBlock::new(2, 0, 2).unwrap(),
            PositionedBlock::new(2, 1, 0).unwrap(),
            PositionedBlock::new(2, 1, 2).unwrap(),
            PositionedBlock::new(2, 2, 0).unwrap(),
            PositionedBlock::new(2, 2, 2).unwrap(),
            PositionedBlock::new(1, 3, 0).unwrap(),
            PositionedBlock::new(4, 3, 1).unwrap(),
            PositionedBlock::new(1, 3, 3).unwrap(),
        ];

        test_board_is_optimal(&blocks, 0);
    }

    #[test]
    fn test_classic_board_solution_works() {
        let blocks = [
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

        test_solution_works(&blocks);
    }

    #[test]
    fn test_classic_board_is_optimal() {
        let blocks = [
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

        test_board_is_optimal(&blocks, 81);
    }

    #[test]
    fn test_easy_board_solution_works() {
        let blocks = [
            PositionedBlock::new(1, 0, 0).unwrap(),
            PositionedBlock::new(4, 0, 1).unwrap(),
            PositionedBlock::new(1, 0, 3).unwrap(),
            PositionedBlock::new(1, 1, 0).unwrap(),
            PositionedBlock::new(1, 1, 3).unwrap(),
            PositionedBlock::new(3, 2, 0).unwrap(),
            PositionedBlock::new(1, 2, 1).unwrap(),
            PositionedBlock::new(1, 2, 2).unwrap(),
            PositionedBlock::new(3, 2, 3).unwrap(),
            PositionedBlock::new(1, 3, 1).unwrap(),
            PositionedBlock::new(1, 3, 2).unwrap(),
            PositionedBlock::new(1, 4, 0).unwrap(),
            PositionedBlock::new(1, 4, 3).unwrap(),
        ];

        test_solution_works(&blocks);
    }

    #[test]
    fn test_easy_board_is_optimal() {
        let blocks = [
            PositionedBlock::new(1, 0, 0).unwrap(),
            PositionedBlock::new(4, 0, 1).unwrap(),
            PositionedBlock::new(1, 0, 3).unwrap(),
            PositionedBlock::new(1, 1, 0).unwrap(),
            PositionedBlock::new(1, 1, 3).unwrap(),
            PositionedBlock::new(3, 2, 0).unwrap(),
            PositionedBlock::new(1, 2, 1).unwrap(),
            PositionedBlock::new(1, 2, 2).unwrap(),
            PositionedBlock::new(3, 2, 3).unwrap(),
            PositionedBlock::new(1, 3, 1).unwrap(),
            PositionedBlock::new(1, 3, 2).unwrap(),
            PositionedBlock::new(1, 4, 0).unwrap(),
            PositionedBlock::new(1, 4, 3).unwrap(),
        ];

        test_board_is_optimal(&blocks, 17);
    }

    #[test]
    fn test_medium_board_solution_works() {
        let blocks = [
            PositionedBlock::new(1, 0, 0).unwrap(),
            PositionedBlock::new(4, 0, 1).unwrap(),
            PositionedBlock::new(1, 0, 3).unwrap(),
            PositionedBlock::new(1, 1, 0).unwrap(),
            PositionedBlock::new(1, 1, 3).unwrap(),
            PositionedBlock::new(3, 2, 0).unwrap(),
            PositionedBlock::new(3, 2, 1).unwrap(),
            PositionedBlock::new(2, 2, 2).unwrap(),
            PositionedBlock::new(2, 3, 2).unwrap(),
            PositionedBlock::new(2, 4, 1).unwrap(),
        ];

        test_solution_works(&blocks);
    }

    #[test]
    fn test_medium_board_is_optimal() {
        let blocks = [
            PositionedBlock::new(1, 0, 0).unwrap(),
            PositionedBlock::new(4, 0, 1).unwrap(),
            PositionedBlock::new(1, 0, 3).unwrap(),
            PositionedBlock::new(1, 1, 0).unwrap(),
            PositionedBlock::new(1, 1, 3).unwrap(),
            PositionedBlock::new(3, 2, 0).unwrap(),
            PositionedBlock::new(3, 2, 1).unwrap(),
            PositionedBlock::new(2, 2, 2).unwrap(),
            PositionedBlock::new(2, 3, 2).unwrap(),
            PositionedBlock::new(2, 4, 1).unwrap(),
        ];

        test_board_is_optimal(&blocks, 40);
    }

    #[test]
    fn test_hard_board_solution_works() {
        let blocks = [
            PositionedBlock::new(1, 0, 0).unwrap(),
            PositionedBlock::new(4, 0, 1).unwrap(),
            PositionedBlock::new(1, 0, 3).unwrap(),
            PositionedBlock::new(3, 1, 0).unwrap(),
            PositionedBlock::new(3, 1, 3).unwrap(),
            PositionedBlock::new(2, 2, 1).unwrap(),
            PositionedBlock::new(1, 3, 0).unwrap(),
            PositionedBlock::new(1, 3, 3).unwrap(),
            PositionedBlock::new(2, 3, 1).unwrap(),
            PositionedBlock::new(2, 4, 1).unwrap(),
        ];

        test_solution_works(&blocks);
    }

    #[test]
    fn test_hard_board_is_optimal() {
        let blocks = [
            PositionedBlock::new(1, 0, 0).unwrap(),
            PositionedBlock::new(4, 0, 1).unwrap(),
            PositionedBlock::new(1, 0, 3).unwrap(),
            PositionedBlock::new(3, 1, 0).unwrap(),
            PositionedBlock::new(3, 1, 3).unwrap(),
            PositionedBlock::new(2, 2, 1).unwrap(),
            PositionedBlock::new(1, 3, 0).unwrap(),
            PositionedBlock::new(1, 3, 3).unwrap(),
            PositionedBlock::new(2, 3, 1).unwrap(),
            PositionedBlock::new(2, 4, 1).unwrap(),
        ];

        test_board_is_optimal(&blocks, 120);
    }
}
