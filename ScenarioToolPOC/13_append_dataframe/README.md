# Scenario 4 – Append new rows to existing DataFrame

Server emits Arrow IPC record batches over WebSocket; client appends using `vstack_mut`.
```rust
// server: StreamWriter writes batches
```

Client appends then rechunks on demand.
```rust
use polars::prelude::*;
use wasm_bindgen::prelude::*;

static mut DF: Option<DataFrame> = None;

#[wasm_bindgen]
pub fn append(bytes: &[u8]) {
    let mut reader = arrow::ipc::reader::StreamReader::try_new(std::io::Cursor::new(bytes), None).unwrap();
    let incoming = DataFrame::try_from(reader.next().unwrap().unwrap()).unwrap();
    unsafe {
        if let Some(df) = &mut DF { df.vstack_mut(&incoming).unwrap(); }
        else { DF = Some(incoming); }
    }
}
```
