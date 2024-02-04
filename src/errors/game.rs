use std::error;
use std::fmt;

#[derive(Debug)]
pub enum BoardError {
    BlockIndexOutOfBounds,
    BlockInvalid,
    BlockPlacementInvalid,
    BoardNotFound,
}

impl error::Error for BoardError {}

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BoardError::BlockIndexOutOfBounds => write!(f, "Block index is out of bounds"),
            BoardError::BlockInvalid => write!(f, "Block ID provided is invalid"),
            BoardError::BlockPlacementInvalid => write!(f, "Block placement is invalid"),
            BoardError::BoardNotFound => write!(f, "No board with matching ID"),
        }
    }
}