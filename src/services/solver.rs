use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

use crate::errors::game::BoardError;
use crate::models::game::{board::Board, move_::Move};

#[derive(Debug, Clone)]
struct TreeNode {
    parent: Option<Rc<Self>>,
    move_: Option<Move>,
    board: Option<Board>,
}

impl TreeNode {
    pub fn new(parent: Option<Rc<Self>>, move_: Option<Move>, board: Option<Board>) -> Self {
        Self {
            parent,
            move_,
            board,
        }
    }

    pub fn board_move(&self) -> Option<&Move> {
        self.move_.as_ref()
    }

    pub fn board(&self) -> Option<&Board> {
        self.board.as_ref()
    }

    pub fn parent(&self) -> Option<Rc<Self>> {
        self.parent.as_ref().map(Rc::clone)
    }

    pub fn collect(&self) -> Vec<Move> {
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

        board
            .get_next_moves()
            .into_iter()
            .enumerate()
            .for_each(|(block_idx, moves)| {
                for move_ in moves {
                    let curr_move = Move::new(block_idx, move_).unwrap();

                    if let Some(parent_move) = parent_node.board_move() {
                        if curr_move.is_opposite(parent_move) {
                            continue;
                        }
                    }

                    children.push(TreeNode::new(
                        Some(Rc::clone(parent_node)),
                        Some(curr_move),
                        Some(board.clone()),
                    ));
                }
            });

        children
    }

    fn upsert_hash(&mut self, board_hash: String) -> bool {
        if self.seen.contains(&board_hash) {
            return false;
        }

        self.seen.insert(board_hash);

        true
    }

    fn update_board_with_move(&self, board: &mut Board, move_: &Move) -> Result<(), BoardError> {
        board.move_block(move_.block_idx(), move_.steps())
    }

    fn bfs(&mut self, root: Rc<TreeNode>) -> Option<Rc<TreeNode>> {
        self.seen.insert(root.board().unwrap().hash());

        let mut queue = VecDeque::from([root]);

        while !queue.is_empty() {
            let queue_size = queue.len();

            for _ in 0..queue_size {
                let node = queue.pop_front().unwrap();

                let mut board = node.board().take().unwrap().to_owned();

                if let Some(move_) = node.board_move() {
                    if self.update_board_with_move(&mut board, move_).is_err() {
                        continue;
                    };

                    if !self.upsert_hash(board.hash()) {
                        continue;
                    }

                    if board.is_solved() {
                        return Some(node.clone());
                    }
                }

                self.get_moves(&board, &node)
                    .into_iter()
                    .for_each(|child| queue.push_back(Rc::new(child)));
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

    pub fn solve(&mut self) -> Option<Vec<Move>> {
        let root = TreeNode::new(None, None, Some(self.start_board.clone()));

        self.bfs(Rc::new(root)).map(|leaf| leaf.collect())
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
