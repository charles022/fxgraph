# fxgraph Project Context

## Project Overview
**fxgraph** is a high-performance dashboard platform architecture designed to minimize latency and memory overhead by leveraging a **Rust Server <-> Rust/WASM Client** model.

The core philosophy is "Zero-Copy & Zero-Parse": avoiding JSON/text parsing in favor of raw binary transfers (Arrow IPC, rkyv, bytemuck) that map directly to memory structures in both the server and the WASM client.

### Key Technologies
*   **Core Language:** Rust (used for both Backend and Frontend/WASM).
*   **Data Analysis:** Polars (DataFrame engine) and Apache Arrow (Memory format).
*   **Transport:** WebSockets (Real-time updates), HTTP Fetch (Bulk/Streamed data).
*   **Serialization:**
    *   **Arrow IPC:** For tabular data (Zero-copy ingestion into Polars).
    *   **rkyv:** For complex structs (Zero-copy deserialization).
    *   **bytemuck:** For fixed-size POD state (In-place mutation).
*   **UI Architecture:** Canvas/WebGL (Primary), "Island Architecture" (Secondary, via Leptos/Rust-native).

## Repository Structure
This repository primarily serves as the **Architecture Design and Proof-of-Concept (POC)** workspace.

*   **Root Documentation:**
    *   `README.md`: High-level vision and philosophy.
    *   `DataTransferScenarioList.md`: Comprehensive list of data transfer scenarios (server -> client).
    *   `DataTransferToolMap.md`: Decision matrix mapping scenarios to the best tools (e.g., "Use Arrow IPC for Tables, rkyv for Structs").
    *   `notes_implementation.md`: Detailed engineering notes and tradeoffs (Polars vs. DataFusion, etc.).

*   **Reference Code:**
    *   `manual_arrowrs_examples.rs`: Standalone examples of using `arrow-rs` directly without Polars. Kept for future reference.

## Development & Usage

### Key Architectural Patterns
When implementing new features, adhere to the patterns defined in `DataTransferToolMap.md`:

1.  **Tabular Data (Streams):** Use **Arrow IPC Streaming** over WebSockets. Ingest into Polars via `vstack` or Arrow Builders.
2.  **Tabular Data (Snapshots):** Use **Parquet** (ZSTD) or **Arrow IPC File** over HTTP Fetch for direct-to-memory assembly.
3.  **Fixed State:** Use **bytemuck** to cast raw bytes to `#[repr(C)]` structs for instant updates.
4.  **Complex Structs:** Use **rkyv** for zero-copy archival/retrieval.

### Tools Required
*   **Rust Toolchain:** `cargo`, `rustc`
*   **WASM Tooling:** `wasm-pack`
*   **Browser:** Modern browser with WebAssembly support.