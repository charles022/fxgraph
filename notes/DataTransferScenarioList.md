


# Table of Contents
## struct from server to client rust/WASM
## Tabular Data from server to client rust/WASM
## Large Payload Transfer from server to client
## Client to Server Request Patterns
## Rust/WASM to JS communication
## memory models


---


## struct from server to client rust/WASM

## 1) custom rust structs shared between client / server (Shared Rust Structs)

### 1.1) shared struct: variable size (contains vec), read only
* server sends a shared Rust struct
* struct may contain `Vec`, `String`, nested data
* read only

### 1.2) shared struct: variable size (contains vec), mutable
* server sends a shared Rust struct
* struct may contain `Vec`, `String`, nested data
* note: may use the same method as when read-only, but with an extra step before it becomes mutable

### 1.3) shared struct: fixed-size structured state, read only
* server sends a fixed-layout POD struct
* no `Vec`, no `String`, only fixed arrays

### 1.4) shared struct: fixed-size structured state, mutable in place
* server sends a fixed-layout POD struct
* no `Vec`, no `String`, only fixed arrays


---


## 2) merge into persistent state - incremental state evolution
* client holds a large persistent state
* server sends small updates
* client puts updates into persistent state
* avoids moving, reallocating or replacing full state
* suitable for incremental state evolution
* apply for table and struct, static and dynamic size

### 2.1) merge into persistent state - incremental state evolution - (table, dynamic)
* apply for updates to a DataFrame with a variable number of rows

### 2.2) merge into persistent state - incremental state evolution - (table, fixed)
* apply for updates to a DataFrame with a fixed number of rows

### 2.3) merge into persistent state - incremental state evolution - (struct, fixed)
* apply for updates to a fixed size struct

### 2.4) merge into persistent state - incremental state evolution - (struct, dyn)
* apply for updates to a struct that may hold a vec field


---


## Tabular Data from server to client rust/WASM

### 3) Replace entire client DataFrame (full refresh)

* Rust/WASM client already holds a DataFrame
* server sends a complete replacement dataset
* uses the same Arrow format (columns, datatypes, etc) as the original

### 3.1) replace DataFrame: different number of rows in the old and new DataFrame
* replace DataFrame: different number of rows in the old and new DataFrame

### 3.2) replace DataFrame: same fixed number of rows in old and new DataFrame
* replace DataFrame: same fixed number of rows in old and new DataFrame


---


### 4) Append new data to an existing DataFrame (incremental growth)
* client holds an existing Polars DataFrame
* server sends “new rows since last update”

### 4.1) Append to DataFrame: Small frequent streaming appends (real-time updates)
* server sends small batches repeatedly (milliseconds / seconds)
* client maintains a growing “live” table
* low-latency prioritized

### 4.2) Append to DataFrame: Append with batching to avoid fragmentation (Arrow Builders hybrid)
* server sends very small updates at high frequency
* client should not `vstack` per update
* client buffers rows in Arrow Builders
* builders reserve contiguous memory upfront
* builders flush on row-count or time threshold
* flushed batches are appended to main table
* reduces fragmentation and SIMD penalties


### 4.5) Insert into DataFrame
* server to client update also involves removing some values from the client table


---


## Large Payload Transfer from server to client

### 5.1) Downloads
* directly to browser
* no rust/WASM interaction

### 5.2) Chunked streaming assembly
* server sends total size first
* payload sent in fixed-size chunks
* chunks appended to object in rust/WASM layer
* chunk/buffer cleared in mandatory JS/browser layer
* repeat
* avoids 2x memory allocation when copying from JS to rust/WASM
* manageable memory profile

### 5.3) HTTP fetch streaming + direct-to-WASM memory
* server exposes HTTP endpoint
* client uses `fetch` with `ReadableStream`
* server sends content length header
* Rust/WASM allocates final buffer once
* JS writes bytes directly into WASM memory view
* minimal JS heap usage

### 5.4) Compression / Encoding Strategy Patterns
* compress data before network transfer
* incorporate compression as a variation on the listed transfer methods


---


## Client to Server Request Patterns

### 6.1) Typed request struct (field-based queries)
* client sends a Rust struct
* fields specify column, value, limit, etc.
* server deserializes struct
* server executes logic directly
* returns tabular or structured response


---


## Rust/WASM to JS communication

## 8) Rust/WASM to JS, return data

### 8.1) Client-side slice for virtualization (visible rows only)
* client holds a large DataFrame / Arrow table
* JS UI only needs currently visible rows
* Rust/WASM slices table (`df.slice(offset, len)`)
* only sliced rows converted to JS values
* minimizes bridge traffic and JS allocations

### 8.2 JS sees view of rust memory
* JS views data at address where Rust/WASM holds data
* variations: from struct and DataFrame

### 8.3) Rust passes/copies values directly to JS
* Rust passes/copies values directly to JS
* variations: from struct and DataFrame

### 8.4) Rust passes/copies values directly to HTML
* Rust passes/copies values directly to JS


---


## 9) Rust/WASM to JS, UI presentation

### 9.1) Rust/WASM renders directly
* Rust/WASM owns rendering
* draws directly to
* JS receives no data
* eliminates bridge cost entirely
* preferred where possible

### 9.2) Rust/WASM returns minimal “signals” to JS
* JS UI reacts to small primitives
* strings, numbers, flags only
* avoids passing large data blobs
* suitable for UI state indicators

### 9.3) Island architecture component
* Rust/WASM owns a specific `<div>`
* JS treats it as a black box
* JS calls exported Rust functions
* Rust emits browser events
* clean separation of concerns


---


## 10) JS UI to Rust/WASM Request Patterns

## 10.1) JS UI to Rust/WASM Request Patterns
* JS UI interaction triggers request to Rust/WASM layer


---


## memory models

## 10) Data Lifetime & Buffer Semantics, memory model

### 10.1) Snapshot buffer lifetime model
* all objects tied to one backing buffer
* cannot free individual elements
* entire buffer freed at once
* acceptable for snapshots and bulk views

### 10.2) Mutable buffer repurposing
* fixed-size data reused in place
* buffer contents updated without reallocation
* suitable for fast state updates


---


## 11) Initial load

### 11.1) Initial page load: “bootstrap snapshot”
* user opens the dashboard
* Rust/WASM client starts empty
* baseline dataset / state required to render initial UI
* “initial login / initial load”
* large transfer considerations (memory spike, streaming vs single blob)

# 11.2) initial page load, w/ early render
* progressive tabular
* render initial rows to UI while remaining dataset finishes arriving


