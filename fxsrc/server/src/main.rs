use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/ws", get(ws_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    println!("Client connected");

    let (mut sender, mut receiver) = socket.split();

    // Spawn a task to handle incoming messages (just print them)
    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                println!("Client said: {}", text);
            }
        }
    });

    // Send a message every second
    let mut counter = 0;
    loop {
        if sender
            .send(Message::Text(format!("Ping {}", counter)))
            .await
            .is_err()
        {
            println!("Client disconnected");
            break;
        }
        counter += 1;
        sleep(Duration::from_millis(1000)).await;
    }
}
