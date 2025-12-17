use std::mem::size_of;

use shared::PhysicsState;
use wasm_bindgen::prelude::*;

static mut SNAPSHOT: Vec<u8> = Vec::new();

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

fn read_buffer() -> Result<&'static [u8], JsValue> {
    unsafe {
        if SNAPSHOT.is_empty() {
            Err(JsValue::from_str("no snapshot loaded"))
        } else {
            Ok(&SNAPSHOT)
        }
    }
}

fn read_buffer_mut() -> Result<&'static mut [u8], JsValue> {
    unsafe {
        if SNAPSHOT.is_empty() {
            Err(JsValue::from_str("no snapshot loaded"))
        } else {
            Ok(&mut SNAPSHOT)
        }
    }
}

fn cast_state() -> Result<&'static PhysicsState, JsValue> {
    bytemuck::try_from_bytes(read_buffer()?)
        .map_err(|err| JsValue::from_str(&format!("cast failed: {err}")))
}

fn cast_state_mut() -> Result<&'static mut PhysicsState, JsValue> {
    bytemuck::try_from_bytes_mut(read_buffer_mut()?)
        .map_err(|err| JsValue::from_str(&format!("cast failed: {err}")))
}

#[wasm_bindgen]
pub fn ingest_snapshot(bytes: &[u8]) -> Result<(), JsValue> {
    if bytes.len() != size_of::<PhysicsState>() {
        return Err(JsValue::from_str("unexpected payload size"));
    }

    unsafe {
        SNAPSHOT.clear();
        SNAPSHOT.extend_from_slice(bytes);
    }
    Ok(())
}

#[wasm_bindgen]
pub fn mutate_in_place() -> Result<f32, JsValue> {
    let st = cast_state_mut()?;
    st.x += 1.0;
    st.y *= 2.0;
    Ok(st.y)
}

#[wasm_bindgen]
pub fn id() -> Result<u32, JsValue> {
    Ok(cast_state()?.id)
}

#[wasm_bindgen]
pub fn x() -> Result<f32, JsValue> {
    Ok(cast_state()?.x)
}

#[wasm_bindgen]
pub fn y() -> Result<f32, JsValue> {
    Ok(cast_state()?.y)
}

#[wasm_bindgen]
pub fn sum() -> Result<f32, JsValue> {
    Ok(cast_state()?.sum())
}
