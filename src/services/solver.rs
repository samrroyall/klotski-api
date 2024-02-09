use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

use crate::errors::game::BoardError;
use crate::models::game::board::{Board, BoardMove};

#[derive(Debug, Clone)]
struct TreeNode {
    parent: Option<Rc<Self>>,
    board_move: Option<BoardMove>,
    board: Board,
}

impl TreeNode {
    pub fn new(parent: Option<Rc<Self>>, board_move: Option<BoardMove>, board: Board) -> Self {
        Self {
            parent,
            board_move,
            board,
        }
    }

    pub fn board_move(&self) -> Option<&BoardMove> {
        self.board_move.as_ref()
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn parent(&self) -> Option<Rc<Self>> {
        if let Some(tree_node) = self.parent.as_ref() {
            return Some(Rc::clone(tree_node));
        }

        None
    }

    pub fn collect(&self) -> Vec<BoardMove> {
        let mut moves = vec![];

        let mut curr_node = Some(Rc::new(self.clone()));

        while let Some(node) = curr_node.as_ref() {
            if let Some(board_move) = node.board_move() {
                moves.push(board_move.clone());
                curr_node = node.parent();
                continue;
            }

            break;
        }

        moves.into_iter().rev().collect()
    }
}

pub struct Solver {
    start_board: Board,
    seen: HashSet<String>,
}

impl Solver {
    fn get_moves(&self, board: &Board, parent_node: &Rc<TreeNode>) -> Vec<TreeNode> {
        let mut children = vec![];

        for (block_idx, moves) in board.get_next_moves().into_iter().enumerate() {
            for move_ in moves.into_iter() {
                let curr_move = BoardMove::new(block_idx, move_);

                if let Some(board_move) = parent_node.board_move() {
                    if board_move.is_opposite(&curr_move) {
                        continue;
                    }
                }

                children.push(TreeNode::new(
                    Some(Rc::clone(parent_node)),
                    Some(curr_move),
                    board.clone(),
                ));
            }
        }

        children
    }

    fn bfs(&mut self, root: Rc<TreeNode>) -> Option<Rc<TreeNode>> {
        self.seen.insert(root.board().hash());

        let mut queue = VecDeque::from([root]);

        while !queue.is_empty() {
            let queue_size = queue.len();

            for _ in 0..queue_size {
                let node = queue.pop_front().unwrap();

                let mut curr_board = node.board().clone();

                if let Some(curr_move) = node.board_move() {
                    if curr_board
                        .move_block(
                            curr_move.block_idx(),
                            curr_move.move_().row_diff(),
                            curr_move.move_().col_diff(),
                        )
                        .is_err()
                    {
                        continue;
                    };

                    let board_hash = curr_board.hash();

                    if self.seen.contains(&board_hash) {
                        continue;
                    }

                    self.seen.insert(board_hash);

                    if curr_board.is_solved() {
                        return Some(node.clone());
                    }
                }

                for child in self.get_moves(&curr_board, &node) {
                    queue.push_back(Rc::new(child));
                }
            }
        }

        None
    }
}

impl Solver {
    pub fn new(start_board: Board) -> Result<Self, BoardError> {
        if !start_board.is_ready_to_solve() {
            return Err(BoardError::BoardNotReady);
        }

        if start_board.is_solved() {
            return Err(BoardError::BoardAlreadySolved);
        }

        Ok(Self {
            start_board,
            seen: HashSet::<String>::new(),
        })
    }

    pub fn solve(&mut self) -> Option<Vec<BoardMove>> {
        let root = TreeNode::new(None, None, self.start_board.clone());

        if let Some(leaf) = self.bfs(Rc::new(root)) {
            return Some(leaf.collect());
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::game::{block::PositionedBlock, board::Board};

    #[test]
    fn test_not_ready_board() {
        let board = Board::default();

        assert!(Solver::new(board).is_err());
    }

    #[test]
    fn test_classic_board() {
        let mut board = Board::default();

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

        for block in blocks.iter() {
            board.add_block(block.clone()).unwrap();
        }

        let maybe_solver = Solver::new(board);

        assert!(maybe_solver.is_ok());

        let mut solver = maybe_solver.unwrap();

        let maybe_moves = solver.solve();

        assert!(maybe_moves.is_some());

        let moves = maybe_moves.unwrap();

        assert_eq!(moves.len(), 81);
    }
}
