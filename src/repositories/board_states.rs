use diesel::prelude::*;
use uuid::Uuid;

use crate::models::db::schema::board_states::dsl::*;
use crate::models::{db::tables::BoardState, domain::game::Board};
use crate::services::db::DbPool;

pub fn create_board_state(pool: DbPool) -> QueryResult<BoardState> {
    let mut conn = pool
        .get()
        .expect("Failed to get a connection from the pool");

    let board_state = BoardState::from(&Uuid::new_v4().to_string(), &Board::default());

    diesel::insert_into(board_states)
        .values(board_state.clone())
        .execute(&mut conn)
        .map(|_| board_state)
}

pub fn get_board_state(search_id: &String, pool: DbPool) -> QueryResult<BoardState> {
    let mut conn = pool
        .get()
        .expect("Failed to get a connection from the pool");

    board_states
        .filter(id.eq(search_id))
        .first::<BoardState>(&mut conn)
}

pub fn delete_board_state(search_id: &String, pool: DbPool) -> QueryResult<usize> {
    let mut conn = pool
        .get()
        .expect("Failed to get a connection from the pool");

    diesel::delete(board_states.filter(id.eq(search_id))).execute(&mut conn)
}

pub fn update_board_state<F>(
    search_id: &String,
    update_fn: F,
    pool: DbPool,
) -> QueryResult<BoardState>
where
    F: FnOnce(&mut Board),
{
    let mut conn = pool
        .get()
        .expect("Failed to get a connection from the pool");

    match board_states
        .filter(id.eq(search_id))
        .first::<BoardState>(&mut conn)
    {
        Ok(board_state) => {
            let mut board = board_state.to_board();

            update_fn(&mut board);

            let new_board_state = BoardState::from(search_id, &board);

            diesel::update(board_states.filter(id.eq(search_id)))
                .set(new_board_state.clone())
                .execute(&mut conn)
                .map(|_| new_board_state)
        }
        Err(e) => Err(e),
    }
}
