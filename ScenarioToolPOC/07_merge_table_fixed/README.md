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
