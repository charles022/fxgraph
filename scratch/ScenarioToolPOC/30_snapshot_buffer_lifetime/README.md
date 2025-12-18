# Snapshot buffer lifetime model

Each snapshot owns its buffer; when a new snapshot arrives, drop the old buffer wholesale.
```rust
use rkyv::{Archive, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Archive, Serialize)]
pub struct GameState { pub score: u64 }

static mut SNAPSHOT: Option<Box<[u8]>> = None;
static mut VIEW: Option<&'static rkyv::Archived<GameState>> = None;

#[wasm_bindgen]
pub fn replace_snapshot(bytes: Vec<u8>) {
    let boxed = bytes.into_boxed_slice(); // old buffer dropped on next replace
    let view_tmp = rkyv::check_archived_root::<GameState>(&boxed).unwrap();
    let view: &'static rkyv::Archived<GameState> = unsafe { std::mem::transmute(view_tmp) }; // boxed lives in static until replaced
    unsafe { SNAPSHOT = Some(boxed); VIEW = Some(view); }
}
```

Old buffer is freed when `SNAPSHOT` is replaced; no partial mutation occurs.
