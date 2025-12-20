
// this doc maps `DatatransferToolExampleList.md` to `DataTransferToolList.md`

---

## 1) Custom Rust structs shared between client/server

### 1.1) Shared struct: variable size (Vec/String), read-only

* **Serialization / layout**

  * `rkyv` (zero-copy archived snapshot in WASM)
* **Transport**

  * WebSocket (binary frames) **or** HTTP `fetch()` (blob/stream)
* **Client ingestion**

  * Treat bytes as backing buffer; access via archived root (no JS parsing)
* **JS/UI exposure**

  * `wasm-bindgen` exported functions (JS asks, Rust reads snapshot)

### 1.2) Shared struct: variable size (Vec/String), mutable

* **Serialization**

  * `bincode` (deserialize to owned mutable struct) **or** `rkyv` + “copy-out” step (archive → owned)
* **Transport**

  * WebSocket or HTTP fetch
* **Client approach**

  * Receive bytes → deserialize into an owned Rust struct → mutate locally

### 1.3) Shared struct: fixed-size POD, read-only

* **Serialization**

  * `bytemuck` (cast `&[u8]` → `&T`) **or** `zerocopy` (POD view)
* **Transport**

  * WebSocket or HTTP fetch
* **Client approach**

  * Zero-copy view into bytes; no allocations

### 1.4) Shared struct: fixed-size POD, mutable in place

* **Serialization**

  * `bytemuck::from_bytes_mut` (cast `&mut [u8]` → `&mut T`)
* **Transport**

  * WebSocket or HTTP fetch
* **Client approach**

  * Write bytes into a stable buffer → mutate fields directly in-place

---

## 2) Merge into persistent state (incremental state evolution)

### 2) Merge into persistent state (general)

* **Approach**

  * Keep a long-lived “big state” in Rust/WASM; apply “small update messages” to it
* **Serialization**

  * POD updates: `bytemuck` / `zerocopy`
  * Dynamic updates: `rkyv` (read-only) or `bincode` (owned mutable)
* **Transport**

  * WebSocket preferred for frequent updates; HTTP fine for occasional

### 2.1) Merge into persistent state (table, dynamic rows)

* **Tabular tools**

  * Arrow Builders (buffer small rows) → flush
  * Polars `DataFrame::vstack` / `vstack_mut`
  * Polars `DataFrame::rechunk` periodically
* **Transport / format**

  * Arrow IPC Streaming or Arrow IPC batches (over WebSocket)

### 2.2) Merge into persistent state (table, fixed rows)

* **Approach**

  * Prefer “overwrite-in-place semantics” if you represent as fixed arrays (POD-style)
  * Otherwise treat as “replace DataFrame” (Scenario 3.2)
* **Tools**

  * POD fixed layout: `bytemuck` / `zerocopy`
  * DataFrame fixed rowcount: Arrow IPC + replace (see 3.2)

### 2.3) Merge into persistent state (struct, fixed)

* **Tools**

  * `bytemuck::from_bytes_mut` (fastest) or `zerocopy`
* **Transport**

  * WebSocket for frequent deltas

### 2.4) Merge into persistent state (struct, dynamic Vec fields)

* **Tools**

  * `bincode` (deserialize owned, then merge) or `rkyv` (snapshot + copy out)
* **Transport**

  * WebSocket or HTTP

---

## 3) Replace entire client DataFrame (full refresh)

### 3) Replace entire client DataFrame (full refresh)

* **Format**

  * Arrow IPC **File** or Arrow IPC (batch bytes)
  * Optional compression (ZSTD via `IpcWriteOptions`)
* **Transport**

  * HTTP fetch (often best for big snapshots) **or** WebSocket (works but watch buffering)
* **Client ingestion**

  * Arrow IPC reader path → construct DataFrame / RecordBatches → replace old state

### 3.1) Replace DataFrame (different row counts)

* **Same as 3**, but emphasizes:

  * Full replacement (no attempt to reuse buffers)
  * UI virtualization uses `DataFrame::slice` after replacement

### 3.2) Replace DataFrame (same fixed row count)

* **Two relevant approaches**

  * Still simplest: full replace via Arrow IPC
  * If truly fixed POD table shape: treat as fixed-size struct array and use `bytemuck` (only if schema is POD and you intentionally avoid Polars)

---

## 4) Append new data to an existing DataFrame (incremental growth)

### 4) Append new rows

* **Format**

  * Arrow IPC Streaming format (record batches) **or** Arrow IPC batches
* **Client tools**

  * Polars `DataFrame::vstack_mut` to append
  * Polars `DataFrame::rechunk` periodically

### 4.1) Small frequent streaming appends (real-time)

* **Transport**

  * WebSocket
* **Format**

  * Arrow IPC Streaming (`StreamWriter` / `StreamReader`)
* **Approach**

  * Keep updates small; avoid compression overhead
  * Rechunk on thresholds

### 4.2) Append with batching to avoid fragmentation (Arrow Builders hybrid)

* **Client tools**

  * Arrow Builders (accumulate rows)
  * Flush builders into Arrow arrays / RecordBatch
  * Append via Polars `vstack_mut`
  * Periodic `rechunk`
* **Transport**

  * WebSocket (small packets) or HTTP streaming if the update feed is unidirectional

### 4.5) Insert into DataFrame (insert + remove)

* **Approach options**

  * “Delta operations” message: send explicit inserts + deletes + keys
  * Or server sends rebuilt slice / partition and client replaces that portion (more brute-force)
