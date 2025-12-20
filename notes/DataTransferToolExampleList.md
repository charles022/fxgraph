
---

# 1) Network Transport / Protocols

## WebSocket
* Rust server (axum-style) sending binary frames:
```rust
use axum::extract::ws::{Message, WebSocket};
use futures_util::SinkExt;

async fn handle_ws(mut ws: WebSocket, bytes: Vec<u8>) {
    ws.send(Message::Binary(bytes)).await.unwrap();
}
```

* JS client receiving bytes:
```js
const ws = new WebSocket("wss://example/ws");
ws.binaryType = "arraybuffer";
ws.onmessage = (e) => {
  const buf = new Uint8Array(e.data); // raw bytes
};
```

---

## HTTP / Fetch API
* JS streaming download (ReadableStream):
```js
const res = await fetch("/data");
const reader = res.body.getReader();
while (true) {
  const { done, value } = await reader.read();
  if (done) break;
  // value is Uint8Array chunk
}
```

* Rust server returning bytes:
```rust
use axum::{body::Bytes, response::IntoResponse};

async fn get_data() -> impl IntoResponse {
    Bytes::from(vec![1u8, 2, 3, 4])
}
```

---

## gRPC
* Rust server service skeleton (tonic):
```rust
// tonic::include_proto!("myapi");
#[derive(Default)]
pub struct MySvc;

#[tonic::async_trait]
impl myapi::my_server::My for MySvc {
    async fn ping(&self, _: tonic::Request<myapi::PingReq>)
      -> Result<tonic::Response<myapi::PingResp>, tonic::Status> {
        Ok(tonic::Response::new(myapi::PingResp{ ok: true }))
    }
}
```

---

## gRPC-Web
* Typical deploy pattern is “proxy translates gRPC-Web → gRPC”
* (There isn’t a clean “few lines of browser code” in the same way as WS/fetch; gRPC-Web typically uses generated JS/TS stubs.)
```text
Browser (gRPC-Web) -> Envoy proxy -> Rust tonic gRPC server
```

---

## Apache Flight
* Server uses Flight service (conceptual):
* (Again: Flight is gRPC-based and typically not used directly in browser WASM per your notes.)
```rust
// arrow_flight::flight_service_server::FlightServiceServer;
// Implement FlightService trait, then serve over gRPC.
```


---

# 2) Serialization / Memory Encoding Crates

## rkyv
* Shared struct + archive + validate:
```rust
use rkyv::{Archive, Serialize, Deserialize};
use rkyv::validation::validators::DefaultValidator;
use rkyv::check_archived_root;

#[derive(Archive, Serialize, Deserialize)]
pub struct GameState { pub score: u64, pub positions: Vec<(f32,f32)> }

let bytes: Vec<u8> = rkyv::to_bytes::<_, 256>(&state).unwrap().to_vec();
let archived: &rkyv::Archived<GameState> = check_archived_root::<GameState>(&bytes).unwrap();
let score = archived.score;
```

---

## bincode
* Serialize + deserialize:
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Req { col: String, val: String, limit: u32 }

let bytes = bincode::serialize(&req).unwrap();
let req2: Req = bincode::deserialize(&bytes).unwrap();
```

---

## bytemuck
* POD struct (fixed-size) + cast from bytes:
```rust
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct PhysicsState { x: f32, y: f32, id: u32 }

let mut net_buf: Vec<u8> = vec![0u8; core::mem::size_of::<PhysicsState>()];
let st: &mut PhysicsState = bytemuck::from_bytes_mut(&mut net_buf);
st.x += 5.0;
```

---

## zerocopy
* Derive an “as-bytes/from-bytes” POD view (example pattern):
```rust
use zerocopy::{AsBytes, FromBytes};

#[repr(C)]
#[derive(AsBytes, FromBytes)]
struct Fixed { a: u32, b: u32 }

let bytes: &[u8] = fixed.as_bytes();
let view: &Fixed = zerocopy::ref_from(bytes).unwrap();
```

---

# 3) Arrow / Columnar Data Ecosystem

## Apache Arrow (arrow-rs)
* Build arrays + record batch:
```rust
use arrow_array::{Int32Array, RecordBatch};
use arrow_schema::{Schema, Field, DataType};
use std::sync::Arc;

let ids = Int32Array::from(vec![1,2,3]);
let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int32, false)]));
let batch = RecordBatch::try_new(schema, vec![Arc::new(ids)]).unwrap();
```

---

## Arrow IPC
* Write batch to IPC bytes:
```rust
use arrow::ipc::writer::{FileWriter, IpcWriteOptions};
let mut buf = Vec::new();
let mut w = FileWriter::try_new_with_options(&mut buf, batch.schema(), IpcWriteOptions::default()).unwrap();
w.write(&batch).unwrap();
w.finish().unwrap();
```

---

## Arrow IPC Streaming Format
* StreamWriter for schema + batch:
```rust
use arrow::ipc::writer::StreamWriter;

