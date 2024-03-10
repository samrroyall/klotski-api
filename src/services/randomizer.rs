use rand::{
    distributions::uniform::SampleUniform, rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng,
};

use crate::errors::board::Error as BoardError;
use crate::models::game::{
    blocks::{Block, Positioned as PositionedBlock},
    board::{Board, State as BoardState},
    utils::Position,
};

fn get_random<T>(min: T, max: T, rng: &mut ThreadRng) -> T
where
    T: PartialOrd + Copy + SampleUniform,
{
    rng.gen_range(min..=max)
}

fn get_cells_free(board: &Board) -> Vec<u8> {
    board
        .grid
        .iter()
        .enumerate()
        .filter(|(_, &cell)| cell.is_none())
        .map(|(i, _)| u8::try_from(i).unwrap())
        .collect::<Vec<u8>>()
}

fn get_random_free_cell(free_cells: &Vec<u8>, rng: &mut ThreadRng) -> Option<Position> {
    let free_cell = free_cells[get_random(0, free_cells.len() - 1, rng)];

    let min_row = free_cell / Board::COLS;
    let min_col = free_cell % Board::COLS;

    Position::new(min_row, min_col)
}

fn add_remaining_blocks(board: &mut Board, rng: &mut ThreadRng) {
    let mut blocks = [
        Block::OneByOne,
        Block::OneByOne,
        Block::OneByTwo,
        Block::TwoByOne,
    ];

    let mut free_cells = get_cells_free(board);

    while free_cells.len() > usize::from(Board::MIN_EMPTY_CELLS) {
        if let Some(position) = get_random_free_cell(&free_cells, rng) {
            blocks.shuffle(rng);

            for block in &blocks {
                if let Some(positioned_block) =
                    PositionedBlock::new(*block, position.row, position.col)
                {
                    if board.add_block(positioned_block).is_ok() {
                        free_cells = get_cells_free(board);

                        break;
                    }
                }
            }
        }
    }
}

fn add_two_by_two_block(board: &mut Board, rng: &mut ThreadRng) {
    let two_by_two_block = PositionedBlock::new(
        Block::TwoByTwo,
        get_random(0, 1, rng),
        get_random(0, 2, rng),
    )
    .unwrap();

    board.add_block(two_by_two_block).unwrap();
}

pub fn randomize(board: &mut Board) -> Result<(), BoardError> {
    let mut rng = thread_rng();

    add_two_by_two_block(board, &mut rng);
    add_remaining_blocks(board, &mut rng);

    board.change_state(BoardState::ReadyToSolve)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::models::game::board::Board;

    #[test]
    fn randomize_() {
        let mut board = Board::default();
        assert!(randomize(&mut board).is_ok());
    }
}
