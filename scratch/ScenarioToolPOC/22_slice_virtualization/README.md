# Scenario 8.1 – Client-side slice for virtualization

Rust/WASM holds full table; JS requests only visible rows.
```rust
use polars::prelude::*;
use wasm_bindgen::prelude::*;

static mut DF: Option<DataFrame> = None;

#[wasm_bindgen]
pub fn visible(offset: i64, len: usize) -> JsValue {
    unsafe {
        let view = DF.as_ref().unwrap().slice(offset, len);
        JsValue::from_serde(&view).unwrap()
    }
}
```

JS calls `visible` on scroll to get small payloads.
```js
const rows = visible(scrollOffset, 50);
renderRows(rows);
```
