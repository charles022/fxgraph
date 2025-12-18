# Scenario 4.5 – Insert into DataFrame (remove + add)

Server sends a batch of replacement rows plus keys to remove.
```rust
// message = { remove_ids: Vec<i64>, insert_batch: Arrow IPC batch bytes }
```

Client filters existing table then appends inserts.
```rust
use polars::prelude::*;
use wasm_bindgen::prelude::*;

static mut DF: Option<DataFrame> = None;

#[wasm_bindgen]
pub fn apply_delta(remove_ids: &[i64], insert_bytes: &[u8]) {
    unsafe {
        if let Some(df) = &mut DF {
            let mask = df.column("id").unwrap().i64().unwrap().apply(|v| !remove_ids.contains(&v));
            let filtered = df.filter(&mask).unwrap();
            let mut reader = arrow::ipc::reader::StreamReader::try_new(std::io::Cursor::new(insert_bytes), None).unwrap();
            let incoming = DataFrame::try_from(reader.next().unwrap().unwrap()).unwrap();
            let mut new_df = filtered.vstack(&incoming).unwrap();
            new_df = new_df.rechunk();
            *df = new_df;
        }
    }
}
```
