# Scenario 4.1 – Small frequent streaming appends (real-time)

Server streams tiny Arrow IPC batches over WebSocket without compression.
```rust
// server: loop { writer.write(&batch); ws.send(buf); buf.clear(); }
```

Client continuously reads `StreamReader` frames and appends; rechunks every N batches.
```rust
use polars::prelude::*;
use wasm_bindgen::prelude::*;

static mut DF: Option<DataFrame> = None;
static mut BATCHES: usize = 0;

#[wasm_bindgen]
pub fn realtime_append(bytes: &[u8]) {
    let mut reader = arrow::ipc::reader::StreamReader::try_new(std::io::Cursor::new(bytes), None).unwrap();
    if let Some(Ok(batch)) = reader.next() {
        let incoming = DataFrame::try_from(batch).unwrap();
        unsafe {
            if let Some(df) = &mut DF { df.vstack_mut(&incoming).unwrap(); }
            else { DF = Some(incoming); }
            BATCHES += 1;
            if BATCHES % 50 == 0 {
                if let Some(df) = &mut DF { *df = df.rechunk(); }
            }
        }
    }
}
```
