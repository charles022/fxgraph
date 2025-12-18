# Scenario 8.4 – Rust writes directly to HTML via web-sys

Rust/WASM updates DOM nodes without returning data to JS.
```rust
use wasm_bindgen::prelude::*;
use web_sys::window;

#[wasm_bindgen]
pub fn set_status(msg: &str) {
    let doc = window().unwrap().document().unwrap();
    let el = doc.get_element_by_id("status").unwrap();
    el.set_text_content(Some(msg));
}
```

JS just triggers `set_status` as needed.
```js
set_status("Loading complete");
```
