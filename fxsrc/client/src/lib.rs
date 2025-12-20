use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"Starting fxgraph client...".into());

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let canvas = document
        .get_element_by_id("fxgraph-canvas")
        .expect("should have #fxgraph-canvas on the page")
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    
    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    // Initial draw
    context.set_fill_style(&"black".into());
    context.fill_rect(0.0, 0.0, 400.0, 400.0);
    context.set_fill_style(&"white".into());
    context.set_font("20px Arial");
    context.fill_text("Waiting for connection...", 10.0, 50.0)?;

    // Setup WebSocket
    let ws = WebSocket::new("ws://127.0.0.1:3000/ws")?;
    
    // Create a shared state for the callback closure
    let context_rc = Rc::new(context);
    let cloned_ctx = context_rc.clone();

    // On Message Callback
    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        if let Some(txt) = e.data().as_string() {
            web_sys::console::log_1(&format!("Received: {}", txt).into());
            
            // Draw visual feedback
            // Parse counter from "Ping N" to change color dynamically or just toggle
            let color = if txt.len() % 2 == 0 { "green" } else { "blue" };
            
            cloned_ctx.set_fill_style(&"black".into());
            cloned_ctx.fill_rect(0.0, 0.0, 400.0, 400.0);
            
            cloned_ctx.set_fill_style(&color.into());
            cloned_ctx.fill_rect(50.0, 50.0, 100.0, 100.0);
            
            cloned_ctx.set_fill_style(&"white".into());
            cloned_ctx.fill_text(&txt, 10.0, 200.0).unwrap_or(());
        }
    });
    
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget(); // Leak memory so closure stays alive

    // On Error Callback
    let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
        web_sys::console::error_1(&"WebSocket error".into());
    });
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    Ok(())
}
