use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GameState {
    pub score: u64,
    pub positions: Vec<(f32, f32)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameDelta {
    pub score_inc: u64,
    pub new_positions: Vec<(f32, f32)>,
}

static STATE: Lazy<Mutex<GameState>> = Lazy::new(|| Mutex::new(GameState::default()));

#[wasm_bindgen]
pub fn apply_delta(bytes: &[u8]) -> bool {
    match bincode::deserialize::<GameDelta>(bytes) {
        Ok(delta) => {
            if let Ok(mut state) = STATE.lock() {
                state.score = state.score.saturating_add(delta.score_inc);
                state.positions.extend(delta.new_positions.into_iter());
                true
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

#[wasm_bindgen]
pub fn encode_delta(delta: JsValue) -> Result<Vec<u8>, JsValue> {
    let parsed: GameDelta = delta
        .into_serde()
        .map_err(|e| JsValue::from_str(&format!("decode delta from JS: {e}")))?;
    bincode::serialize(&parsed)
        .map_err(|e| JsValue::from_str(&format!("serialize delta: {e}")))
}

#[wasm_bindgen]
pub fn get_state_json() -> Result<String, JsValue> {
    let state = STATE
        .lock()
        .map_err(|_| JsValue::from_str("state lock poisoned"))?;
    serde_json::to_string(&*state).map_err(|e| JsValue::from_str(&format!("serialize state: {e}")))
}

#[wasm_bindgen]
pub fn reset_state() {
    if let Ok(mut state) = STATE.lock() {
        *state = GameState::default();
    }
}
