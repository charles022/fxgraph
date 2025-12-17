use std::net::SocketAddr;

use axum::{http::header, response::IntoResponse, routing::get, Router};
use shared::PhysicsState;
use tower_http::services::ServeDir;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    let app = Router::new()
        .route("/snapshot", get(snapshot))
        .nest_service("/", ServeDir::new("server/static"));

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    info!("serving http://{addr} (GET /snapshot)");

    if let Err(err) = axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await {
        error!("server error: {err}");
    }
}

async fn snapshot() -> impl IntoResponse {
    let st = PhysicsState::demo();
    let body = bytemuck::bytes_of(&st);

    (
        [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        body.to_vec(),
    )
}
