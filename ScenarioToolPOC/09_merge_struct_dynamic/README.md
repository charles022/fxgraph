# Scenario 2.4 – Merge into persistent state (struct with Vec fields)

Server sends small dynamic updates encoded with bincode; client deserializes and merges into an existing owned struct.
```rust
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Default)]
pub struct ChatState { pub messages: Vec<String> }

#[derive(Serialize, Deserialize)]
pub struct ChatDelta { pub new_messages: Vec<String> }

static mut CHAT: ChatState = ChatState { messages: Vec::new() };

#[wasm_bindgen]
pub fn apply_chat(bytes: &[u8]) {
    let delta: ChatDelta = bincode::deserialize(bytes).unwrap();
    unsafe { CHAT.messages.extend(delta.new_messages); }
}
```
