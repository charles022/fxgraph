# Scenario 8.3 – Rust copies small values to JS

Rust/WASM exports a function returning a small Vec; wasm-bindgen copies it across boundary.
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn selected_row(ids: &[u32]) -> Vec<u32> {
    ids.to_vec() // small copies are acceptable here
}
```

JS simply calls and receives a JS array.
```js
const ids = selected_row(new Uint32Array([1,2,3]));
console.log(ids);
```
