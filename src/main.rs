#![warn(clippy::pedantic)]

use axum::{
    http::Method,
    routing::{delete, post, put},
    Extension, Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, Registry};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;

mod docs;
mod errors;
mod handlers;
mod models;
mod repositories;
mod services;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let environment = dotenvy::var("ENVIRONMENT").expect("ENVIRONMENT is not set");
    let log_level = dotenvy::var("LOG_LEVEL").expect("LOG_LEVEL is not set");
    let dsn = dotenvy::var("SENTRY_DSN").expect("SENTRY_DSN is not set");
    let bind_url = dotenvy::var("BIND_URL").expect("BIND_URL is not set");
    let bind_port = dotenvy::var("BIND_PORT").expect("BIND_PORT is not set");

    let _sentry_guard = sentry::init((
        dsn,
        sentry::ClientOptions {
            environment: Some(environment.into()),
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    let subscriber = Registry::default()
        .with(tracing_subscriber::EnvFilter::new(log_level))
        .with(tracing_subscriber::fmt::layer())
        .with(sentry_tracing::layer());

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    let db_pool = services::db::get_db_pool();

    let cors = CorsLayer::new()
        .allow_methods([Method::DELETE, Method::POST, Method::PUT])
        .allow_headers(Any)
        .allow_origin(Any);

    let block_routes = Router::new()
        .route("/", post(handlers::block::add))
        .route("/:block_idx", put(handlers::block::alter))
        .route("/:block_idx", delete(handlers::block::remove));

    let board_routes = Router::new()
        .route("/", post(handlers::board::new))
        .route("/:board_id", put(handlers::board::alter))
        .route("/:board_id", delete(handlers::board::delete))
        .route("/:board_id/solve", post(handlers::board::solve))
        .nest("/:board_id/block", block_routes);

    let api_routes = Router::new().nest("/board", board_routes);

    let app = Router::new()
        .nest("/api", api_routes)
        .layer(Extension(db_pool))
        .layer(cors)
        .merge(
            RapiDoc::with_openapi("/api-docs/openapi.json", docs::ApiDoc::openapi())
                .path("/rapidoc"),
        );

    let listener = tokio::net::TcpListener::bind(format!("{bind_url}:{bind_port}"))
        .await
        .unwrap();

    tracing::debug!("Listening on {bind_url}:{bind_port}");

    axum::serve(listener, app).await.unwrap();
}
