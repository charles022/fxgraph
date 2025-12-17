use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use shared::{PlayerProfile, PlayerReq};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/profile", post(profile))
        .nest_service("/", ServeDir::new("web"));

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    println!("Serving http://{addr}/ with bincode payloads at POST /profile");

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn profile(Json(req): Json<PlayerReq>) -> impl IntoResponse {
    let mut profile = PlayerProfile::demo();
    profile.id = req.id;

    match bincode::serialize(&profile) {
        Ok(bytes) => (
            [(header::CONTENT_TYPE, "application/octet-stream")],
            bytes,
        )
            .into_response(),
        Err(err) => {
            eprintln!("failed to encode profile: {err}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
