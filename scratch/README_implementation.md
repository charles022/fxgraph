

## 1. The Data Pipeline

We do not use a "one size fits all" REST API. Instead, we select the optimal transport and encoding based on the data shape and frequency.

### Transport Layer
*   **WebSockets:** The default for real-time, bidirectional communication, control signals, and frequent small updates.
*   **HTTP Streaming (Fetch API):** Used for large initial payloads or massive dataset transfers. This allows for **"Direct-to-Memory Assembly"**, where data is streamed from the network directly into a pre-allocated WASM memory buffer, bypassing the JavaScript heap entirely.

### Serialization & Memory Strategy
| Data Type | Tooling | Why? |
| :--- | :--- | :--- |
| **Tabular Data** | **Arrow IPC** | The native memory format for our analytics engine (Polars). Allows zero-copy ingestion of DataFrames. |
| **Complex Structs** | **rkyv** | Guarantees zero-copy deserialization. The client reads the network buffer directly as if it were a struct, with no parsing step. |
| **Fixed State (POD)** | **bytemuck** | "Plain Old Data". We cast raw byte arrays into mutable Rust structs (`&mut T`) for instant physics/state updates. |
| **Simple Messages** | **bincode** | Low-overhead binary serialization for simple request/response patterns where owning the data is necessary. |

---

## 2. Tabular Data & Analytics

The core of the dashboard is powered by **Polars** and **Apache Arrow** running inside the WASM client.

*   **Ingestion:** The server streams **Arrow IPC** bytes (Streaming or File format). The WASM client consumes these bytes directly into a Polars `DataFrame`.
*   **Updates:**
    *   **Full Refresh:** Replace the internal DataFrame pointer with a new Arrow IPC snapshot.
    *   **Streaming Appends:** Use **Arrow Builders** to buffer high-frequency updates and flush them into the main DataFrame to avoid memory fragmentation.
*   **Visualization:** We employ **Virtualization**. The Rust engine holds the full dataset (millions of rows) but only "slices" (`df.slice`) the currently visible rows to the UI, keeping the DOM lightweight.

---

## 3. UI & Rendering Strategy

We aim to bypass the slow DOM wherever performance is critical.

*   **Primary (High Perf):** **Canvas / WebGL / wgpu**. Data is rendered directly from WASM to a canvas element. The browser sees only a single image, eliminating DOM layout thrashing.
*   **Secondary (Interactive):** **"Island Architecture"**. Rust components (potentially using Leptos) mount themselves into specific HTML `<div>`s.
*   **Signals:** Rust sends minimal primitive signals (integers, strings) to JavaScript to trigger lightweight UI updates (e.g., updating a counter or toggling a modal) without passing heavy data objects.

---

## 4. Architectural Comparison

### Why not JSON?
JSON requires parsing text into thousands of small JavaScript objects (Object, Array, String). This creates massive **GC Pressure**, causing frame drops and lag. Our binary approach allocates a single buffer in WASM memory that the JS GC ignores.

### Why not gRPC-Web?
gRPC-Web requires a proxy (like Envoy) to translate HTTP/2 to HTTP/1.1 and often adds Base64 encoding overhead. It is optimized for polyglot backend microservices, not for raw browser performance. Our approach uses standard WebSockets and HTTP with raw binary payloads, removing the need for proxies and translation layers.
