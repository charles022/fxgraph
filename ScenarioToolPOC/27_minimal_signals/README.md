# Scenario 9.2 – Rust/WASM returns minimal signals to JS

Rust emits small primitives or events to drive UI.
```rust
use wasm_bindgen::prelude::*;
use web_sys::{window, CustomEvent};

#[wasm_bindgen]
pub fn publish_ready() {
    let ev = CustomEvent::new("data-ready").unwrap();
    window().unwrap().dispatch_event(&ev).unwrap();
}
```

JS listens for the event and updates UI without receiving large payloads.
```js
window.addEventListener("data-ready", () => set_status("Data prepared"));
```
