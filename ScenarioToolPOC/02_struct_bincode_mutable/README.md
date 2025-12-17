# Scenario 1.2 – Shared struct (Vec/String) mutable via bincode + HTTP fetch

Proof-of-concept showing an Axum server sending a bincode-encoded struct and a WASM client that fetches, owns, and mutates it.

## Project layout
- `shared/` – `PlayerReq` and `PlayerProfile` types shared by server and client.
- `server/` – Axum server. `POST /profile` returns a bincode payload; static files are served from `web/`.
- `wasm_client/` – `wasm-bindgen` client exporting `pull_profile()` that fetches `/profile`, deserializes, pushes an item into the inventory, and renders it.
- `web/index.html` – Minimal page that loads the WASM bundle from `web/pkg/` and calls `pull_profile()`.

## Build the WASM bundle
Prereqs: a toolchain capable of producing `wasm32-unknown-unknown` and either `wasm-pack` or `wasm-bindgen` CLI installed locally.

Using `wasm-pack` (recommended):
```bash
wasm-pack build wasm_client --target web --out-dir ../web/pkg
```

Using `cargo` + `wasm-bindgen` manually:
```bash
cargo build -p wasm_client --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir web/pkg target/wasm32-unknown-unknown/release/wasm_client.wasm
```

## Run the server + page
```bash
cargo run -p server
# open http://127.0.0.1:3000
```
Click “Pull profile” to fetch the bincode payload, deserialize it in the browser, mutate the inventory, and display it on the page.
