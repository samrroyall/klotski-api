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

fn get_random_free_cell(free_cells: &[u8], rng: &mut ThreadRng) -> Option<Position> {
    let free_cell = free_cells[get_random(0, free_cells.len() - 1, rng)];

    let min_row = free_cell / Board::COLS;
    let min_col = free_cell % Board::COLS;

    Position::new(min_row, min_col)
}

fn add_remaining_blocks(board: &mut Board, rng: &mut ThreadRng) {
    let mut blocks = [
        Block::OneByOne,
        Block::OneByOne,
        Block::OneByOne,
        Block::TwoByOne,
        Block::TwoByOne,
        Block::OneByTwo,
    ];

    let mut free_cells = get_cells_free(board);

    while free_cells.len() > usize::from(Board::MIN_EMPTY_CELLS) {
        if let Some(position) = get_random_free_cell(&free_cells, rng) {
            blocks.shuffle(rng);

            let mut seen = vec![];

            for block in &blocks {
                if seen.contains(&block) {
                    continue;
                }

                seen.push(block);

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

// Randomly add block to the board in the building state. Add 2x2 block to a
// random valid position in the first three rows. Then add  remaining blocks at
// random until the board has no remaining empty cells. Remaining block
// probabilities are: 1/2 for 1x1 block, 1/3 for 2x1 block, and 1/6 1x2 block.
// This is done to reduce the risk of the board being unsolvable.
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
