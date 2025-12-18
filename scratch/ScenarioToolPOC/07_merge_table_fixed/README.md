# Scenario 2.2 – Merge into persistent state (table, fixed rows)

For fixed-size tables, treat columns as POD arrays and overwrite slots in place.
```rust
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Row { pub id: u32, pub temperature: f32 }

static mut ROWS: [Row; 1024] = [Row::zeroed(); 1024];

#[wasm_bindgen]
pub fn apply_patch(bytes: &[u8]) {
    // bytes holds a Vec<Row> encoded as plain bytes
    let patch: &[Row] = bytemuck::cast_slice(bytes);
    unsafe { ROWS[..patch.len()].copy_from_slice(patch); }
}
```

Server simply sends an array of `Row` as raw bytes over HTTP or WebSocket; client updates in place without Polars overhead.

## Proof of concept in this folder

`poc/` contains a tiny Rust demo that mirrors the snippet above: a WebSocket server emits `Row` arrays as plain bytes, and a client casts them with `bytemuck::cast_slice` to overwrite the front of a 1024-slot table.

Run from this folder:
1) Start the server that sends 64-row patches every ~950 ms:
```
cd poc
cargo run -- server
```
2) In another shell, connect the client (defaults to `ws://127.0.0.1:3000/ws`):
```
cd poc
cargo run -- client
```

The client applies each binary payload directly into the fixed buffer and prints the head of the table after every patch, demonstrating in-place updates without any deserialization step.
