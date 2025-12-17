# Scenario 1.3 – Fixed-size POD read-only via bytemuck + HTTP

Proof-of-concept: an Axum endpoint returns raw bytes for a fixed-layout `PhysicsState`, and the WASM client uses `bytemuck::from_bytes` to view the payload without parsing or allocation.

## Project layout
- `shared/` – defines `PhysicsState` as a POD type (`#[repr(C)]`, `bytemuck::Pod`).
- `server/` – Axum server exposing `GET /snapshot` and serving the static demo.
- `client-wasm/` – `wasm-bindgen` library that casts the incoming bytes to `PhysicsState` and surfaces getters.
- `server/static/` – HTML/JS shell that fetches `/snapshot`, calls into the wasm bundle, and renders the fields.

## Build the WASM bundle
Requires `wasm-pack` on your PATH.
```bash
wasm-pack build client-wasm --target web --out-dir ../server/static/pkg
```

## Run the server + demo
```bash
cargo run -p server
# open http://127.0.0.1:3000
```

Click “Fetch snapshot” to pull the 12-byte struct, reinterpret it inside WASM via bytemuck, and display the id/x/y/sum.
