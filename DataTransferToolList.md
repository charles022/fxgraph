





---

# Table of Contents
## 1) Network Transport / Protocols
* WebSocket
* HTTP / Fetch API
* gRPC
* gRPC-Web
* Apache Flight

## 2) Serialization / Memory Encoding Crates
* rkyv
* bincode
* bytemuck
* zerocopy

## 3) Arrow / Columnar Data Ecosystem
* Apache Arrow (arrow-rs)
* Arrow IPC
* Arrow IPC Streaming Format
* Arrow IPC File Format
* Arrow Builders
* Arrow Compute Kernels (`arrow::compute`)

## 4) Polars DataFrame Ecosystem
* Polars
* DataFrame::vstack
* DataFrame::rechunk
* DataFrame::slice
* polars_sql::SQLContext

## 5) Compression & Writers
* ZSTD
* IpcWriteOptions
* Parquet

## 6) Arrow IPC Writers / Readers
* FileWriter
* StreamWriter / IpcStreamWriter
* StreamReader

## 7) WASM / Browser Interop
* WebAssembly.Memory
* Uint8Array
* ReadableStream

## 8) wasm-bindgen
* wasm-bindgen
* #[wasm_bindgen]

## 9) UI / Rendering
* Canvas / WebGL
* wgpu
* web-sys
* egui
* Leptos

## 10) JS ↔ Rust Communication
* Exported Rust Functions
* CustomEvent

## 11) Memory / Buffer Operations
* Vec::with_capacity
* Vec::set_len
* bytemuck::from_bytes / bytemuck::from_bytes_mut

## 12) Concurrency Models
* Rayon
* Tokio

## 13) Explicitly Avoided
* JSON
* Protobuf (Browser)
* Apache DataFusion (Client)


# End of Table of Contents


---


# 1) Network Transport / Protocols

## WebSocket
- A persistent, bidirectional connection between browser and server that allows sending binary frames (`ArrayBuffer`) in either direction.
- Why it appears in your design
  - Enables low-latency request/response and push updates
  - Works naturally with binary data (Arrow IPC, rkyv blobs)
  - Avoids polling and repeated HTTP setup costs
- Strengths
  - Ideal for real-time updates and interactive dashboards
  - Simple mental model (send message → receive message)
  - Works well with Arrow IPC Streaming format
- Weaknesses
  - Browser buffering can cause memory spikes for very large payloads
  - Limited backpressure control compared to HTTP streams
  - Framing overhead per message

---

## HTTP / Fetch API

- Standard browser HTTP requests accessed via `fetch()`, optionally exposing a `ReadableStream` for incremental reading.
- Why it appears in your design
  - Best browser-optimized path for large one-directional transfers
  - Supports true streaming consumption with backpressure
  - Avoids WebSocket buffering pitfalls for very large payloads
- Strengths
  - Native backpressure support
  - Efficient for multi-MB to 100MB+ payloads
  - Enables direct-to-WASM memory streaming
- Weaknesses
  - Not bidirectional
  - Less convenient for interactive request/response workflows

---

## gRPC
- A language-agnostic RPC framework using Protobuf over HTTP/2.
- Why it is mentioned
  - Serves as a comparison point
- Why it is rejected
  - Browsers cannot make native gRPC calls
  - Requires gRPC-Web + proxy + translation
  - Optimized for cross-language compatibility, not raw speed

---

## gRPC-Web
- A browser-compatible wrapper around gRPC that tunnels Protobuf over HTTP.
- Why it is rejected
  - Adds base64 framing overhead
  - Requires Envoy or similar proxy
  - Introduces extra parsing and translation layers
  - Undermines Rust-to-Rust shared memory layouts

---

## Apache Flight
- An Arrow-based RPC protocol built on gRPC.
- Why it exists
  - Efficient Arrow transport between backend services
- Why it is rejected here
  - Built on `tonic` and gRPC
  - Does not compile cleanly to WASM
  - Requires JS bindings, defeating Rust-first logic

---

# 2) Serialization / Memory Encoding Crates

## rkyv
- A Rust-native zero-copy serialization framework.
- What it does
  - Archives Rust structs into byte buffers with relative pointers
  - Allows direct access without deserialization
- Why it fits your design
  - Shared Rust structs on server and client
  - WASM reads complex data without allocation
  - Eliminates JS parsing entirely
- Tradeoffs
  - Data is immutable unless deserialized
  - Lifetimes tied to the backing buffer
  - Requires strict schema discipline

---

## bincode
- A compact binary serializer for Rust.
- What it does
  - Serializes Rust structs into bytes
  - Requires full deserialization on client
- Why it appears
  - Simple fallback when zero-copy is unnecessary
  - Useful when mutability is required post-receipt
- Tradeoffs
  - Allocations during deserialize
  - Slower than rkyv for large or frequent transfers

---

## bytemuck

- A crate for safely reinterpreting byte buffers as Rust structs.
- What it enables
  - Zero-copy casting to `&T` or `&mut T`
  - In-place mutation of received data
- Constraints
  - `#[repr(C)]`
  - Fixed-size only
  - No pointers, `Vec`, or `String`
- Why it matters
  - Fastest possible path for small state updates
  - Ideal for physics state, counters, matrices

---

## zerocopy
- A POD-oriented zero-copy memory interpretation crate.
- Role
  - Reinforces the “no deserialize, no allocation” approach
  - Fixed-layout data only

---

# 3) Arrow / Columnar Data Ecosystem

## Apache Arrow (arrow-rs)
- A standardized columnar in-memory data format.
- Why it matters
  - Underlies Polars
  - Enables SIMD-friendly computation
  - Preserves column semantics across network transfers

---

