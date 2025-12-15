# Scenario 6.1 – Typed request struct client → server

Client (WASM) sends a typed query encoded with bincode over WebSocket; server responds with Arrow IPC.
```rust
// client side
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct Query { pub city: String, pub limit: u32 }

#[wasm_bindgen]
pub fn send_query(ws: &web_sys::WebSocket, city: String, limit: u32) {
    let msg = bincode::serialize(&Query { city, limit }).unwrap();
    ws.send_with_u8_array(&msg).unwrap();
}
```

Server decodes request and returns filtered table.
```rust
use serde::{Serialize, Deserialize};
use axum::extract::ws::{Message, WebSocket};
use polars::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct Query { pub city: String, pub limit: u32 }

pub async fn handle(mut ws: WebSocket, df: DataFrame) {
    if let Some(Ok(Message::Binary(msg))) = ws.recv().await {
        let q: Query = bincode::deserialize(&msg).unwrap();
        let filtered = df.lazy().filter(col("city").eq(lit(q.city))).limit(q.limit).collect().unwrap();
        let mut cur = std::io::Cursor::new(Vec::new());
        polars::prelude::IpcStreamWriter::new(&mut cur).finish(&mut filtered.clone()).unwrap();
        ws.send(Message::Binary(cur.into_inner())).await.unwrap();
    }
}
```
