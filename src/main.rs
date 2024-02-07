use axum::{
    routing::{delete, get, post, put},
    Extension, Router,
};

mod errors;
mod handlers;
mod models;
mod repositories;
mod services;

#[tokio::main]
async fn main() {
    let db_pool = services::db::get_db_pool();

    let board_routes = Router::new()
        .route("/", post(handlers::board::new_board))
        .route("/:board_id", get(handlers::board::get_board))
        .route("/:board_id", put(handlers::board::undo_move))
        .route("/:board_id", delete(handlers::board::delete_board))
        .route("/:board_id/block", post(handlers::board::add_block))
        .route(
            "/:board_id/block/:block_idx",
            put(handlers::board::alter_block),
        )
        .route(
            "/:board_id/block/:block_idx",
            delete(handlers::board::remove_block),
        );

    let api_routes = Router::new().nest("/board", board_routes);

    let app = Router::new()
        .nest("/api", api_routes)
        .layer(Extension(db_pool));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