## Arrow IPC
- A binary serialization format for Arrow data.
- What it preserves
  - Column layout
  - Data types
  - Nullability
  - Zero-copy semantics
- Why it’s central
  - Server and client exchange tables without conversion
  - WASM Polars ingests directly

---

## Arrow IPC Streaming Format

- Sequential format: schema followed by record batches.
- Why it’s used
  - Enables progressive rendering
  - Supports real-time updates
  - Avoids waiting for full dataset arrival

---

## Arrow IPC File Format

- A self-contained Arrow file with footer.
- Why it’s used
  - Ideal for full snapshots
  - Supports compression
  - Easy to transmit as a blob

---

## Arrow Builders

- Mutable column builders (`Int32Builder`, `StringBuilder`, etc.).
- Why they matter
  - Avoid repeated `vstack` fragmentation
  - Enable batching of small updates
  - Maintain contiguous memory
- Core benefit
  - Converts many tiny updates into one clean append

---

## Arrow Compute Kernels (`arrow::compute`)
- Low-level column operations.
- Examples
  - `filter`
  - `take`
  - `sort_to_indices`
- Why mentioned
  - Enables Arrow-purist workflows
  - Maximum control with minimal dependency footprint

---

# 4) Polars DataFrame Ecosystem

## Polars
- A high-performance DataFrame engine built on Arrow.
- Why you use it
  - Complex operations (groupby, joins)
  - Optimized SIMD kernels
  - Familiar DataFrame semantics
- Tradeoff
  - Larger WASM binary than raw Arrow

---

## DataFrame::vstack

- Appends rows by linking chunks.
- Why it’s fast
  - Zero-copy
- Why it’s dangerous
  - Causes fragmentation
  - Degrades SIMD performance over time

---

## DataFrame::rechunk

- Rebuilds contiguous memory.
- Why it’s necessary
  - Restores performance after many appends
  - Explicit, controllable cost

---

## DataFrame::slice
- Returns a row window without copying.
- Why it’s critical
  - Enables UI virtualization
  - Minimizes JS bridge traffic

---

## polars_sql::SQLContext
- Executes SQL queries over Polars.
- Why it exists
  - Allows string-based queries from client
  - Trades runtime parsing for flexibility

---

# 5) Compression & Writers

## ZSTD
- A general-purpose compression algorithm.
- Why used
  - Fast decompression
  - Good compression ratio
  - Supported natively by Arrow and Parquet

---

## IpcWriteOptions
- Arrow IPC writer configuration.
- Why important
  - Enables compression
  - Controls streaming vs file behavior

---

## Parquet
- Columnar disk format for Arrow/Polars.
- Why preferred for large history
  - Column-aware compression (RLE, delta)
  - Smaller than IPC+ZSTD
  - Higher encode cost, better storage density

---

# 6) Arrow IPC Writers / Readers

## FileWriter
- Writes Arrow IPC File format.
- Use cases
  - Initial load
  - Large snapshots

---

## StreamWriter / IpcStreamWriter
- Writes Arrow IPC Streaming format.
- Use cases
  - Real-time updates
  - Progressive rendering

---

## StreamReader
- Reads Arrow IPC streams incrementally.
- Use cases
  - Client ingestion during streaming

---

# 7) WASM / Browser Interop

## WebAssembly.Memory
- Linear memory backing WASM.
- Why it matters
  - JS can write directly into it
  - Enables direct-to-memory streaming

---

## Uint8Array
- JS view over raw bytes.
- Why it matters
  - Bridges network → WASM memory
  - Enables zero-copy views

---

## ReadableStream
- Browser streaming abstraction.
- Why it matters
  - Enables incremental reads
  - Backpressure-aware

---

# 8) wasm-bindgen

## wasm-bindgen
- Rust ↔ JS interop glue.
- Why essential
  - Exposes Rust APIs to JS
  - Enables safe memory sharing

---

## #[wasm_bindgen]
- Attribute to export Rust functions and structs to JS.

---

# 9) UI / Rendering

## Canvas / WebGL
- Pixel-based rendering target.
- Why preferred
  - Avoids DOM bottlenecks
  - Scales to large datasets

---

## wgpu
- Rust GPU abstraction.
- Role
  - High-performance rendering backend

---

## web-sys
- Low-level browser API bindings.

---

## egui
- Immediate-mode Rust UI framework.
- Why mentioned
  - Extremely fast
  - Canvas-based
  - No DOM overhead

---

## Leptos
- Rust WASM UI framework.
- Why mentioned
  - DOM-based
  - Useful for island architecture

---

# 10) JS ↔ Rust Communication

## Exported Rust functions
- JS calls into Rust for control and state updates.

---

## CustomEvent
- Browser event emitted by Rust.
- Why
  - Enables Rust → JS notifications without tight coupling

---

# 11) Memory / Buffer Operations

## Vec::with_capacity
- Preallocates buffer once for known-size payloads.

---

## Vec::set_len
- Marks buffer filled after streaming.
- Unsafe but required for direct-to-memory assembly.

---

## bytemuck::from_bytes / from_bytes_mut
- Zero-copy casting APIs.
- Enables immutable or mutable access to network data.

---

# 12) Concurrency Models

## Rayon
- Thread-pool based parallelism.
- Used for CPU-bound Polars workloads.

---

## Tokio
- Async I/O runtime.
- Mentioned primarily in contrast to Rayon and DataFusion.

---

# 13) Explicitly Avoided

## JSON
- High parsing cost
- Heavy JS GC pressure

---

## Protobuf (browser)
- Parsing overhead
- JS object allocation cost

---

## Apache DataFusion (client)
- Async scheduling overhead
- Large WASM footprint
- Reserved for backend use only

---




