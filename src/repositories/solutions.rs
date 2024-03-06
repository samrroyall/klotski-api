use diesel::prelude::*;
use diesel::result::Error;

use crate::models::db::schema::solutions::dsl::{hash, solutions};
use crate::models::{
    db::tables::{InsertableSolution, SelectableSolution},
    game::moves::FlatBoardMove,
};
use crate::services::db::Pool as DbPool;

pub fn create(
    new_hash: u64,
    moves: Option<Vec<FlatBoardMove>>,
    pool: &DbPool,
) -> Result<(), Error> {
    let mut conn = pool.get().unwrap();

    let new_solution = InsertableSolution::from(new_hash, moves);

    diesel::insert_into(solutions)
        .values(&new_solution)
        .execute(&mut conn)?;

    Ok(())
}

pub fn get(search_hash: u64, pool: &DbPool) -> Result<Option<Vec<FlatBoardMove>>, Error> {
    let mut conn = pool.get().unwrap();

    let moves = solutions
        .filter(hash.eq(search_hash as i64))
        .first::<SelectableSolution>(&mut conn)?
        .get_moves();

    Ok(moves)
}
