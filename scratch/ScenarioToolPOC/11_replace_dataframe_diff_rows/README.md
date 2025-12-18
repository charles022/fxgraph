# Scenario 3.1 – Replace DataFrame (different row counts)

Server still sends full Arrow IPC snapshot; client discards old table and swaps in new regardless of row count.
```rust
// same FileWriter path as Scenario 3, but snapshot may shrink or grow rows
```

Client swaps and uses slicing for UI virtualization after replacement.
```rust
use polars::prelude::*;
use wasm_bindgen::prelude::*;

static mut TABLE: Option<DataFrame> = None;

#[wasm_bindgen]
pub fn swap(bytes: &[u8], offset: i64, len: usize) -> Vec<u8> {
    replace(bytes); // from Scenario 3
    unsafe {
        let view = TABLE.as_ref().unwrap().slice(offset, len);
        // small JSON-esque export for UI; keeps bridge small
        serde_json::to_vec(&view).unwrap()
    }
}
```
