# Scenario 1.3 – Fixed-size POD read-only via bytemuck + HTTP

Server writes a fixed-layout physics state to bytes and responds over HTTP.
```rust
use axum::response::IntoResponse;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct PhysicsState { pub x: f32, pub y: f32, pub id: u32 }

pub async fn snapshot() -> impl IntoResponse {
    let st = PhysicsState { x: 1.0, y: 2.0, id: 99 };
    bytemuck::bytes_of(&st).to_vec()
}
```

Client (Rust/WASM) casts incoming bytes directly to `&PhysicsState` with no allocation.
```rust
use bytemuck::from_bytes;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn view_state(bytes: &[u8]) -> f32 {
    let st: &PhysicsState = from_bytes(bytes);
    st.x + st.y
}
```

JS fetches once and passes bytes into WASM.
```js
const buf = await (await fetch("/snapshot")).arrayBuffer();
const view = new Uint8Array(buf);
console.log("sum", view_state(view));
```
