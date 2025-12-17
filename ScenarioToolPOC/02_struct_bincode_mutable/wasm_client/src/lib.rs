use js_sys::Uint8Array;
use shared::PlayerProfile;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Headers, Request, RequestInit, RequestMode, Response};

#[wasm_bindgen]
pub async fn pull_profile() -> Result<(), JsValue> {
    let req_body = r#"{"id":7}"#;
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::SameOrigin);
    opts.body(Some(&JsValue::from_str(req_body)));

    let headers = Headers::new()?;
    headers.append("Content-Type", "application/json")?;
    opts.headers(&headers);

    let request = Request::new_with_str_and_init("/profile", &opts)?;
    let resp_value = JsFuture::from(
        window()
            .ok_or_else(|| JsValue::from_str("missing window"))?
            .fetch_with_request(&request),
    )
    .await?;

    let resp: Response = resp_value.dyn_into().map_err(|e| {
        JsValue::from_str(&format!("response conversion failed: {:?}", e))
    })?;
    let buffer = JsFuture::from(resp.array_buffer()?).await?;
    let bytes = Uint8Array::new(&buffer).to_vec();

    let mut profile: PlayerProfile = bincode::deserialize(&bytes)
        .map_err(|e| JsValue::from_str(&format!("bincode decode failed: {e}")))?;
    profile.add_item("ultra-ball");

    render_profile(&profile)?;
    Ok(())
}

fn render_profile(profile: &PlayerProfile) -> Result<(), JsValue> {
    let text = format!(
        "Player {} – {}. Inventory: {}",
        profile.id,
        profile.name,
        profile.inventory.join(", ")
    );

    let document = window()
        .and_then(|w| w.document())
        .ok_or_else(|| JsValue::from_str("missing document"))?;

    if let Some(element) = document.get_element_by_id("profile") {
        element.set_text_content(Some(&text));
    }

    Ok(())
}
