use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::game::BoardError;
use crate::models::db::schema::board_states::dsl::*;
use crate::models::{
    db::tables::BoardState,
    game::{board::Board, move_::Move},
};
use crate::services::db::DbPool;

#[derive(Debug)]
pub enum BoardStateRepositoryError {
    BoardError(BoardError),
    DieselError(diesel::result::Error),
}

impl From<BoardError> for BoardStateRepositoryError {
    fn from(e: BoardError) -> Self {
        BoardStateRepositoryError::BoardError(e)
    }
}

impl From<diesel::result::Error> for BoardStateRepositoryError {
    fn from(e: diesel::result::Error) -> Self {
        BoardStateRepositoryError::DieselError(e)
    }
}

pub fn create_board_state(pool: DbPool) -> Result<BoardState, BoardStateRepositoryError> {
    let mut conn = pool.get().unwrap();

    let board_state = BoardState::from(&Uuid::new_v4().to_string(), &Board::default());

    diesel::insert_into(board_states)
        .values(board_state.clone())
        .execute(&mut conn)?;

    Ok(board_state)
}

pub fn get_board_state(
    search_id: &String,
    pool: DbPool,
) -> Result<BoardState, BoardStateRepositoryError> {
    let mut conn = pool.get().unwrap();

    let board_state = board_states
        .filter(id.eq(search_id))
        .first::<BoardState>(&mut conn)?;

    Ok(board_state)
}

fn get_num_board_states(pool: &DbPool) -> i64 {
    let mut conn = pool.get().unwrap();

    board_states.count().first::<i64>(&mut conn).unwrap()
}

pub fn delete_board_state(
    search_id: &String,
    pool: DbPool,
) -> Result<(), BoardStateRepositoryError> {
    let mut conn = pool.get().unwrap();

    let old_count = get_num_board_states(&pool);

    diesel::delete(board_states.filter(id.eq(search_id))).execute(&mut conn)?;

    if get_num_board_states(&pool) == old_count {
        return Err(BoardStateRepositoryError::BoardError(
            BoardError::BoardNotFound,
        ));
    }

    Ok(())
}

fn get_updated_board<F>(
    search_id: &String,
    update_fn: F,
    pool: &DbPool,
) -> Result<Board, BoardStateRepositoryError>
where
    F: FnOnce(&mut Board) -> Result<(), BoardError>,
{
    let mut conn = pool.get().unwrap();

    let board_state = board_states
        .filter(id.eq(search_id))
        .first::<BoardState>(&mut conn)?;

    let mut board = board_state.to_board();

    update_fn(&mut board)?;

    Ok(board)
}

pub fn update_board_state_building<F>(
    search_id: &String,
    update_fn: F,
    pool: DbPool,
) -> Result<BoardState, BoardStateRepositoryError>
where
    F: FnOnce(&mut Board) -> Result<(), BoardError>,
{
    let updated_board = get_updated_board(search_id, update_fn, &pool)?;

    let new_board_state = BoardState::from(search_id, &updated_board);

    let mut conn = pool.get().unwrap();

    diesel::update(board_states.filter(id.eq(search_id)))
        .set(new_board_state.clone())
        .execute(&mut conn)?;

    Ok(new_board_state)
}

pub fn update_board_state_solving<F>(
    search_id: &String,
    update_fn: F,
    pool: DbPool,
) -> Result<(BoardState, Vec<Vec<Move>>), BoardStateRepositoryError>
where
    F: FnOnce(&mut Board) -> Result<(), BoardError>,
{
    let updated_board = get_updated_board(search_id, update_fn, &pool)?;

    let next_moves = updated_board.get_next_moves();

    let new_board_state = BoardState::from(search_id, &updated_board);

    let mut conn = pool.get().unwrap();

    diesel::update(board_states.filter(id.eq(search_id)))
        .set(new_board_state.clone())
        .execute(&mut conn)?;

    Ok((new_board_state, next_moves))
}
