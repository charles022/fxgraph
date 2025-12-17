use std::{env, time::Duration};

use anyhow::{Context, Result};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{StreamExt, TryStreamExt};
use merge_struct_fixed_poc::{apply_telemetry, reset, snapshot, Telemetry, TelemetryDelta};
use rand::Rng;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{error, info, warn};

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
    eprintln!("  merge_struct_fixed_poc server               # start WebSocket server on 127.0.0.1:3000");
    eprintln!("  merge_struct_fixed_poc client [ws_url]      # connect as client (default ws://127.0.0.1:3000/ws)");
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
        if let Err(err) = stream_deltas(socket).await {
            error!(?err, "websocket stream failed");
        }
    })
}

async fn stream_deltas(mut socket: WebSocket) -> Result<()> {
    let mut rng = rand::thread_rng();
    let mut tick: u64 = 0;

    loop {
        let delta = TelemetryDelta {
            cpu_inc: rng.gen_range(-2.0..5.0) + tick as f32 * 0.01,
            mem_inc: rng.gen_range(-32.0..48.0),
        };
        let payload = bytemuck::bytes_of(&delta).to_vec();

        socket
            .send(Message::Binary(payload))
            .await
            .context("sending telemetry delta over websocket")?;

        tick = tick.wrapping_add(1);
        tokio::time::sleep(Duration::from_millis(950)).await;
    }
}

async fn run_client(url: &str) -> Result<()> {
    reset();
    info!("connecting to {url}");

    let (stream, _) = connect_async(url)
        .await
        .with_context(|| format!("connecting websocket client to {url}"))?;
    let (_write, read) = stream.split();

    let mut delta_count: u32 = 0;
    read.try_for_each(|msg| {
        delta_count = delta_count.wrapping_add(1);
        let seq = delta_count;

        async move {
            match msg {
                WsMessage::Binary(bytes) => handle_delta(&bytes, seq),
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

fn handle_delta(bytes: &[u8], seq: u32) {
    match apply_telemetry(bytes) {
        Ok(updated) => {
            let delta = bytemuck::from_bytes::<TelemetryDelta>(bytes);
            info!(
                "delta #{seq} applied: cpu += {:+.2}, mem += {:+.2}",
                delta.cpu_inc, delta.mem_inc
            );
            print_snapshot(seq, updated);
        }
        Err(err) => warn!("delta #{seq} rejected: {err:?}"),
    }
}

fn print_snapshot(seq: u32, updated: Telemetry) {
    let live = snapshot();
    println!(
        "after #{seq:03}: cpu = {cpu:6.2}% | mem = {mem:6.2} MB (reported {:?})",
        updated,
        cpu = live.cpu,
        mem = live.mem
    );
}
