





# fxgraph: High-Performance Rust/WASM Dashboard Architecture

## 1. Vision & Core Philosophy

**fxgraph** is a high-performance dashboard platform built on a **Rust Server** and **Rust/WASM Client** architecture. Unlike traditional Single Page Applications (SPAs) that rely heavily on JavaScript, we treat the browser primarily as a runtime for WebAssembly, utilizing JavaScript only for the necessary "bridge" to browser APIs and user UI.

### Key Tenets
*   - single language business logic is written in Rust and shared via common crates between the server and the WASM client
*   - Zero-Copy & Zero-Parse whenever possible and appropriate
*   - We reject JSON. Instead, we favor memory layouts that minimize or eliminate CPU-intensive parsing or allocation
*   - By keeping data in WASM linear memory, we avoid the heavy cost of JavaScript Garbage Collection (GC)



# Main proposal:
- Rust server <-> Rust/WASM (client) <-> custom JavaScript UI
- rust server
    - single node machine
    - all tablular data exists in a Polars table in memory
        - does not need to request/pull data from external sources like
          drive/disk, other nodes, or from over the network
    - non-tabular data lives in custom structs
    - the initial load of data is pulled from parquet on the same server machine
- rust/wasm client
    - interacts with the rust server and JS UI
    - receives and interprets requests from JavaScript for all updated data to
      display (ie when scrolling, filtering, aggregating data)
    - JS UI initiaes requests to the rust/WASM layer, and when needed, the
      rust/WASM layer sends the request to the rust server. JS layer does not
      directly send requests to the rust server.
- JavaScript UI
    - custom layer for displaying content and interacting with the user
    - does not perform computations like data aggregation, nor does it send
      requests to the server. These needs are communicated to the rust/WASM
      layer which then performs computations and/or communicates with the rust
      server then returns the minimal relevant result to be displayed by the JS
      layer
- use/control both ends with the same language (Rust)...
- share the exact struct definitions via shared crate and table format/layout
  (arrow)
- use the best method for transfering and accessing data based on the specifics of the data and its use
- reject the use of JSON


# proposed future enhancements
    - add a proxy/router ahead of the server to distibute requests to the first
      available server, as all data will live on each server
    - (phase 2) wrap in rayon for handling multiple client requests

---
