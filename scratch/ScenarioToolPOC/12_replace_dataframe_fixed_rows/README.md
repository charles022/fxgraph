# Scenario 3.2 – Replace DataFrame (same fixed row count)

When schema and row count are fixed, server can send a POD struct array for ultra-fast swap.
```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Row { pub id: u32, pub balance: f64 }

pub async fn snapshot_rows() -> Vec<u8> {
    let rows = [Row { id: 1, balance: 100.0 }, Row { id: 2, balance: 200.0 }];
    bytemuck::cast_slice(&rows).to_vec()
}
```

Client overwrites its fixed buffer with the incoming rows.
```rust
use wasm_bindgen::prelude::*;

static mut ROWS: [Row; 2] = [Row { id: 0, balance: 0.0 }; 2];

#[wasm_bindgen]
pub fn replace_rows(bytes: &[u8]) {
    let incoming: &[Row] = bytemuck::cast_slice(bytes);
    unsafe { ROWS.copy_from_slice(incoming); }
}
```
