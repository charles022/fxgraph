# Scenario 2 – Merge into persistent state (general pattern)

Long-lived Rust/WASM state is updated with small delta messages. Server sends compact updates over WebSocket.
```rust
// server: stream deltas
ws.send(Message::Binary(delta_bytes)).await?; // delta_bytes may be POD or bincode/rkyv
```

Client holds global state and applies each delta.
```rust
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct GameState { pub score: u64, pub positions: Vec<(f32, f32)> }

#[derive(Serialize, Deserialize)]
pub struct GameDelta { pub score_inc: u64, pub new_positions: Vec<(f32, f32)> }

static mut GAME_STATE: GameState = GameState { score: 0, positions: Vec::new() };

#[wasm_bindgen]
pub fn apply_delta(bytes: &[u8]) {
    if let Ok(delta) = bincode::deserialize::<GameDelta>(bytes) {
        unsafe {
            GAME_STATE.score += delta.score_inc;
            GAME_STATE.positions.extend(delta.new_positions);
        }
    }
}
```

JS host wires WebSocket `onmessage` directly to `apply_delta`, keeping allocations bounded while state grows slowly in Rust.