* **Tools implicated**

  * Arrow IPC for the inserted rows
  * Polars operations for removal + insertion (exact method depends on your keying strategy)
  * If doing index-based removal: Arrow/Polars filtering/take patterns (`arrow::compute::filter/take`) or Polars equivalent

---

## 5) Large Payload Transfer (server → client)

### 5.1) Downloads (browser direct, no WASM)

* **Transport**

  * HTTP download (plain fetch / browser navigation)
* **No Rust/WASM path**

  * Data bypasses WASM entirely

### 5.2) Chunked streaming assembly (WebSocket)

* **Transport**

  * WebSocket binary frames
* **Approach**

  * Server sends total size → sends fixed-size chunks
  * Client accumulates into a preallocated buffer (`Vec::with_capacity`)
  * JS layer clears its chunk buffer each iteration (to avoid 2× spike)

### 5.3) HTTP fetch streaming + direct-to-WASM memory

* **Transport / browser API**

  * `fetch()` + `ReadableStream`
* **Interop tools**

  * `WebAssembly.Memory` + `Uint8Array` view into WASM memory
* **Rust buffer tools**

  * `Vec::with_capacity` + `Vec::set_len` (once fully written)

### 5.4) Compression / Encoding variations

* **Compression**

  * ZSTD (Arrow IPC compression via `IpcWriteOptions`)
  * Parquet + ZSTD for max size reduction (more CPU)
* **Where applied**

  * Primarily for large snapshot transfers (initial load / historical)

---

## 6) Client → Server Request Patterns

### 6.1) Typed request struct (field-based queries)

* **Serialization**

  * `bincode` (common simple binary) or `rkyv` (if you want archived struct)
* **Transport**

  * WebSocket request/response **or** HTTP POST
* **Server execution**

  * Polars query on server (DataFrame operations)
* **Response format**

  * Arrow IPC (table) or rkyv/bytemuck (struct), depending on returned shape

---

## 8) Rust/WASM → JS communication (return data)

### 8.1) Client-side slice for virtualization (visible rows only)

* **Client tools**

  * Polars `DataFrame::slice`
* **Bridge**

  * `wasm-bindgen` exported functions return small row windows / primitives
* **Goal**

  * Minimize JS allocations and bridge bandwidth

### 8.2) JS sees view of Rust memory

* **Interop tools**

  * `WebAssembly.Memory` + `Uint8Array`
* **Approach**

  * JS reads directly from WASM memory (careful with lifetimes / relocation)

### 8.3) Rust passes/copies values directly to JS

* **Bridge**

  * `wasm-bindgen` function returns arrays/primitives (copy across boundary)
* **Approach**

  * Used when the returned data is small enough that copying is acceptable

### 8.4) Rust passes/copies values directly to HTML

* **Tools**

  * `web-sys` DOM APIs (Rust manipulates DOM or emits events)
* **Approach**

  * Rust directly updates UI nodes (bypasses JS computations)

---

## 9) Rust/WASM → JS UI presentation

### 9.1) Rust/WASM renders directly

* **Rendering tools**

  * Canvas/WebGL
  * Optionally `wgpu`
  * Optionally `egui` (canvas-driven UI)
* **Approach**

  * JS receives no data; JS only hosts canvas & forwards input events

### 9.2) Rust/WASM returns minimal “signals” to JS

* **Bridge tools**

  * `wasm-bindgen` exported functions
  * `CustomEvent` notifications
* **Approach**

  * JS acts as “renderer only”; Rust provides small primitives

### 9.3) Island architecture component

* **Framework tools**

  * `Leptos` (DOM-based Rust UI in a bounded div)
  * `wasm-bindgen` for mounting + JS calls
  * `CustomEvent` for Rust→JS notifications

---

## 10) JS UI → Rust/WASM request patterns

### 10.1) JS UI interaction triggers request to Rust/WASM

* **Tools**

  * `wasm-bindgen` exported functions (JS calls Rust)
  * Possibly `CustomEvent` (JS listens to Rust state changes)
* **Approach**

  * JS forwards user actions; Rust/WASM executes all logic

---

## Memory models

### 10.1) Snapshot buffer lifetime model

* **Relevant tools**

  * `rkyv` archived snapshots (buffer owns everything)
  * Arrow IPC buffers / record batches as immutable backing memory
* **Approach**

  * One backing buffer per snapshot; free all at once

### 10.2) Mutable buffer repurposing

* **Relevant tools**

  * `bytemuck::from_bytes_mut` (in-place mutation)
  * `WebAssembly.Memory` views (JS writes into fixed buffers)
  * Rust `Vec::with_capacity` and controlled reuse

---

## 11) Initial load

### 11.1) Initial page load: bootstrap snapshot

* **Transport**

  * HTTP fetch (often best for big) or WebSocket
* **Format**

  * Arrow IPC File (tables) + optional ZSTD
  * rkyv snapshot (struct state)
* **Large payload mechanics (if needed)**

  * HTTP `ReadableStream` direct-to-WASM memory for huge initial state

### 11.2) Initial page load with early render (progressive tabular)

* **Format**

  * Arrow IPC Streaming format
* **Transport**

  * HTTP streaming or WebSocket streaming
* **Client ingestion**

  * `StreamReader` reads batches; UI can render as batches arrive
  * Virtualization uses `DataFrame::slice` as needed

---

If you want, the next step is to turn this into a **single canonical “recipe list”** where each scenario becomes:

* Inputs (request shape)
* Transport
* Encoding
* Client assembly
* Client storage/update (Polars/rkyv/bytemuck)
* JS presentation contract

