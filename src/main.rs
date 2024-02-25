#![warn(clippy::pedantic)]

use axum::{
    http::{header, HeaderValue, Method},
    routing::{delete, post, put},
    Extension, Router,
};
use tower_http::cors::CorsLayer;

mod errors;
mod handlers;
mod models;
mod repositories;
mod services;

use crate::handlers::board as board_handlers;

#[tokio::main]
async fn main() {
    let db_pool = services::db::get_db_pool();

    let cors = CorsLayer::new()
        .allow_methods([Method::DELETE, Method::POST, Method::PUT])
        .allow_headers([header::CONTENT_TYPE])
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_credentials(false);

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
        .layer(Extension(db_pool))
        .layer(cors);

    dotenvy::dotenv().ok();

    let bind_url = dotenvy::var("BIND_URL").expect("BIND_URL is not set");
    let bind_port = dotenvy::var("BIND_PORT").expect("BIND_PORT is not set");

    let listener = tokio::net::TcpListener::bind(format!("{bind_url}:{bind_port}"))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
