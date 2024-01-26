use axum::{
    routing::get, 
    Router
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    match listener.local_addr() {
        Ok(addr) => {
            println!("Server is running at http://{}:{}", addr.ip(), addr.port());
        },
        Err(e) => {
            println!("Error while starting the server: {}", e);
        }
    }

    axum::serve(listener, app).await.unwrap();
}
