# Scenario 10.1 – JS UI interaction triggers Rust/WASM request

UI button calls exported Rust function; Rust sends query to server and updates state.
```rust
use wasm_bindgen::prelude::*;
use web_sys::WebSocket;

#[wasm_bindgen]
pub fn on_filter_click(ws: &WebSocket, region: String) {
    let payload = format!("region={region}");
    ws.send_with_str(&payload).unwrap();
}
```

JS wiring connects DOM to Rust function.
```js
document.getElementById("apply").onclick = () => {
  on_filter_click(ws, document.getElementById("region").value);
};
```
