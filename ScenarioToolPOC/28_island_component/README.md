# Scenario 9.3 – Island architecture component (Leptos)

Rust/WASM owns a single div via Leptos; JS treats it as a black box.
```rust
use leptos::*;
use wasm_bindgen::prelude::*;

#[component]
fn CounterIsland() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    view! { <button on:click=move |_| set_count.update(|c| *c += 1)>{ move || format!("count {count()}") }</button> }
}

#[wasm_bindgen]
pub fn mount_island(id: &str) {
    leptos::mount_to(
        &web_sys::window().unwrap().document().unwrap().get_element_by_id(id).unwrap(),
        || view! { <CounterIsland/> }
    );
}
```

JS only calls `mount_island("island-root")`; all logic stays in Rust.
