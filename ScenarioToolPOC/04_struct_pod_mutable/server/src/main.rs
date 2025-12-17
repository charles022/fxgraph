use std::net::SocketAddr;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
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
        .route("/ws", get(upgrade_ws))
        .nest_service("/", ServeDir::new("server/static"));

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    info!("serving http://{addr} (connect with WS /ws)");

    if let Err(err) = axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await {
        error!("server error: {err}");
    }
}

async fn upgrade_ws(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let st = PhysicsState::demo();
    let payload = bytemuck::bytes_of(&st).to_vec();

    if let Err(err) = socket.send(Message::Binary(payload)).await {
        error!("failed to push snapshot: {err}");
        return;
    }

    // Keep the connection alive until the client closes it.
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Close(_)) => break,
            Ok(_) => continue,
            Err(err) => {
                error!("websocket recv error: {err}");
                break;
            }
        }
    }
}
