# Scenario 1.4 – Fixed-size POD mutable in place via bytemuck::from_bytes_mut

Proof-of-concept: the server pushes a single `PhysicsState` over WebSocket as raw bytes. The WASM client stores those bytes in a `Vec<u8>`, casts them with `bytemuck::from_bytes_mut`, and mutates the fields directly inside the buffer (no deserialize/serialize step).

## Project layout
- `shared/` – `PhysicsState` POD type shared by server and WASM.
- `server/` – Axum server that serves the static demo and streams one binary message on `/ws`.
- `client-wasm/` – `wasm-bindgen` library that owns the raw buffer, exposes getters, and a `mutate_in_place()` that tweaks `x`/`y` in-place via `bytemuck::from_bytes_mut`.
- `server/static/` – HTML/JS shell that opens the WebSocket, hands the bytes to WASM, and lets you trigger another in-place mutation.

## Build the WASM bundle
Prereq: `wasm-pack` on your PATH (or use `cargo build --target wasm32-unknown-unknown` + `wasm-bindgen` manually).
```bash
wasm-pack build client-wasm --target web --out-dir ../server/static/pkg
```

## Run the server + demo
```bash
cargo run -p server
# open http://127.0.0.1:3000
```

Click “Connect + pull snapshot” to receive the 12-byte struct, then “Mutate in WASM buffer” to update the same bytes in place. The UI reads back the fields through the WASM exports, showing that the backing buffer was mutated without re-fetching.
