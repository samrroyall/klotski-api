#![warn(clippy::pedantic)]

use axum::{
    routing::{delete, post, put},
    Extension, Router,
};

mod errors;
mod handlers;
mod models;
mod repositories;
mod services;

use crate::handlers::board as board_handlers;

#[tokio::main]
async fn main() {
    let db_pool = services::db::get_db_pool();

    let board_routes = Router::new()
        .route("/", post(board_handlers::new))
        .route("/:board_id", put(board_handlers::alter))
        .route("/:board_id", delete(board_handlers::delete))
        .route("/:board_id/block", post(board_handlers::add_block))
        .route(
            "/:board_id/block/:block_idx",
            put(board_handlers::alter_block),
        )
        .route(
            "/:board_id/block/:block_idx",
            delete(handlers::board::remove_block),
        )
        .route("/:board_id/solve", post(board_handlers::solve));

    let api_routes = Router::new().nest("/board", board_routes);

    let app = Router::new()
        .nest("/api", api_routes)
        .layer(Extension(db_pool));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
