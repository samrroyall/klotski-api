use std::error;
use std::fmt;

#[derive(Debug)]
pub enum BoardError {
    BlockIndexOutOfBounds,
    BlockInvalid,
    BlockPlacementInvalid,
    BoardAlreadySolved,
    BoardNotFound,
    BoardNotReady,
    NoMovesToUndo,
}

impl error::Error for BoardError {}

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BoardError::BlockIndexOutOfBounds => write!(f, "Block index is out of bounds"),
            BoardError::BlockInvalid => write!(f, "Block ID provided is invalid"),
            BoardError::BlockPlacementInvalid => write!(f, "Block placement is invalid"),
            BoardError::BoardAlreadySolved => write!(f, "Board is already solved"),
            BoardError::BoardNotFound => write!(f, "No board with matching ID"),
            BoardError::BoardNotReady => write!(f, "Board not ready to solve"),
            BoardError::NoMovesToUndo => write!(f, "No board moves to undo"),
        }
    }
}
