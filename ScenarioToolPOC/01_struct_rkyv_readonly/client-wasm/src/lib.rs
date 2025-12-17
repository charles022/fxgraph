use js_sys::Array;
use rkyv::{check_archived_root, Archive};
use shared::GameState;
use wasm_bindgen::prelude::*;

type ArchivedGameState = <GameState as Archive>::Archived;

// Stored bytes keep the archived reference alive across JS calls.
static mut SNAPSHOT_BYTES: Vec<u8> = Vec::new();
static mut SNAPSHOT: Option<&'static ArchivedGameState> = None;

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn ingest_snapshot(bytes: &[u8]) -> Result<(), JsValue> {
    unsafe {
        SNAPSHOT_BYTES.clear();
        SNAPSHOT_BYTES.extend_from_slice(bytes);

        let archived = check_archived_root::<GameState>(&SNAPSHOT_BYTES)
            .map_err(|e| JsValue::from_str(&format!("invalid archive: {e:?}")))?;

        // Safety: SNAPSHOT_BYTES is held in static storage for the lifetime of the process.
        SNAPSHOT = Some(std::mem::transmute::<&ArchivedGameState, &'static ArchivedGameState>(
            archived,
        ));
    }
    Ok(())
}

fn snapshot() -> Result<&'static ArchivedGameState, JsValue> {
    unsafe { SNAPSHOT.ok_or_else(|| JsValue::from_str("no snapshot ingested yet")) }
}

#[wasm_bindgen]
pub fn tick() -> Result<u64, JsValue> {
    Ok(snapshot()?.tick)
}

#[wasm_bindgen]
pub fn score() -> Result<u64, JsValue> {
    Ok(snapshot()?.score)
}

#[wasm_bindgen]
pub fn positions_len() -> Result<usize, JsValue> {
    Ok(snapshot()?.positions.len())
}

#[wasm_bindgen]
pub fn position(idx: usize) -> Result<Option<Array>, JsValue> {
    let snap = snapshot()?;
    let pos = match snap.positions.get(idx) {
        Some(p) => p,
        None => return Ok(None),
    };

    let arr = Array::new();
    arr.push(&JsValue::from_f64(pos.0 as f64));
    arr.push(&JsValue::from_f64(pos.1 as f64));
    Ok(Some(arr))
}