let mut buf = Vec::new();
let mut w = StreamWriter::try_new(&mut buf, batch.schema()).unwrap();
w.write(&batch).unwrap();
w.finish().unwrap();
```

---

## Arrow IPC File Format
* FileWriter (same as “Arrow IPC” write snippet; it’s the file format):
```rust
use arrow::ipc::writer::FileWriter;

let mut buf = Vec::new();
let mut w = FileWriter::try_new(&mut buf, batch.schema()).unwrap();
w.write(&batch).unwrap();
w.finish().unwrap();
```

---

## Arrow Builders
* Batch small updates into contiguous arrays:
```rust
use arrow_array::builder::Int32Builder;

let mut b = Int32Builder::with_capacity(1024);
b.append_value(10);
b.append_value(11);
let arr = b.finish(); // freezes into immutable Int32Array (zero-copy handoff)
```

---

## Arrow Compute Kernels (`arrow::compute`)
* Filter / take / sort_to_indices patterns:
```rust
use arrow::compute::{filter, take, sort_to_indices};
use arrow_array::{Int32Array, BooleanArray};

let a = Int32Array::from(vec![5, 9, 1]);
let mask = BooleanArray::from(vec![true, false, true]);
let filtered = filter(&a, &mask).unwrap();

let idx = sort_to_indices(&a, None, None).unwrap();
let sorted = take(&a, &idx, None).unwrap();
```

---

# 4) Polars DataFrame Ecosystem

## Polars
* Create a DF and do a simple operation:
```rust
use polars::prelude::*;

let df = df!("age" => [10, 40, 30], "name" => ["a","b","c"]).unwrap();
let out = df.lazy().filter(col("age").gt(lit(25))).collect().unwrap();
```

---

## DataFrame::vstack
* Append rows (creates chunks):
```rust
use polars::prelude::*;

let mut df1 = df!("x" => [1,2]).unwrap();
let df2 = df!("x" => [3,4]).unwrap();
df1.vstack_mut(&df2).unwrap();
```

---

## DataFrame::rechunk
* Defragment after many appends:
```rust
use polars::prelude::*;
df1 = df1.rechunk();
```

---

## DataFrame::slice
* Virtualization window:
```rust
use polars::prelude::*;
let view = df1.slice(100, 20); // offset=100, len=20
```

---

## polars_sql::SQLContext
* Execute SQL against a DataFrame:
```rust
use polars::prelude::*;
use polars_sql::SQLContext;

let df = df!("city" => ["a","b"], "salary" => [10, 20]).unwrap();
let mut ctx = SQLContext::new();
ctx.register("t", df.lazy());
let res = ctx.execute("SELECT * FROM t WHERE salary > 10").unwrap().collect().unwrap();
```

---

# 5) Compression & Writers

## ZSTD
* Arrow IPC compression option (ZSTD):
```rust
use arrow::ipc::writer::IpcWriteOptions;
use arrow::ipc::compression::CompressionType;

let opts = IpcWriteOptions::try_new()
    .with_compression(Some(CompressionType::ZSTD))
    .unwrap();
```

---

## IpcWriteOptions
* Configure IPC writer:
```rust
use arrow::ipc::writer::{FileWriter, IpcWriteOptions};

let mut buf = Vec::new();
let mut w = FileWriter::try_new_with_options(&mut buf, batch.schema(), IpcWriteOptions::default()).unwrap();
```

---

## Parquet
* Write Polars DataFrame to Parquet with ZSTD:
```rust
use polars::prelude::*;
use std::io::Cursor;

let mut df = df!("x" => [1,2,3]).unwrap();
let mut out = Cursor::new(Vec::new());
ParquetWriter::new(&mut out)
    .with_compression(ParquetCompression::Zstd(None))
    .finish(&mut df)
    .unwrap();
let parquet_bytes = out.into_inner();
```

---

# 6) Arrow IPC Writers / Readers

## FileWriter
* Snapshot writer:
```rust
use arrow::ipc::writer::FileWriter;

let mut buf = Vec::new();
let mut w = FileWriter::try_new(&mut buf, batch.schema()).unwrap();
w.write(&batch).unwrap();
w.finish().unwrap();
```

---

## StreamWriter / IpcStreamWriter
* Arrow StreamWriter (record batches):
```rust
use arrow::ipc::writer::StreamWriter;

let mut buf = Vec::new();
let mut w = StreamWriter::try_new(&mut buf, batch.schema()).unwrap();
w.write(&batch).unwrap();
w.finish().unwrap();
```

* Polars IpcStreamWriter (DataFrame → IPC stream bytes):
```rust
use polars::prelude::*;
use std::io::Cursor;

let mut df = df!("x" => [1,2,3]).unwrap();
let mut cur = Cursor::new(Vec::new());
IpcStreamWriter::new(&mut cur).finish(&mut df).unwrap();
let bytes = cur.into_inner();
```

---

## StreamReader
* Read Arrow IPC stream bytes:
```rust
use arrow::ipc::reader::StreamReader;
use std::io::Cursor;

