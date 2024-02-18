use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

use crate::errors::board::Error as BoardError;
use crate::models::game::{
    board::{Board, State as BoardState},
    moves::FlatBoardMove,
};

type Edge = Rc<RefCell<TreeNode>>;

#[derive(Debug, Clone)]
struct TreeNode {
    parent: Option<Edge>,
    move_: FlatBoardMove,
    board: Board,
}

impl TreeNode {
    pub fn new(parent: Option<Edge>, move_: FlatBoardMove, board: Board) -> Self {
        Self {
            parent,
            move_,
            board,
        }
    }

    pub fn from_board(board: Board) -> Self {
        Self {
            parent: None,
            move_: FlatBoardMove::default(),
            board,
        }
    }
}

pub struct Solver {
    start_board: Board,
    seen: HashSet<String>,
}

impl Solver {
    fn upsert_hash(&mut self, board_hash: String) -> bool {
        if self.seen.contains(&board_hash) {
            false
        } else {
            self.seen.insert(board_hash);
            true
        }
    }

    fn get_children(&mut self, parent_node: &Rc<RefCell<TreeNode>>) -> Vec<TreeNode> {
        let mut children = vec![];

        let board = parent_node.borrow().board.clone();

        for (block_idx, moves) in (0u8..).zip(board.get_next_moves()) {
            for move_ in moves {
                let child_move = FlatBoardMove::new(block_idx, &move_);

                if child_move.is_opposite(&parent_node.borrow().move_) {
                    continue;
                }

                let mut child_board = board.clone();

                child_board
                    .move_block_unchecked(block_idx, move_.row_diff, move_.col_diff)
                    .unwrap();

                if !self.upsert_hash(child_board.hash()) {
                    continue;
                }

                children.push(TreeNode::new(
                    Some(parent_node.clone()),
                    child_move,
                    child_board,
                ));
            }
        }

        children
    }

    fn bfs(&mut self, root: TreeNode) -> Option<Rc<RefCell<TreeNode>>> {
        let root_cell = Rc::new(RefCell::new(root));

        self.seen.insert(root_cell.borrow().board.hash());

        let mut queue = VecDeque::from([root_cell]);

        while !queue.is_empty() {
            let queue_size = queue.len();

            for _ in 0..queue_size {
                let node = queue.pop_front().unwrap();

                if node.borrow().board.is_solved() {
                    return Some(node.clone());
                }

                for child in self.get_children(&node) {
                    queue.push_back(Rc::new(RefCell::new(child)));
                }
            }
        }

        None
    }
}

impl Solver {
    pub fn new(start_board: &mut Board) -> Result<Self, BoardError> {
        if start_board.state != BoardState::AlgoSolving {
            if start_board.is_ready_to_solve() {
                start_board.state = BoardState::AlgoSolving;
            } else {
                return Err(BoardError::BoardStateInvalid);
            }
        }

        Ok(Self {
            start_board: start_board.clone(),
            seen: HashSet::<String>::new(),
        })
    }

    pub fn solve(&mut self) -> Option<Vec<FlatBoardMove>> {
        let root_node = TreeNode::from_board(self.start_board.clone());

        self.bfs(root_node).map(|tail_node| {
            let mut moves = vec![];

            let mut maybe_node = Some(tail_node);

            while let Some(node) = maybe_node {
                let mut node = node.borrow_mut();

                if node.parent.is_some() {
                    moves.push(node.move_.clone());
                }

                maybe_node = node.parent.take();
            }

            moves.into_iter().rev().collect()
        })
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

    #[test]
    fn test_solved_board() {
        let mut board = Board::default();

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

        for block in blocks {
            board.add_block(block).unwrap();
        }

        assert!(Solver::new(&mut board).is_err());
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
                .move_block_unchecked(move_.block_idx, move_.row_diff, move_.col_diff)
                .unwrap();
        }

        assert!(board.is_solved());
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
