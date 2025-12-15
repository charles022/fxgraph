# Scenario 8.2 – JS views Rust memory directly

Rust exposes pointer/len to a snapshot buffer; JS wraps it with `Uint8Array`.
```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

static mut SNAPSHOT: Vec<u8> = Vec::new();

#[wasm_bindgen]
pub fn snapshot_ptr() -> *const u8 { unsafe { SNAPSHOT.as_ptr() } }

#[wasm_bindgen]
pub fn snapshot_len() -> usize { unsafe { SNAPSHOT.len() } }
```

JS obtains a view without copying.
```js
const ptr = snapshot_ptr();
const len = snapshot_len();
const bytes = new Uint8Array(wasmMemory.buffer, ptr, len);
console.log(bytes[0]);
```
