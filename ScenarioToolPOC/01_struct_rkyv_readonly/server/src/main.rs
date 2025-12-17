use std::net::SocketAddr;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use rkyv::to_bytes;
use shared::GameState;
use tokio::time::interval;
use tower_http::services::ServeDir;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .nest_service("/", ServeDir::new("server/static"));

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    info!("serving on http://{addr}");

    if let Err(err) = axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await {
        error!("server error: {err}");
    }
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(run_socket)
}

async fn run_socket(mut socket: WebSocket) {
    let mut ticker = interval(Duration::from_secs(1));
    let mut tick: u64 = 0;

    loop {
        ticker.tick().await;
        let snapshot = GameState::moving_points(tick);
        tick = tick.wrapping_add(1);

        let bytes = match to_bytes::<_, 256>(&snapshot) {
            Ok(bytes) => bytes,
            Err(err) => {
                error!("failed to archive snapshot: {err}");
                break;
            }
        };

        if socket.send(Message::Binary(bytes.into())).await.is_err() {
            break;
        }
    }
}
