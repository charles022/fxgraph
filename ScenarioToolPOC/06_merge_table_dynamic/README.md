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
