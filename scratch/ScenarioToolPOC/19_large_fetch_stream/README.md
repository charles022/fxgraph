# Scenario 5.3 – HTTP fetch streaming direct to WASM memory

JS streams `ReadableStream` chunks into a WASM buffer once.
```js
const res = await fetch("/big.arrow");
const reader = res.body.getReader();
const total = Number(res.headers.get("content-length"));
alloc(total);
let offset = 0;
while (true) {
  const { done, value } = await reader.read();
  if (done) break;
  write_at(offset, value); // exported WASM fn writing into preallocated Vec
  offset += value.length;
}
finalize();
```

Rust/WASM side preallocates and finalizes with `Vec::set_len` once all bytes are written.
```rust
use wasm_bindgen::prelude::*;
use std::cell::RefCell;

thread_local! { static BUF: RefCell<Vec<u8>> = RefCell::new(Vec::new()); }

#[wasm_bindgen]
pub fn alloc(size: usize) { BUF.with(|b| b.borrow_mut().reserve_exact(size)); }

#[wasm_bindgen]
pub fn write_at(offset: usize, chunk: &[u8]) {
    BUF.with(|b| {
        let mut buf = b.borrow_mut();
        if buf.len() < offset + chunk.len() { unsafe { buf.set_len(offset + chunk.len()); } }
        buf[offset..offset + chunk.len()].copy_from_slice(chunk);
    });
}

#[wasm_bindgen]
pub fn finalize() -> usize { BUF.with(|b| b.borrow().len()) }
```
