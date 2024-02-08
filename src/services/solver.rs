use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

use crate::errors::game::BoardError;
use crate::models::game::board::{Board, BoardMove};

#[derive(Debug, Clone)]
struct BoardMoveNode {
    parent: Option<Rc<RefCell<BoardMoveNode>>>,
    board_move: BoardMove,
}

impl BoardMoveNode {
    pub fn new(parent: Option<Rc<RefCell<BoardMoveNode>>>, board_move: BoardMove) -> Self {
        Self { parent, board_move }
    }

    pub fn board_move(&self) -> &BoardMove {
        &self.board_move
    }

    pub fn parent(&self) -> Option<&Rc<RefCell<BoardMoveNode>>> {
        self.parent.as_ref()
    }

    pub fn collect(self) -> Vec<BoardMove> {
        let mut moves = vec![];

        let mut curr_node = Some(self);

        while let Some(node) = curr_node.as_ref() {
            moves.push(node.board_move().clone());
            curr_node = node.parent().map(|p| p.borrow().clone());
        }

        moves.into_iter().rev().collect()
    }
}

pub struct Solver {
    start_board: Board,
    board_hashes_seen: HashSet<String>,
}

impl Solver {
    fn get_moves(
        &mut self,
        board: &mut Board,
        parent_move_node: Option<Rc<RefCell<BoardMoveNode>>>,
    ) -> Vec<BoardMoveNode> {
        let mut children = vec![];

        for (block_idx, moves) in board.get_next_moves().into_iter().enumerate() {
            for move_ in moves.into_iter() {
                let curr_move = BoardMove::new(block_idx, move_);

                if let Some(parent) = parent_move_node.as_ref() {
                    children.push(BoardMoveNode::new(Some(Rc::clone(parent)), curr_move));
                } else {
                    children.push(BoardMoveNode::new(None, curr_move));
                }
            }
        }

        children
    }
}

impl Solver {
    pub fn new(start_board: Board) -> Result<Self, BoardError> {
        if !start_board.is_ready_to_solve() {
            return Err(BoardError::BoardNotReady);
        }

        Ok(Self {
            board_hashes_seen: HashSet::from([start_board.hash()]),
            start_board,
        })
    }

    pub fn solve(&mut self) -> Option<Vec<BoardMove>> {
        let mut curr_board = self.start_board.clone();

        let initial_move_nodes = self.get_moves(&mut curr_board, None);

        for node in initial_move_nodes.iter() {
            println!("{:?}", node.board_move());
        }

        let mut queue = VecDeque::from(initial_move_nodes);

        let mut count = 0;

        while !queue.is_empty() {
            count += 1;

            let queue_size = queue.len();

            println!("Level {} - Queue Size: {}", count, queue_size);

            for _ in 0..queue_size {
                let move_node = queue.pop_front().unwrap();

                let curr_move = move_node.board_move();

                let move_result = curr_board.move_block(
                    curr_move.block_idx(),
                    curr_move.move_().row_diff(),
                    curr_move.move_().col_diff(),
                );

                if move_result.is_err() {
                    continue;
                }

                let board_hash = curr_board.hash();

                if self.board_hashes_seen.contains(&board_hash) {
                    continue;
                }

                self.board_hashes_seen.insert(board_hash);

                if curr_board.is_solved() {
                    return Some(move_node.clone().collect());
                }

                let next_move_nodes =
                    self.get_moves(&mut curr_board, Some(Rc::new(RefCell::new(move_node))));

                for node in next_move_nodes.into_iter() {
                    queue.push_back(node);
                }
            }
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

        for block in blocks {
            board.add_block(block).unwrap();
        }

        let maybe_solver = Solver::new(board);

        assert!(maybe_solver.is_ok());

        let mut solver = maybe_solver.unwrap();

        let maybe_moves = solver.solve();

        assert!(maybe_moves.is_some());
        assert_eq!(maybe_moves.unwrap().len(), 81);
    }
}
