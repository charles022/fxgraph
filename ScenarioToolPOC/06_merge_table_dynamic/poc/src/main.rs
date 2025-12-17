use std::{env, sync::Arc, time::Duration};

use anyhow::{anyhow, Context, Result};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use chrono::Utc;
use futures::{StreamExt, TryStreamExt};
use polars::{
    df,
    io::ipc::{IpcStreamReader, IpcStreamWriter},
    prelude::*,
};
use rand::Rng;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut args = env::args().skip(1);
    let Some(mode) = args.next() else {
        print_usage();
        return Ok(());
    };

    match mode.as_str() {
        "server" => run_server().await,
        "client" => {
            let url = args
                .next()
                .unwrap_or_else(|| "ws://127.0.0.1:3000/ws".to_string());
            run_client(&url).await
        }
        _ => {
            print_usage();
            Ok(())
        }
    }
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  poc server                  # start WebSocket server on 127.0.0.1:3000");
    eprintln!("  poc client [ws_url]         # connect as client (default ws://127.0.0.1:3000/ws)");
}

async fn run_server() -> Result<()> {
    let app = Router::new().route("/ws", get(ws_handler));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .context("binding websocket server")?;
    info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .await
        .context("running websocket server")
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| async move {
        if let Err(err) = stream_batches(socket).await {
            error!(?err, "websocket stream failed");
        }
    })
}

async fn stream_batches(mut socket: WebSocket) -> Result<()> {
    let mut rng = rand::thread_rng();
    let mut id: i64 = 0;

    loop {
        let df = df! {
            "id" => &[id],
            "value" => &[rng.gen_range(0..10_000)],
            "ts" => &[Utc::now().to_rfc3339()],
        }?;

        let mut bytes = Vec::new();
        let mut writer = IpcStreamWriter::new(&mut bytes);
        let mut df_copy = df.clone();
        writer.finish(&mut df_copy)?;

        socket
            .send(Message::Binary(bytes))
            .await
            .context("sending IPC batch over websocket")?;

        id += 1;
        tokio::time::sleep(Duration::from_millis(750)).await;
    }
}

async fn run_client(url: &str) -> Result<()> {
    info!("connecting to {url}");
    let (stream, _) = connect_async(url).await.context("connecting websocket client")?;
    let (_write, read) = stream.split();

    let table: Arc<Mutex<Option<DataFrame>>> = Arc::new(Mutex::new(None));
    let table_ref = table.clone();

    read.try_for_each(|msg| {
        let table_ref = table_ref.clone();
        async move {
            match msg {
                WsMessage::Binary(bytes) => {
                    let incoming = parse_batch(&bytes).context("parsing IPC batch")?;
                    merge_df(table_ref, incoming).await?;
                }
                WsMessage::Text(text) => info!("text frame: {text}"),
                WsMessage::Close(frame) => info!("received close frame: {frame:?}"),
                _ => {}
            }
            Ok(())
        }
    })
    .await
    .context("receiving websocket frames")?;

    Ok(())
}

fn parse_batch(bytes: &[u8]) -> Result<DataFrame> {
    let cursor = std::io::Cursor::new(bytes);
    let mut reader = IpcStreamReader::new(cursor);
    reader
        .next()
        .transpose()?
        .ok_or_else(|| anyhow!("empty IPC message"))
}

async fn merge_df(table: Arc<Mutex<Option<DataFrame>>>, incoming: DataFrame) -> Result<()> {
    let mut guard = table.lock().await;
    match guard.as_mut() {
        Some(df) => {
            df.vstack_mut(&incoming)?;
            info!("rows total: {} | latest batch height: {}", df.height(), incoming.height());
            println!("{}", df.tail(Some(5)));
        }
        None => {
            *guard = Some(incoming.clone());
            info!("initialized table with {} rows", incoming.height());
            println!("{}", incoming);
        }
    }
    Ok(())
}
