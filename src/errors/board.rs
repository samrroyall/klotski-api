use std::error;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    BlockIndexOutOfBounds,
    BlockInvalid,
    BlockPlacementInvalid,
    BoardNotFound,
    BoardStateInvalid,
    NoMovesToUndo,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::BlockIndexOutOfBounds => write!(f, "Block index is out of bounds"),
            Error::BlockInvalid => write!(f, "Block ID provided is invalid"),
            Error::BlockPlacementInvalid => write!(f, "Block placement is invalid"),
            Error::BoardNotFound => write!(f, "No board with matching ID"),
            Error::BoardStateInvalid => write!(f, "Board state is invalid for operation"),
            Error::NoMovesToUndo => write!(f, "No board moves to undo"),
        }
    }
}
