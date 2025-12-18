# Mutable buffer repurposing model

Reuse a preallocated Vec for repeated downloads; JS writes directly into the same WASM memory.
```rust
use wasm_bindgen::prelude::*;
use std::cell::RefCell;

thread_local! { static BUF: RefCell<Vec<u8>> = RefCell::new(Vec::new()); }

#[wasm_bindgen]
pub fn prepare(size: usize) {
    BUF.with(|b| {
        let mut buf = b.borrow_mut();
        buf.clear();
        buf.reserve_exact(size);
        unsafe { buf.set_len(size); }
    });
}

#[wasm_bindgen]
pub fn buf_ptr() -> *mut u8 { BUF.with(|b| b.borrow_mut().as_mut_ptr()) }
```

JS creates a `Uint8Array` view once and streams new payloads into the same memory address on each refresh, avoiding churn.
