# Scenario 3 – Replace entire DataFrame (full refresh)

Server writes a full Polars DataFrame to Arrow IPC File format and serves over HTTP with optional ZSTD.
```rust
use polars::prelude::*;
use polars::prelude::IpcWriter;

pub async fn full_snapshot() -> Vec<u8> {
    let mut df = df!("id" => [1,2,3], "val" => [10,20,30]).unwrap();
    let mut out = std::io::Cursor::new(Vec::new());
    IpcWriter::new(&mut out).finish(&mut df).unwrap();
    out.into_inner()
}
```

Client fetches snapshot, reads IPC bytes, and replaces its local table.
```rust
use arrow::ipc::reader::FileReader;
use polars::prelude::*;
use wasm_bindgen::prelude::*;

static mut TABLE: Option<DataFrame> = None;

#[wasm_bindgen]
pub fn replace(bytes: &[u8]) {
    let mut reader = FileReader::try_new(std::io::Cursor::new(bytes), None).unwrap();
    let mut batches = Vec::new();
    for batch in reader { batches.push(batch.unwrap()); }
    let df = DataFrame::try_from(batches).unwrap();
    unsafe { TABLE = Some(df); }
}
```
