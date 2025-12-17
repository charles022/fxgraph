use std::mem::size_of;

use shared::PhysicsState;
use wasm_bindgen::prelude::*;

static mut SNAPSHOT: Option<PhysicsState> = None;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

fn parse_state(bytes: &[u8]) -> Result<PhysicsState, JsValue> {
    if bytes.len() != size_of::<PhysicsState>() {
        return Err(JsValue::from_str("unexpected payload size"));
    }

    Ok(*bytemuck::from_bytes::<PhysicsState>(bytes))
}

fn snapshot() -> Result<PhysicsState, JsValue> {
    unsafe { SNAPSHOT.ok_or_else(|| JsValue::from_str("no snapshot ingested")) }
}

#[wasm_bindgen]
pub fn ingest_snapshot(bytes: &[u8]) -> Result<(), JsValue> {
    let state = parse_state(bytes)?;
    unsafe { SNAPSHOT = Some(state); }
    Ok(())
}

#[wasm_bindgen]
pub fn view_state(bytes: &[u8]) -> Result<f32, JsValue> {
    Ok(parse_state(bytes)?.sum())
}

#[wasm_bindgen]
pub fn id() -> Result<u32, JsValue> {
    Ok(snapshot()?.id)
}

#[wasm_bindgen]
pub fn x() -> Result<f32, JsValue> {
    Ok(snapshot()?.x)
}

#[wasm_bindgen]
pub fn y() -> Result<f32, JsValue> {
    Ok(snapshot()?.y)
}

#[wasm_bindgen]
pub fn sum() -> Result<f32, JsValue> {
    Ok(snapshot()?.sum())
}
