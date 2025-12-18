# Scenario 5.2 – Chunked streaming assembly over WebSocket

Server sends total size then fixed-size chunks.
```rust
ws.send(Message::Binary(total_size.to_le_bytes().to_vec())).await?;
for chunk in file_bytes.chunks(16 * 1024) {
    ws.send(Message::Binary(chunk.to_vec())).await?;
}
```

Client preallocates a Vec and writes each chunk into WASM memory; JS clears its buffer each loop.
```rust
use wasm_bindgen::prelude::*;
use std::cell::RefCell;

thread_local! { static BUF: RefCell<Vec<u8>> = RefCell::new(Vec::new()); }

#[wasm_bindgen]
pub fn alloc(size: usize) {
    BUF.with(|b| b.borrow_mut().reserve_exact(size));
}

#[wasm_bindgen]
pub fn write_at(offset: usize, chunk: &[u8]) {
    BUF.with(|b| {
        let mut buf = b.borrow_mut();
        if buf.len() < offset + chunk.len() { unsafe { buf.set_len(offset + chunk.len()); } }
        buf[offset..offset + chunk.len()].copy_from_slice(chunk);
    });
}
```

JS wiring:
```js
const ws = new WebSocket("wss://example/download");
ws.binaryType = "arraybuffer";
let offset = 0, total = 0;
ws.onmessage = ({ data }) => {
  const chunk = new Uint8Array(data);
  if (!total) { total = new DataView(chunk.buffer).getUint32(0, true); alloc(total); return; }
  write_at(offset, chunk); offset += chunk.length;
};
```
