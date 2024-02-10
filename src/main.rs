use axum::{
    routing::{delete, post, put},
    Extension, Router,
};

mod errors;
mod handlers;
mod models;
mod repositories;
mod services;

use crate::handlers::board::*;

#[tokio::main]
async fn main() {
    let db_pool = services::db::get_db_pool();

    let board_routes = Router::new()
        .route("/", post(new_board))
        .route("/:board_id", put(undo_move))
        .route("/:board_id", delete(delete_board))
        .route("/:board_id/block", post(add_block))
        .route("/:board_id/block/:block_idx", put(alter_block))
        .route(
            "/:board_id/block/:block_idx",
            delete(handlers::board::remove_block),
        )
        .route("/:board_id/solve", post(solve_board));

    let api_routes = Router::new().nest("/board", board_routes);

    let app = Router::new()
        .nest("/api", api_routes)
        .layer(Extension(db_pool));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
