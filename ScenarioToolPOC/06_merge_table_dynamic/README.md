# Scenario 2.1 – Merge into persistent state (table, dynamic rows)

Server streams Arrow IPC record batches over WebSocket as new rows appear.
```rust
use arrow::ipc::writer::StreamWriter;
use axum::extract::ws::{Message, WebSocket};

pub async fn stream_batches(mut ws: WebSocket, batch: RecordBatch) {
    let mut buf = Vec::new();
    StreamWriter::try_new(&mut buf, batch.schema()).unwrap().write(&batch).unwrap();
    ws.send(Message::Binary(buf)).await.unwrap();
}
```

Client (Rust/WASM) ingests each batch and appends to a long-lived Polars DataFrame.
```rust
use polars::prelude::*;
use arrow::ipc::reader::StreamReader;
use wasm_bindgen::prelude::*;

static mut TABLE: Option<DataFrame> = None;

#[wasm_bindgen]
pub fn append_batch(bytes: &[u8]) {
    let mut reader = StreamReader::try_new(std::io::Cursor::new(bytes), None).unwrap();
    if let Some(Ok(batch)) = reader.next() {
        let incoming = DataFrame::try_from(batch).unwrap();
        unsafe {
            if let Some(df) = &mut TABLE {
                df.vstack_mut(&incoming).unwrap();
            } else {
                TABLE = Some(incoming);
            }
        }
    }
}
```

## Proof of concept in this folder

Run the small Rust demo inside `poc/`:
1) Start the WebSocket server that emits Arrow IPC batches encoded with Polars:
```
cargo run --bin poc -- server
```
2) In another shell, connect a client that merges every incoming batch into a long-lived Polars `DataFrame` and prints the tail:
```
cargo run --bin poc -- client          # or provide a URL: cargo run --bin poc -- client ws://127.0.0.1:3000/ws
```

The server generates a new row every ~750 ms and streams it as an IPC message. The client decodes each IPC batch, calls `vstack_mut` to append it to the existing table, and shows the last few rows so you can see the dynamic growth.
