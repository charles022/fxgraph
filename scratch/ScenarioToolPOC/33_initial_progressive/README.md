# Scenario 11.2 – Initial load with early render (progressive tabular)

Server streams Arrow IPC Streaming format so client can render batches before full completion.
```rust
// server: StreamWriter over HTTP response body or WebSocket, writing schema then batches
```

Client starts reading batches immediately, updating UI as they arrive.
```rust
use arrow::ipc::reader::StreamReader;
use polars::prelude::*;
use wasm_bindgen::prelude::*;

static mut DF: Option<DataFrame> = None;

#[wasm_bindgen]
pub fn progressive_ingest(bytes: &[u8]) {
    let mut reader = StreamReader::try_new(std::io::Cursor::new(bytes), None).unwrap();
    while let Some(Ok(batch)) = reader.next() {
        let incoming = DataFrame::try_from(batch).unwrap();
        unsafe {
            if let Some(df) = &mut DF { df.vstack_mut(&incoming).unwrap(); }
            else { DF = Some(incoming); }
        }
    }
}
```

JS renders visible slice (`visible` from Scenario 8.1) after each `progressive_ingest` call to give early feedback.
