# Scenario 1.2 – Shared struct (Vec/String) mutable via bincode + HTTP fetch

Server serializes a dynamic struct with bincode and serves it over HTTP.
```rust
use axum::{response::IntoResponse, Json};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct PlayerReq { pub id: u32 }

#[derive(Serialize, Deserialize)]
pub struct PlayerProfile { pub id: u32, pub name: String, pub inventory: Vec<String> }

pub async fn profile(_: Json<PlayerReq>) -> impl IntoResponse {
    let profile = PlayerProfile { id: 7, name: "Ash".into(), inventory: vec!["potion".into()] };
    bincode::serialize(&profile).unwrap()
}
```

Client (Rust/WASM) fetches, deserializes to owned data, and can mutate freely.
```rust
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerProfile { pub id: u32, pub name: String, pub inventory: Vec<String> }

#[wasm_bindgen]
pub async fn pull_profile() -> Result<(), JsValue> {
    let resp_val = wasm_bindgen_futures::JsFuture::from(web_sys::window().unwrap().fetch_with_str("/profile")).await?;
    let resp: web_sys::Response = resp_val.dyn_into()?;
    let buf = wasm_bindgen_futures::JsFuture::from(resp.array_buffer()?).await?;
    let bytes = js_sys::Uint8Array::new(&buf);
    let owned: PlayerProfile = bincode::deserialize(&bytes.to_vec()).unwrap();
    mutate_inventory(owned);
    Ok(())
}

fn mutate_inventory(mut p: PlayerProfile) { p.inventory.push("ultra-ball".into()); }
```
