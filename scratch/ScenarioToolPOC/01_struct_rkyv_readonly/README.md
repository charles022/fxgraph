# Scenario 1.1 – Shared struct (Vec/String) read-only via rkyv + WebSocket

Server (Rust, axum-style) builds a dynamic struct, archives with rkyv, and pushes over a binary WebSocket frame.
```rust
use axum::extract::ws::{Message, WebSocket};
use rkyv::{Archive, Serialize};

#[derive(Archive, Serialize)]
pub struct GameState { pub score: u64, pub positions: Vec<(f32, f32)> }

pub async fn send_snapshot(mut ws: WebSocket) {
    let state = GameState { score: 42, positions: vec![(1.0, 2.0), (3.0, 4.0)] };
    let bytes = rkyv::to_bytes::<_, 256>(&state).unwrap();
    ws.send(Message::Binary(bytes.into())).await.unwrap();
}
```

Client (Rust/WASM) receives bytes, validates archive, and exposes getters to JS without allocation.
```rust
use rkyv::{check_archived_root, Archived};
use wasm_bindgen::prelude::*;

static mut SNAPSHOT: Option<&'static Archived<GameState>> = None;

#[wasm_bindgen]
pub fn ingest_snapshot(bytes: &[u8]) {
    let archived: &Archived<GameState> = check_archived_root(bytes).unwrap();
    unsafe { SNAPSHOT = Some(archived); }
}

#[wasm_bindgen]
pub fn score() -> u64 {
    unsafe { SNAPSHOT.unwrap().score }
}
```

Host JS wires the WebSocket to `ingest_snapshot`.
```js
const ws = new WebSocket("wss://example/ws");
ws.binaryType = "arraybuffer";
ws.onmessage = ({ data }) => ingest_snapshot(new Uint8Array(data));
```

## Proof of concept in this repo

- `server/`: axum server that streams an archived `GameState` every second over `/ws` and serves the static demo UI.
- `client-wasm/`: wasm-bindgen library that validates the archive with rkyv and exposes getters to JS without copying.
- `server/static/`: HTML/JS shell that wires a WebSocket to the wasm bindings and renders the score/positions.

### Build the wasm client
Requires [`wasm-pack`](https://rustwasm.github.io/wasm-pack/).
```bash
wasm-pack build client-wasm --target web --out-dir ../server/static/pkg
```

### Run the server + demo
```bash
cargo run -p server
# open http://localhost:3000 in a browser
```

Every WebSocket client receives a new snapshot once per second with animated positions. The client validates the archive bytes and surfaces `score()`/`positions_len()`/`position(idx)` to JS.
