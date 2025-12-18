# Scenario 4.2 – Append with Arrow Builders to avoid fragmentation

Client buffers tiny updates in Arrow builders and flushes into Polars when thresholds hit.
```rust
use arrow_array::{RecordBatch, builder::{Int64Builder, Float64Builder}};
use polars::prelude::*;
use wasm_bindgen::prelude::*;

static mut BUILDER_ID: Option<Int64Builder> = None;
static mut BUILDER_VAL: Option<Float64Builder> = None;
static mut DF: Option<DataFrame> = None;

#[wasm_bindgen]
pub fn buffer_row(id: i64, val: f64) {
    unsafe {
        let b_id = BUILDER_ID.get_or_insert_with(|| Int64Builder::with_capacity(1024));
        let b_val = BUILDER_VAL.get_or_insert_with(|| Float64Builder::with_capacity(1024));
        b_id.append_value(id);
        b_val.append_value(val);
        if b_id.len() >= 512 {
            flush();
        }
    }
}

#[wasm_bindgen]
pub fn flush() {
    unsafe {
        let ids = BUILDER_ID.take().unwrap().finish();
        let vals = BUILDER_VAL.take().unwrap().finish();
        let rb = RecordBatch::try_new(
            std::sync::Arc::new(arrow_schema::Schema::new(vec![
                arrow_schema::Field::new("id", arrow_schema::DataType::Int64, false),
                arrow_schema::Field::new("val", arrow_schema::DataType::Float64, false),
            ])),
            vec![std::sync::Arc::new(ids), std::sync::Arc::new(vals)],
        ).unwrap();
        let incoming = DataFrame::try_from(rb).unwrap();
        if let Some(df) = &mut DF { df.vstack_mut(&incoming).unwrap(); }
        else { DF = Some(incoming); }
    }
}
```
