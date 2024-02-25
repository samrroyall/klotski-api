use diesel::prelude::*;

use crate::errors::board::Error as BoardError;
use crate::models::db::schema::boards::dsl::{boards, id};
use crate::models::{
    db::tables::{InsertableBoard, SelectableBoard},
    game::board::Board,
};
use crate::services::db::Pool as DbPool;

#[derive(Debug)]
pub enum Error {
    BoardError(BoardError),
    DieselError(diesel::result::Error),
}

impl From<BoardError> for Error {
    fn from(e: BoardError) -> Self {
        Error::BoardError(e)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        Error::DieselError(e)
    }
}

pub fn create(pool: &DbPool) -> Result<Board, Error> {
    let mut conn = pool.get().unwrap();

    let new_board_state = InsertableBoard::from(&Board::default());

    let result = diesel::insert_into(boards)
        .values(&new_board_state)
        .get_result::<SelectableBoard>(&mut conn)?
        .into_board();

    Ok(result)
}

pub fn get(search_id: i32, pool: &DbPool) -> Result<Board, Error> {
    let mut conn = pool.get().unwrap();

    let board = boards
        .filter(id.eq(search_id))
        .first::<SelectableBoard>(&mut conn)?
        .into_board();

    Ok(board)
}

fn get_count(pool: &DbPool) -> i64 {
    let mut conn = pool.get().unwrap();

    boards.count().first::<i64>(&mut conn).unwrap()
}

pub fn delete(search_id: i32, pool: &DbPool) -> Result<(), Error> {
    let mut conn = pool.get().unwrap();

    let old_count = get_count(pool);

    diesel::delete(boards.filter(id.eq(search_id))).execute(&mut conn)?;

    if get_count(pool) == old_count {
        return Err(Error::BoardError(BoardError::BoardNotFound));
    }

    Ok(())
}

pub fn update<F>(search_id: i32, update_fn: F, pool: &DbPool) -> Result<Board, Error>
where
    F: FnOnce(&mut Board) -> Result<(), BoardError>,
{
    let mut conn = pool.get().unwrap();

    let mut board = boards
        .filter(id.eq(search_id))
        .first::<SelectableBoard>(&mut conn)?
        .into_board();

    update_fn(&mut board)?;

    diesel::update(boards.filter(id.eq(search_id)))
        .set(&InsertableBoard::from(&board.clone()))
        .execute(&mut conn)?;

    Ok(board)
}
