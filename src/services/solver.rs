use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

use crate::errors::board::Error as BoardError;
use crate::models::game::{board::Board, moves::FlatBoardMove};

#[derive(Debug, Clone)]
struct TreeNode {
    parent: Option<Rc<Self>>,
    move_: Option<FlatBoardMove>,
    board: Option<Board>,
}

impl TreeNode {
    pub fn new(
        parent: Option<Rc<Self>>,
        move_: Option<FlatBoardMove>,
        board: Option<Board>,
    ) -> Self {
        Self {
            parent,
            move_,
            board,
        }
    }

    pub fn move_(&self) -> Option<&FlatBoardMove> {
        self.move_.as_ref()
    }

    pub fn board(&self) -> Option<&Board> {
        self.board.as_ref()
    }

    pub fn parent(&self) -> Option<Rc<Self>> {
        self.parent.as_ref().map(Rc::clone)
    }

    pub fn collect(&self) -> Vec<FlatBoardMove> {
        let mut moves = vec![];

        let mut curr_node = Some(Rc::new(self.clone()));

        while let Some(node) = curr_node.as_ref() {
            if let Some(move_) = node.move_() {
                moves.push(move_.clone());

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
    fn get_children(board: &Board, parent_node: &Rc<TreeNode>) -> Vec<TreeNode> {
        let mut children = vec![];

        for (block_idx, moves) in board.get_next_moves().into_iter().enumerate() {
            for move_ in moves {
                let board_move = FlatBoardMove::new(block_idx, &move_);

                if let Some(parent_move) = parent_node.move_() {
                    if board_move.is_opposite(parent_move) {
                        continue;
                    }
                }

                children.push(TreeNode::new(
                    Some(Rc::clone(parent_node)),
                    Some(board_move),
                    Some(board.clone()),
                ));
            }
        }

        children
    }

    fn upsert_hash(&mut self, board_hash: String) -> bool {
        if self.seen.contains(&board_hash) {
            return false;
        }

        self.seen.insert(board_hash);

        true
    }

    fn bfs(&mut self, root: Rc<TreeNode>) -> Option<Rc<TreeNode>> {
        self.seen.insert(root.board().unwrap().hash());

        let mut queue = VecDeque::from([root]);

        while !queue.is_empty() {
            let queue_size = queue.len();

            for _ in 0..queue_size {
                let node = queue.pop_front().unwrap();

                let mut board = node.board().take().unwrap().to_owned();

                if let Some(move_) = node.move_() {
                    let _ = board.move_block_optimistic(
                        move_.block_idx(),
                        move_.row_diff(),
                        move_.col_diff(),
                    );

                    if !self.upsert_hash(board.hash()) {
                        continue;
                    }

                    if board.is_solved() {
                        return Some(node.clone());
                    }
                }

                for child in Self::get_children(&board, &node) {
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

    pub fn solve(&mut self) -> Option<Vec<FlatBoardMove>> {
        let root = TreeNode::new(None, None, Some(self.start_board.clone()));

        self.bfs(Rc::new(root)).map(|leaf| leaf.collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::game::{blocks::Positioned as PositionedBlock, board::Board};

    #[test]
    fn test_not_ready_board() {
        let board = Board::default();

        assert!(Solver::new(board).is_err());
    }

    fn test_board(blocks: &[PositionedBlock], expected_moves: usize) {
        let mut board = Board::default();

        for block in blocks.iter() {
            board.add_block(block.clone()).unwrap();
        }

        let mut solver = Solver::new(board).unwrap();

        let maybe_moves = solver.solve();

        assert!(maybe_moves.is_some());

        assert_eq!(maybe_moves.unwrap().len(), expected_moves);
    }

    #[test]
    fn test_classic_board() {
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

        test_board(&blocks, 81);
    }

    #[test]
    fn test_short_board() {
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

        test_board(&blocks, 17);
    }

    #[test]
    fn test_medium_board() {
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

        test_board(&blocks, 40);
    }

    #[test]
    fn test_hard_board() {
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

        test_board(&blocks, 120);
    }
}
