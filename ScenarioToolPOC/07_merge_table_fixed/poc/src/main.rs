use std::{env, time::Duration};

use anyhow::{Context, Result};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{StreamExt, TryStreamExt};
use merge_table_fixed_poc::{apply_patch, reset_table, snapshot_front, Row, TABLE_CAPACITY};
use rand::Rng;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{error, info, warn};

const PATCH_ROWS: usize = 64;
const PREVIEW_ROWS: usize = 6;

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
    eprintln!("  merge_table_fixed_poc server              # start WebSocket server on 127.0.0.1:3000");
    eprintln!("  merge_table_fixed_poc client [ws_url]     # connect as client (default ws://127.0.0.1:3000/ws)");
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
        if let Err(err) = stream_patches(socket).await {
            error!(?err, "websocket stream failed");
        }
    })
}

async fn stream_patches(mut socket: WebSocket) -> Result<()> {
    let mut tick: u32 = 0;
    loop {
        let rows = build_patch(tick);
        let payload = bytemuck::cast_slice(&rows).to_vec();

        socket
            .send(Message::Binary(payload))
            .await
            .context("sending patch over websocket")?;

        tick = tick.wrapping_add(1);
        tokio::time::sleep(Duration::from_millis(950)).await;
    }
}

fn build_patch(tick: u32) -> Vec<Row> {
    let mut rng = rand::thread_rng();
    (0..PATCH_ROWS)
        .map(|idx| Row {
            id: idx as u32,
            temperature: (rng.gen_range(-10_000..35_000) as f32 / 100.0) + tick as f32 * 0.1,
        })
        .collect()
}

async fn run_client(url: &str) -> Result<()> {
    reset_table();
    info!("connecting to {url}");

    let (stream, _) = connect_async(url)
        .await
        .with_context(|| format!("connecting websocket client to {url}"))?;
    let (_write, read) = stream.split();

    let mut patch_count: u32 = 0;
    read.try_for_each(|msg| {
        patch_count = patch_count.wrapping_add(1);
        let seq = patch_count;

        async move {
            match msg {
                WsMessage::Binary(bytes) => handle_patch(&bytes, seq),
                WsMessage::Close(frame) => warn!("socket closed: {frame:?}"),
                WsMessage::Text(text) => info!("text frame: {text}"),
                _ => {}
            }
            Ok(())
        }
    })
    .await
    .context("receiving websocket frames")?;

    Ok(())
}

fn handle_patch(bytes: &[u8], patch_count: u32) {
    match apply_patch(bytes) {
        Ok(written) => {
            info!("patch #{patch_count}: wrote {} row(s)", written);
            print_preview(written);
        }
        Err(err) => warn!("patch #{patch_count} rejected: {err:?}"),
    }
}

fn print_preview(written: usize) {
    let sample = snapshot_front(PREVIEW_ROWS);
    println!(
        "table head ({} / {} rows shown) after writing {} rows:",
        sample.len(),
        TABLE_CAPACITY,
        written
    );
    for row in sample {
        println!("  id {:04} -> {:6.2} C", row.id, row.temperature);
    }
    println!("--------------------------------------------------");
}
