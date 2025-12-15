# Scenario 1.4 – Fixed-size POD mutable in place via bytemuck::from_bytes_mut

Server streams a single POD struct; client mutates fields directly inside the backing buffer.
```rust
use axum::extract::ws::{Message, WebSocket};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct PhysicsState { pub x: f32, pub y: f32, pub id: u32 }

pub async fn push_state(mut ws: WebSocket) {
    let st = PhysicsState { x: 10.0, y: 0.5, id: 1 };
    ws.send(Message::Binary(bytemuck::bytes_of(&st).to_vec())).await.unwrap();
}
```

Client (Rust/WASM) stores bytes in a Vec, then obtains a mutable view.
```rust
use bytemuck::from_bytes_mut;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn adjust(buf: &mut [u8]) -> f32 {
    let st: &mut PhysicsState = from_bytes_mut(buf);
    st.x += 1.0;
    st.y *= 2.0;
    st.y
}
```
