use diesel::prelude::*;

use crate::errors::board::Error as BoardError;
use crate::models::db::schema::boards::dsl::{boards, id};
use crate::models::game::moves::FlatMove;
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

pub fn create(pool: &DbPool) -> Result<SelectableBoard, Error> {
    let mut conn = pool.get().unwrap();

    let new_board_state = InsertableBoard::from(&Board::default());

    let result = diesel::insert_into(boards)
        .values(&new_board_state)
        .get_result(&mut conn)?;

    Ok(result)
}

pub fn get(search_id: i32, pool: &DbPool) -> Result<SelectableBoard, Error> {
    let mut conn = pool.get().unwrap();

    let board_state = boards
        .filter(id.eq(search_id))
        .first::<SelectableBoard>(&mut conn)?;

    Ok(board_state)
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

fn get_updated<F>(search_id: i32, update_fn: F, pool: &DbPool) -> Result<Board, Error>
where
    F: FnOnce(&mut Board) -> Result<(), BoardError>,
{
    let mut conn = pool.get().unwrap();

    let board_record = boards
        .filter(id.eq(search_id))
        .first::<SelectableBoard>(&mut conn)?;

    let mut board = board_record.to_board();

    update_fn(&mut board)?;

    Ok(board)
}

pub fn update_while_building<F>(
    search_id: i32,
    update_fn: F,
    pool: &DbPool,
) -> Result<SelectableBoard, Error>
where
    F: FnOnce(&mut Board) -> Result<(), BoardError>,
{
    let updated_board = get_updated(search_id, update_fn, pool)?;

    let new_board_state = InsertableBoard::from(&updated_board);

    let mut conn = pool.get().unwrap();

    let result = diesel::update(boards.filter(id.eq(search_id)))
        .set(&new_board_state)
        .get_result(&mut conn)?;

    Ok(result)
}

type NextMoves = Vec<Vec<FlatMove>>;

pub fn update_while_solving<F>(
    search_id: i32,
    update_fn: F,
    pool: &DbPool,
) -> Result<(SelectableBoard, NextMoves), Error>
where
    F: FnOnce(&mut Board) -> Result<(), BoardError>,
{
    let updated_board = get_updated(search_id, update_fn, pool)?;

    let next_moves = updated_board.get_next_moves()?;

    let new_board_state = InsertableBoard::from(&updated_board);

    let mut conn = pool.get().unwrap();

    let result = diesel::update(boards.filter(id.eq(search_id)))
        .set(&new_board_state)
        .get_result(&mut conn)?;

    Ok((result, next_moves))
}
