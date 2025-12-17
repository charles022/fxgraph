# Scenario 2 – Merge into persistent state (general pattern)

This proof of concept shows long-lived Rust/WASM state that is updated with compact binary deltas. A mock WebSocket in the browser emits bincode-encoded messages; JS forwards the raw bytes straight into a `wasm-bindgen` export. Rust mutates its global state and returns JSON for the UI.

## Layout
- `src/lib.rs`: Rust state + delta codec (`apply_delta`, `encode_delta`, `get_state_json`, `reset_state`).
- `web/index.html` & `web/main.js`: Browser host that simulates a WebSocket feed and renders the state.

## Build and run
1) Build the WASM package (requires the `wasm32-unknown-unknown` target and `wasm-pack`):
```
wasm-pack build --target web --release
```
This creates `pkg/` with the JS bindings.

2) Serve the folder (any static server works). Example:
```
python -m http.server 8080
```

3) Open `http://localhost:8080/web/` in a browser. Use “Start stream” to let the mock socket push binary deltas, “Send single delta” for a one-off update, and “Reset state” to clear the Rust-side memory.

## How it works
- JS builds small `GameDelta` objects and hands them to `encode_delta`, returning `Uint8Array` bincode payloads.
- A mock socket forwards those bytes to `apply_delta`, which deserializes and mutates a `Lazy<Mutex<GameState>>`.
- The UI polls `get_state_json` to show the live Rust state without copying large buffers back and forth.
