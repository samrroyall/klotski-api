use std::error;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    BlockIndexOutOfBounds,
    BlockInvalid,
    BlockPlacementInvalid,
    BoardAlreadySolved,
    BoardNotFound,
    BoardNotReady,
    NoMovesToUndo,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::BlockIndexOutOfBounds => write!(f, "Block index is out of bounds"),
            Error::BlockInvalid => write!(f, "Block ID provided is invalid"),
            Error::BlockPlacementInvalid => write!(f, "Block placement is invalid"),
            Error::BoardAlreadySolved => write!(f, "Board is already solved"),
            Error::BoardNotFound => write!(f, "No board with matching ID"),
            Error::BoardNotReady => write!(f, "Board not ready to solve"),
            Error::NoMovesToUndo => write!(f, "No board moves to undo"),
        }
    }
}
