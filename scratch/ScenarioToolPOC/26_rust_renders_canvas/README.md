# Scenario 9.1 – Rust/WASM renders directly (Canvas/WebGL)

Rust owns a canvas, draws without handing data to JS.
```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen]
pub fn render(canvas_id: &str) {
    let doc = web_sys::window().unwrap().document().unwrap();
    let canvas: HtmlCanvasElement = doc.get_element_by_id(canvas_id).unwrap().dyn_into().unwrap();
    let ctx = canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
    ctx.set_fill_style(&"#0af".into());
    ctx.fill_rect(10.0, 10.0, 100.0, 50.0);
}
```

JS merely passes the canvas id once.
```js
render("viz");
```