let cur = Cursor::new(bytes);
let mut r = StreamReader::try_new(cur, None).unwrap();
while let Some(Ok(batch)) = r.next() {
    println!("rows={}", batch.num_rows());
}
```

---

# 7) WASM / Browser Interop

## WebAssembly.Memory
* JS gets a view into WASM memory:
```js
// wasmMemory is WebAssembly.Memory from wasm-bindgen init output
const mem = wasmMemory.buffer;
```

---

## Uint8Array
* JS wraps bytes and writes into WASM view:
```js
const wasmView = new Uint8Array(wasmMemory.buffer, ptr, totalSize);
wasmView.set(chunk, offset);
```

---

## ReadableStream
* JS reads chunks from fetch body:
```js
const reader = response.body.getReader();
const { done, value } = await reader.read(); // value is Uint8Array
```

---

# 8) wasm-bindgen

## wasm-bindgen
* Export a Rust function to JS:
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 { a + b }
```

---

## #[wasm_bindgen]
* Export a struct + methods:
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct DataReceiver { buffer: Vec<u8> }

#[wasm_bindgen]
impl DataReceiver {
  pub fn new(size: usize) -> DataReceiver { DataReceiver { buffer: Vec::with_capacity(size) } }
}
```

---

# 9) UI / Rendering

## Canvas / WebGL
* JS gets canvas + WebGL context:
```js
const canvas = document.getElementById("c");
const gl = canvas.getContext("webgl2");
```

---

## wgpu
* (wgpu setup is longer; this is the essential call flow you’d expand.)
* WASM init pattern (very short shape):
```rust
// wgpu::Instance::new -> request_adapter -> request_device -> create_surface(canvas)
```


---

## web-sys
* Access DOM from Rust:
```rust
use web_sys::window;

let doc = window().unwrap().document().unwrap();
let el = doc.get_element_by_id("rust-table-root").unwrap();
```

---

## egui
* Egui frame update shape:
```rust
// inside your egui update loop:
ui.label("row value");
if ui.button("Sort").clicked() { /* trigger sort */ }
```

---

## Leptos
* Mount a component into a specific div:
```rust
use leptos::*;
use wasm_bindgen::prelude::*;

#[component]
fn DataViewer() -> impl IntoView { view! { <div>"hello"</div> } }

#[wasm_bindgen]
pub fn mount_table(id: &str) {
    mount_to(
        document().get_element_by_id(id).unwrap(),
        || view! { <DataViewer /> }
    )
}
```

---

# 10) JS ↔ Rust Communication

## Exported Rust Functions
* JS calls into Rust:
```js
import init, { apply_js_filter } from "./pkg/app.js";
await init();
apply_js_filter("US-East");
```

* Rust receives input:
```rust
#[wasm_bindgen]
pub fn apply_js_filter(region: String) {
    // update internal state / signal, rerender, etc.
}
```

---

## CustomEvent
* Rust dispatches event:
```rust
use web_sys::{window, CustomEvent};

let ev = CustomEvent::new("rust-row-selected").unwrap();
window().unwrap().dispatch_event(&ev).unwrap();
```

* JS listens:
```js
window.addEventListener("rust-row-selected", () => {
  console.log("row selected");
});
```

---

# 11) Memory / Buffer Operations

## Vec::with_capacity
* Preallocate once:
```rust
let mut buf: Vec<u8> = Vec::with_capacity(total_size);
```

---

## Vec::set_len
* Mark as filled after direct writes (unsafe):

```rust
unsafe { buf.set_len(total_size); }
```

---

## bytemuck::from_bytes / from_bytes_mut
* Immutable / mutable casts:
```rust
let st: &PhysicsState = bytemuck::from_bytes(&net_buf);
let st_mut: &mut PhysicsState = bytemuck::from_bytes_mut(&mut net_buf);
```

---

# 12) Concurrency Models

## Rayon
* Parallel iterator (CPU-bound):
```rust
use rayon::prelude::*;
let sum: i64 = (0..1_000_000i64).into_par_iter().sum();
```

---

## Tokio
* Async task for I/O-bound:
```rust
#[tokio::main]
async fn main() {
    let _ = tokio::spawn(async move { /* network I/O */ }).await;
}
```

---

# 13) Explicitly Avoided

## JSON
* (Avoided) Typical JS parse path:
```js
const obj = JSON.parse(text); // allocates lots of objects; GC pressure
```

---

## Protobuf (browser)
* (Avoided) Typical JS decode shape:
```js
// const msg = MyProto.decode(bytes); // allocates JS objects; parsing overhead
```

---

## Apache DataFusion (client)
* (Avoided) Query shape (Rust):
```rust
// use datafusion::prelude::*;
// let df = ctx.sql("SELECT ...").await?; // heavy + async engine in WASM context
```

---

