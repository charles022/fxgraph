








Main proposal:
-
    (Rust server) - gRPC - JS
    with
    Rust server - rkyv - Rust/WASM


Proposed Rust + WASM approach:
    - Rust Server: Dumps memory directly to a binary stream (Serialization)
    - Network: Transmits raw bytes.
    - WASM Client: Loads bytes directly back into Rust Structs (Deserialization).
        - no deserialization if using rkyv, zero-copy

(( alternate to the current... ))
((  - Rust Server: Converts Rust Struct -> Protobuf Binary. ))
((  - Network: Transmits Protobuf (often wrapped in gRPC-Web text/base64 framing). ))
((  - JS Client: Parses Protobuf $\rightarrow$ JavaScript Objects. ))

- use/control both ends with the same language (Rust)...
- share the exact struct definitions via a common library (crate)...
- allows us to use **serialization** formats that are strictly coupled to code
    - (extremely fast, faster than gRPC)



Why is this faser han gRPC?
-
    - gRPC (Protobuf) is designed to be language agnostic
    - handles fields that are option, out of order, or versioned
    - whereas, rust-native formats dont always need those checks

Zero-Copy possible using rkyv (pronounced "archive")
-
    - Traditional (Serde/Bincode): The server turns the object into bytes. The client reads
      bytes and allocates new memory to recreate the object.
    - Zero-Copy (rkyv): The server aligns the data in memory and sends it. The client
      receives the buffer and casts a pointer to it.
    - There is effectively no deserialization step. The client can use the data strictly as
      it arrived over the network without the CPU doing any work to "parse" it.


Current Architecture (gRPC + JS):
-
    - Protocol - gRPC-Web
    - Serialization - Protocol Buffers - compact but requires parsing
    - Client - JS / TypeScript
    - Type Safety - enforced via .proto files generation
    - Bottleneck - JS parsing overhead & garbach collection

Proposed Architecture (Rust/WASM):
-
    - Protocol - WebSocket or HTTP/2 Binary Strams
    - Serialization - Bincode (small, fast) or rkyv (instant)
    - Client - Rust compiled to .wasm
    - Type Safety - enforced by Rust compiler (shared crates [/struct/module?]
    - Bottleneck - WASM/JS "bridge" overhead


Proposed project workspace structure:
-
// ./shared/src/lib.rs ... lib crate
    #[derive(Serialize, Deserialize)] // or #[derive(Archive)] for rkyv
    pub struct GameState {
        pub score: u64,
        pub positions: Vec<(f32, f32)>,}

// ./server/ ... bin crate
    - imports 'shared' (ie ./shared/src/lib.rs?)
    - uses 'bincode::serialize' to send data over a WebSocket or HTTP response
// ./client/ ... WASM crate
    - imports 'shared'
    - uses 'fetch' or 'ws' to get the bytes, then
    - 'bincode::deserialize'

Solutions to the "bridge" tax of moving data from Rust/WASM to JS
-
    - Canvas/WebGL ('wgpu' or 'web-sys')
        - preferred method where possible
        - draw he data to a <canvas> so WASM never needs to talk to JS
    - Signals
        - only pass specific strungs/numbers needed for the UI to JS, rather
          than the entire object
    - Frameworks (Leptos, Dioxus, Yew)
        - rust frameworks that handle the WASM/DOM interaction efficiently
        - may try these down the road, but for now do not use, as we are
          trying to keep this build minimal and avoid additional frameworks


Summary/notes:
-
    - Rust (server) - Rust/WASM (client)
    - transfer w/
        - zero-copy
            - rkyv
            - #[derive(Archive)]
        - serializeserialize/deserialize w/ share structs otherwise
            - bincode::deserialize
            - #[derive(Serialize, Deserialize)]
    - render WASM w/
        - Canvas/WebGL, wgpu or web-sys, draw data to a <canvas>
        - DO NOT USE: possible future frameworks: Leptos, Dioxus, Yew
./
├── shared
│   ├── Cargo.toml
│   └── src
│       └── lib.rs // shared structures
├── server
│   ├── Cargo.toml
│   └── src
│       └── ___.rs
├── client
│   ├── Cargo.toml
│   └── src
│       └── ___.rs
└── ...





Apache Flight
-
    - may be overkill and actually slower
    - additional  archiectural complexity in the browser
    - significant hurdles when running inside a web
      browser
    - built on gRPC/HTTP2 - browsers can not make
      direct gPRC calls
    - server must first translate gPRC -> gPRC-Web
      (usually w/ proxy)
    - serialization translation, network hop,
      infrastructure complexity
vs Custom Rust Binary:
-
    - over standard **WebSockets**
    - direct TCP-like connection
    - send raw binary bytes from server
    - browser receives raw binary bytes
      **(ArrayBuffer)**
    - no proxies, headers, translation

!! important trap !!
-
    - official Rust 'arrow-flight' crate relies on
      'tonic' (a gRPC library) and tokio, which
      generatlly **do not compile to
      'wasm32-unknown-unknown** because they expect
      system TCP sockes, which the browser sandbox
      forbids
    - workaround:
    - ... to use Flight in WASM, often have to use
      JavaScript Filght client and "bind" it to the
      Rus code, **which defeats the purpose of
      writing our logic in Rust**
    - why our solution works better:
    - ... rkyv and bincode are no_std compatible and
      work flawlessly in WASM out of the box

(sweet spot) Arrow ICP over WebSockets
-
    - if using Polars, use the Arrow memory format,
      but avoid the "Flight" protocol wrapper

Best Hybrid Approach: Polars + Arrow IPC:
-
    - for straming raw bytes
    - Zero-Copy benefits of Arrow...
      ... w/o weight of gRPC/Fight
    - Server: Use Polars/Arrow to create a DataFrame
    - Serialize: Dump the dataframe to Arrow IPC
      Streaming Format (raw bytes).
    - Transport: Send those bytes over a WebSocket.
    - Client (WASM): Read the bytes directly into a
      Polars DataFrame (using polars_core or
      arrow-wasm).


Feature Comparison: Flight vs Arrow IPC vs Custom Binary:
-
    - Transport:
        - Apache Flight: gRPC-Web (requires proxy)
        - Arrow IPC over WebSocket: WebSocket (direct)
        - Custom Binary (rkyv): WebSocket (direct)

    - Data Format:
        - Apache Flight: Arrow (columnar)
        - Arrow IPC over WebSocket: Arrow (columnar)
        - Custom Binary (rkyv): Rust struct (row-based)

    - Deserialization:
        - Apache Flight: Zero-copy (mostly)
        - Arrow IPC over WebSocket: Zero-copy
        - Custom Binary (rkyv): Zero-copy (instant)

    - Browser Support:
        - Apache Flight: Poor (requires JS wrappers)
        - Arrow IPC over WebSocket: Excellent
        - Custom Binary (rkyv): Excellent

    - Best For...:
        - Apache Flight: Inter-service (Backend-to-Backend)
        - Arrow IPC over WebSocket: Polars/DataFrames in Browser
        - Custom Binary (rkyv): Game state / Simple structs


When sending tabular data (Polars DataFrames), do
not use Flight. Instead, stream Arrow IPC bytes over
a WebSocket. This preserves the efficient columnar
layout (which Polars needs) but avoids the
gRPC/Envoy complexity that Flight requires in the
browser.

Verdict: Polars v Arrow in browser:
-
    - use Polars in browser
    - in addition...
    - can use arrow-rs kernels for small light actions
    - sort/filter are not too bad w/ arrow kernels
    - groupby is untennable w/ arrow kernels

Arrow v Polars:
-
    - TL;DR...
    - Use Arrow when...
        - sort/filter is the extent
        - goal is just Data Access
        - ie getting data from server to screen
        - sort, filter, etc arrow kernels are not
          bad but still must be paired with
          additional code logic
    - Use Polars when...
        - doing more serious data access/manipulation on browser
        - group by is untenable in Arrow
    - Arrow is significantly lighter than Polars
    - Arrow is not as capable in terms of high-level features
    - Arrow can build/do anything, but must assemble
      every piece yourself
        - likely leads to...
        - micro optimizations in rare cases
        - more bugs and code to do the same things
        - irrelevant amount of "weight" saved over Polars
    - arrow-rs is modular, can compile only the
      specific features needed
    - Arrow likely <1MB compressed
    - Polars likely 3-5MB compressed (heavy for a
      web browser )
    - "Raw Arrow" is a low-level memory format, not
      a data analysis library.

arrow-rs v Polars: code comparison:
-
    Filter:
    (Polars)
    - df.filter(col("age").gt(30))
    (arrow)
    - arrow::compute::filter kernel
    - compute a boolean mask array [true, false,
    ...], then use the filter kernel to create a new
    array using that mask
    - ...
    - ...
    Sort:
    (Polars)
    - df.sort("name")
    (arrow)
    - arrow::compute::sort/sort_to_indices kernel
    - use the 'sort_to_indices' kernel to get a list
      of index positions, then use the 'take' kernel
      to reorder every column manually based on
      those indices.
    - ...
    - ...
    Group by
    (Polars)
    - df.group_by("city").agg(sum("salary"))
    (Arrow)
    - **Extremely difficult**
    - manually hash the "city" column, map indices
      to buckets, and iterate through the "salary"
      column to sum values into those buckets


Apache DataFusion (avoid)
-
    - SQL engine built on Arrow
    - similar to Polars
    - much heavier, not optimized for browser like
      Polars is




Arrow Purist: Streaming Raw Arrow / Arrow IPC Ingestion:
-
    - open a WebSocket and stream raw bytes
    - do not need 100% of data to arrive before we can access it
    - 1) Arrow Schema arrives (the first 1KB)
    -    Polars (which uses Arrow under the hood) knows the column names
    - 2) First N rows arrive (the next nKb)
    -    Polars can render these rows on the screen
    - ...
    - (server): serializes a RecordBatch into Arrow IPC Streaming format
        basically a raw memory dump
    - (network): sends bytes
    - (client): receives the Uint8Array. Pass this array into
        arrow::ipc::reader::StreamReader.

Arrow Purist: Storage
-
    - In Rust, our table would look like this...
    struct UserTable {
        ids: Int32Array,      // [1, 2, 3, ...]
        names: StringArray,   // ["Alice", "Bob", "Charlie", ...]
        scores: Float32Array, // [95.5, 88.0, 42.0, ...]}


Arrow Purist: code for manual manipulation
-
    - build exactly the features we need using the low level
      kernels: 'filter', 'take', 'sort_to_indices'
    - 'arrow::compute' module as the toolkit for SQL-esk actions
    - SQL: SELECT * WHERE scores > 90
    - arrow-rs: 
        - use 'gt_scalar' kernel on 'scores' array to return a
          bool bitmask [False, False, True, ...]
        - use 'filter' kernel on **every column** w/ bitmask
          filter(ids, mask) -> [1]
          filter(names, mask) -> ["Alice"]
          filter(scores, mask) -> [95.5]

Arrow Purist: render to UI
-
    - Strategy A: (preferred) only send the rows currently visible on the
      screen to JS (negligible cost)
        - user scrolls to rows 100-120
        - rust slices the arrays at 100-120
        - Rust converts just those rows to a JS Array/Object
        - JS renders 20 'div's
        - negligible cost
    - Strategy B: (advanced) shared buffer, pass a pointer to the
      Arrow memory directly to JS
        - JS uses the apache-arrow library to wrap a view arount
          the WASM memory
        - JS reads the data without Rust doing any work










    - handle massive datasets without overwhelming the browser
    - while maintaining instant interactivity for smaller, aggregated datasets

    - "Hybrid Data Strategy"... seamlessly switches between:
        **Client-Side Processing** (for summary data)
        **Server-Side Processing** (for massive granular data)
    within the same user interface.


## **2\. Architecture Overview**

### **The Tech Stack**

* **Backend:** **Rust** (High-performance systems language)
  * *Framework:* Tonic (gRPC implementation) \+ Tonic-Web (Browser compatibility).
  * *Role:* Data generation, heavy sorting, filtering, and aggregation.
* **Protocol:** **gRPC-Web** (Binary over HTTP)
  * *Role:* Strongly typed, bandwidth-efficient communication between browser and server.
  * *Schema:* Defined via Protocol Buffers (.proto).
* **Frontend:** **TypeScript** \+ **Next.js** (React)
  * *Client:* ConnectRPC (Modern gRPC client for web).
  * *State/UI:* TanStack Table (Headless UI library for data grids).

### **Architecture Diagram**

Code snippet

graph LR
    subgraph Browser \[Frontend: Next.js\]
        UI\[User Interface\]
        RegionTable\[Region Table \<br/\> (Client-Side Sort)\]
        ShipmentTable\[Shipment Table \<br/\> (Server-Side Sort)\]
        Connect\[ConnectRPC Client\]
    end

    subgraph Network \[gRPC-Web\]
        Proto\[Binary Stream\]
    end

    subgraph Server \[Backend: Rust\]
        Tonic\[Tonic gRPC Server\]
        MockDB\[Mock Database \<br/\> (10k Rows RAM)\]
    end

    UI \--\> RegionTable
    UI \--\> ShipmentTable
    RegionTable \-- "Fetch Once (All Data)" \--\> Connect
    ShipmentTable \-- "Fetch Page/Sort" \--\> Connect
    Connect \<--\> Proto \<--\> Tonic
    Tonic \<--\> MockDB

---

## **3\. The Hybrid Data Strategy**

This PoC specifically demonstrates two distinct patterns for handling data, proving that we are not locked into a single approach.

### **Pattern A: "Big Data" (Shipment Manifest)**

* **Use Case:** Granular records, logs, or datasets \> 10,000 rows (potentially millions).
* **Mechanism:** The browser acts as a "remote control." It holds only the data currently visible (e.g., 50 rows).
* **User Experience:** When the user sorts by "Weight," the browser sends a request to the server. The server sorts the full dataset and returns only the top 50 results. This ensures the browser never crashes from memory overload.

### **Pattern B: "Small Data" (Region Stats)**

* **Use Case:** High-level aggregations, summaries, or datasets \< 5,000 rows.
* **Mechanism:** The server calculates the aggregation *once* and sends the complete list to the browser.
* **User Experience:** Sorting and filtering are **instant** (0ms latency) because they occur in the browser's CPU. No network requests are made after the initial load.

---

## **4\. Project Structure**

Plaintext

/fxgraph
├── /proto  
│   └── dashboard.proto       \# Single Source of Truth (Schema)  
├── /backend  
│   ├── build.rs              \# Pre-build script: Compiles .proto to Rust  
│   ├── Cargo.toml            \# Rust dependencies  
│   └── src/main.rs           \# Server entry point, Mock DB, Business Logic  
└── /frontend  
    ├── buf.gen.yaml          \# Buf configuration for TS generation  
    ├── package.json         \# Dependencies (ConnectRPC, TanStack Table)
    ├── app/page.tsx         \# Main UI combining both tables
    ├── lib/client.ts         \# ConnectRPC Transport singleton  
    ├── components/  
    │   ├── columns.ts       \# TanStack Column Definitions
    │   ├── DashboardTable.tsx \# Server-Side Sort Implementation  
    │   └── RegionTable.tsx    \# Client-Side Sort Implementation  
    └── gen/                  \# Auto-generated TS types (DO NOT EDIT)

---

## **5\. Implementation Details**

### **The Schema (dashboard.proto)**

This file is the single source of truth. If this file changes, both the Backend and Frontend builds will fail, ensuring type safety.

Protocol Buffers

service AnalyticsService {
  // Pattern A: Big Data (Server does the work)
  rpc GetShipments (ViewRequest) returns (ViewResponse);

  // Pattern B: Small Data (Client does the work)
  rpc GetRegionStats (Empty) returns (RegionList);
}

### **The Rust Backend (backend/src/main.rs)**

* **Mock Data:** Generates 10,000 random shipments on startup.
* **CORS:** Configured using tower-http to allow the Next.js frontend to read gRPC headers (grpc-status).
* **Logic:** Implements a custom sort comparator to handle dynamic column sorting (e.g., sorting by Timestamp vs. Weight) on the server.

### **The Frontend Client (frontend/lib/client.ts)**

Uses **ConnectRPC** to communicate. This abstracts away the binary parsing.

TypeScript

// Example usage
const response \= await client.getShipments({
  pageNumber: 1,
  sort: { columnId: "cargoWeightKg", isAscending: false }
});

---

## **6\. Setup & Run Guide**

### **Prerequisites**

1. **Rust:** Install via rustup.
2. **Node.js:** Install Node v18+.
3. **Buf:** (Optional but recommended) for efficient code generation, though npm scripts handle it in this PoC.

### **Step 1: Start the Backend**

The backend must be running to serve gRPC requests.

Bash

cd backend
cargo run
\# Expected Output: Server listening on 0.0.0.0:50051

### **Step 2: Start the Frontend**

Open a new terminal window.

Bash

cd frontend
npm install
npm run dev
\# Expected Output: Ready in 2.3s. Listening on http://localhost:3000

### **Step 3: Verify**

Navigate to http://localhost:3000.

1. **Test Region Table:** Click the "Revenue" header. It should sort instantly. Check the Backend terminal—you should see **no** new logs.
2. **Test Shipment Table:** Click the "Weight" header. It should take a brief moment. Check the Backend terminal—you should see a log entry: Request received for page: 1\.

---

## **7\. Future Considerations (Post-PoC)**

* **Database:** Replace the in-memory Vec\<MockShipment\> with a connection pool to PostgreSQL (using sqlx) or ClickHouse for analytics.
* **Filtering:** The .proto file already supports FilterCriteria. We can implement SQL WHERE clause generation in Rust to support complex filtering.
* **Streaming:** For data exports (e.g., "Download CSV"), we can utilize gRPC Server Streaming to send data in chunks without buffering.




TECHNICAL




# **Technical Specification: Logistics Analytics Dashboard (PoC)**

Version: 1.0 (PoC)  
Date: November 23, 2025  
Architecture: Hybrid Client/Server Data Processing via gRPC-Web.

---

## **1\. Technology Stack**

### **Backend (Service Layer)**

* **Runtime:** Rust (Edition 2021\)  
* **Framework:** tonic (v0.12) \- Native gRPC implementation over HTTP/2.  
* **Web Interop:** tonic-web (v0.12) \- gRPC-Web translation layer for browser compatibility.  
* **Async Runtime:** tokio (v1.0+) \- Multi-threaded asynchronous execution.  
* **Middleware:** tower-http (v0.5) \- Handles CORS and HTTP layer instrumentation.  
* **Serialization:** prost (v0.13) \- Protocol Buffers implementation for Rust.

### **Frontend (Presentation Layer)**

* **Framework:** Next.js 14+ (React 18\)  
* **Language:** TypeScript 5+  
* **Network Client:** ConnectRPC (@connectrpc/connect) \- Type-safe gRPC client.  
* **Transport:** @connectrpc/connect-web \- Implements the gRPC-Web protocol over HTTP/1.1.  
* **State/UI Logic:** TanStack Table v8 (@tanstack/react-table) \- Headless data grid.

### **Interface Definition**

* **IDL:** Protocol Buffers v3 (proto3)  
* **Transport Protocol:** gRPC-Web (application/grpc-web+proto)

---

## **2\. Directory Structure & Key Files**

Plaintext

/fxgraph
├── /proto  
│   └── dashboard.proto       \# Single Source of Truth (Schema)  
├── /backend  
│   ├── build.rs              \# Pre-build script: Compiles .proto to Rust  
│   ├── Cargo.toml            \# Rust dependencies  
│   └── src/main.rs           \# Server entry point, Mock DB, Business Logic  
└── /frontend  
    ├── buf.gen.yaml          \# Buf configuration for TS generation  
    ├── package.json         \# Dependencies (ConnectRPC, TanStack Table)
    ├── app/page.tsx         \# Main UI combining both tables
    ├── lib/client.ts         \# ConnectRPC Transport singleton  
    ├── components/  
    │   ├── columns.ts       \# TanStack Column Definitions
    │   ├── DashboardTable.tsx \# Server-Side Sort Implementation  
    │   └── RegionTable.tsx    \# Client-Side Sort Implementation  
    └── gen/                  \# Auto-generated TS types (DO NOT EDIT)

---

## **3\. Interface Specification (dashboard.proto)**

The API defines two distinct data access patterns.

Protocol Buffers

syntax \= "proto3";  
package dashboard;

service AnalyticsService {  
  // Pattern A: Server-Side Processing (Pagination/Sorting)  
  rpc GetShipments (ViewRequest) returns (ViewResponse);

  // Pattern B: Client-Side Processing (Snapshot)  
  rpc GetRegionStats (Empty) returns (RegionList);  
}

message ViewRequest {  
  int32 page\_number \= 1;  
  int32 items\_per\_page \= 2;  
  SortOption sort \= 3;        // Optional: if null, default sort applies  
  repeated FilterCriteria filters \= 4;  
}

message SortOption {  
  string column\_id \= 1;  
  bool is\_ascending \= 2;  
}

// ... (See source for full message definitions)

---

## **4\. Backend Implementation Details**

### **4.1 Server Configuration (main.rs)**

The server enables the GrpcWebLayer to translate browser HTTP/1.1 requests into HTTP/2 gRPC calls.

Crucial Layer Ordering:  
Middleware layers are wrapped outside-in. CORS must be the outermost layer to handle preflight OPTIONS requests before they reach the gRPC logic.

Rust

Server::builder()  
    .accept\_http1(true)                   // Required for gRPC-Web  
    .layer(cors\_layer)                    // 1\. Handle CORS  
    .layer(GrpcWebLayer::new())           // 2\. Translate gRPC-Web \-\> gRPC  
    .add\_service(AnalyticsServiceServer::new(service))  
    .serve(addr)  
    .await?;

### **4.2 CORS Configuration**

Browsers enforce strict CORS for gRPC-Web. The tower-http configuration must expose specific gRPC headers, otherwise the client will throw generic protocol errors even on successful requests.

* **Allowed Origin:** MirrorRequest (or explicit localhost:3000)  
* **Exposed Headers:** grpc-status, grpc-message, grpc-status-details-bin

### **4.3 Data Logic**

* **Mock Data:** Vec\<MockShipment\> generated via rand at startup (10k rows).  
* **get\_shipments (Server Sort):**  
  1. Clones the in-memory vector.  
  2. Performs sort\_by based on request.sort.column\_id.  
  3. Applies skip((page-1) \* limit).take(limit).  
  4. Returns ViewResponse.  
* **get\_region\_stats (Client Sort):**  
  1. Returns a static Vec\<RegionStat\>.  
  2. No sorting/paging logic applied serverside.

---

## **5\. Frontend Implementation Details**

### **5.1 Code Generation**

TypeScript types are generated using buf (or protoc-gen-es).

* **Command:** npx buf generate ../proto  
* **Output:**  
  * \*\_pb.ts: TypeScript interfaces for messages.  
  * \*\_connect.ts: Service definition and method signatures.

### **5.2 ConnectRPC Client (lib/client.ts)**

Instantiates a singleton client using createGrpcWebTransport.

* **BaseUrl:** http://localhost:50051  
* **Transport:** Binary format (default).

### **5.3 TanStack Table Configuration**

Scenario A: Server-Side (Big Data)  
File: DashboardTable.tsx

* **Mode:** Manual.  
* **Configuration:**  
  TypeScript  
  manualPagination: true,  
  manualSorting: true,

* **Effect:** Sort/Page state changes trigger useEffect \-\> analyticsClient.getShipments. Table does *not* sort the data array locally.

Scenario B: Client-Side (Small Data)  
File: RegionTable.tsx

* **Mode:** Automatic.  
* **Configuration:**  
  TypeScript  
  getCoreRowModel: getCoreRowModel(),  
  getSortedRowModel: getSortedRowModel(), // Enables local sorting  
  // manualSorting is OMITTED/FALSE

* **Effect:** useEffect runs once on mount. Sort state changes are handled entirely by the JS engine via getSortedRowModel.

---

## **6\. Build & Run Instructions**

### **6.1 Backend**

**Build Requirement:** protoc (Protocol Buffers Compiler) must be in $PATH, or available via tonic-build.

Bash

cd backend  
\# Compiles .proto via build.rs and starts server  
cargo run

### **6.2 Frontend**

**Build Requirement:** Node.js v18+.

Bash

cd frontend  
\# 1\. Install Deps  
npm install  
\# 2\. Generate TS Client (if proto changed)  
npx buf generate ../proto  
\# 3\. Start Dev Server  
npm run dev

## **7\. Known Limitations (PoC)**

1. **Data Persistence:** Backend uses ephemeral in-memory Vec. Restarting the server resets data.  
2. **Error Handling:** Basic Result types. Production should map Results to specific gRPC Status Codes (e.g., Status::invalid\_argument).  
3. **Security:** No TLS/SSL implemented. Traffic is plaintext HTTP. Production requires a reverse proxy (Nginx/Envoy) or Rust TLS configuration.




INIT



# **Technical Specification: Logistics Analytics Dashboard (PoC)**

Version: 1.0 (PoC)  
Date: November 23, 2025  
Architecture: Hybrid Client/Server Data Processing via gRPC-Web.

---

## **1\. Technology Stack**

### **Backend (Service Layer)**

* **Runtime:** Rust (Edition 2021\)  
* **Framework:** tonic (v0.12) \- Native gRPC implementation over HTTP/2.  
* **Web Interop:** tonic-web (v0.12) \- gRPC-Web translation layer for browser compatibility.  
* **Async Runtime:** tokio (v1.0+) \- Multi-threaded asynchronous execution.  
* **Middleware:** tower-http (v0.5) \- Handles CORS and HTTP layer instrumentation.  
* **Serialization:** prost (v0.13) \- Protocol Buffers implementation for Rust.

### **Frontend (Presentation Layer)**

* **Framework:** Next.js 14+ (React 18\)  
* **Language:** TypeScript 5+  
* **Network Client:** ConnectRPC (@connectrpc/connect) \- Type-safe gRPC client.  
* **Transport:** @connectrpc/connect-web \- Implements the gRPC-Web protocol over HTTP/1.1.  
* **State/UI Logic:** TanStack Table v8 (@tanstack/react-table) \- Headless data grid.

### **Interface Definition**

* **IDL:** Protocol Buffers v3 (proto3)  
* **Transport Protocol:** gRPC-Web (application/grpc-web+proto)

---

## **2\. Directory Structure & Key Files**

Plaintext

/fxgraph
├── /proto  
│   └── dashboard.proto       \# Single Source of Truth (Schema)  
├── /backend  
│   ├── build.rs              \# Pre-build script: Compiles .proto to Rust  
│   ├── Cargo.toml            \# Rust dependencies  
│   └── src/main.rs           \# Server entry point, Mock DB, Business Logic  
└── /frontend  
    ├── buf.gen.yaml          \# Buf configuration for TS generation  
    ├── package.json         \# Dependencies (ConnectRPC, TanStack Table)
    ├── app/page.tsx         \# Main UI combining both tables
    ├── lib/client.ts         \# ConnectRPC Transport singleton  
    ├── components/  
    │   ├── columns.ts       \# TanStack Column Definitions
    │   ├── DashboardTable.tsx \# Server-Side Sort Implementation  
    │   └── RegionTable.tsx    \# Client-Side Sort Implementation  
    └── gen/                  \# Auto-generated TS types (DO NOT EDIT)

---

## **3\. Interface Specification (dashboard.proto)**

The API defines two distinct data access patterns.

Protocol Buffers

syntax \= "proto3";  
package dashboard;

service AnalyticsService {  
  // Pattern A: Server-Side Processing (Pagination/Sorting)  
  rpc GetShipments (ViewRequest) returns (ViewResponse);

  // Pattern B: Client-Side Processing (Snapshot)  
  rpc GetRegionStats (Empty) returns (RegionList);  
}

message ViewRequest {  
  int32 page\_number \= 1;  
  int32 items\_per\_page \= 2;  
  SortOption sort \= 3;        // Optional: if null, default sort applies  
  repeated FilterCriteria filters \= 4;  
}

message SortOption {  
  string column\_id \= 1;  
  bool is\_ascending \= 2;  
}

// ... (See source for full message definitions)

---

## **4\. Backend Implementation Details**

### **4.1 Server Configuration (main.rs)**

The server enables the GrpcWebLayer to translate browser HTTP/1.1 requests into HTTP/2 gRPC calls.

Crucial Layer Ordering:  
Middleware layers are wrapped outside-in. CORS must be the outermost layer to handle preflight OPTIONS requests before they reach the gRPC logic.

Rust

Server::builder()  
    .accept\_http1(true)                   // Required for gRPC-Web  
    .layer(cors\_layer)                    // 1\. Handle CORS  
    .layer(GrpcWebLayer::new())           // 2\. Translate gRPC-Web \-\> gRPC  
    .add\_service(AnalyticsServiceServer::new(service))  
    .serve(addr)  
    .await?;

### **4.2 CORS Configuration**

Browsers enforce strict CORS for gRPC-Web. The tower-http configuration must expose specific gRPC headers, otherwise the client will throw generic protocol errors even on successful requests.

* **Allowed Origin:** MirrorRequest (or explicit localhost:3000)  
* **Exposed Headers:** grpc-status, grpc-message, grpc-status-details-bin

### **4.3 Data Logic**

* **Mock Data:** Vec\<MockShipment\> generated via rand at startup (10k rows).  
* **get\_shipments (Server Sort):**  
  1. Clones the in-memory vector.  
  2. Performs sort\_by based on request.sort.column\_id.  
  3. Applies skip((page-1) \* limit).take(limit).  
  4. Returns ViewResponse.  
* **get\_region\_stats (Client Sort):**  
  1. Returns a static Vec\<RegionStat\>.  
  2. No sorting/paging logic applied serverside.

---

## **5\. Frontend Implementation Details**

### **5.1 Code Generation**

TypeScript types are generated using buf (or protoc-gen-es).

* **Command:** npx buf generate ../proto  
* **Output:**  
  * \*\_pb.ts: TypeScript interfaces for messages.  
  * \*\_connect.ts: Service definition and method signatures.

### **5.2 ConnectRPC Client (lib/client.ts)**

Instantiates a singleton client using createGrpcWebTransport.

* **BaseUrl:** http://localhost:50051  
* **Transport:** Binary format (default).

### **5.3 TanStack Table Configuration**

Scenario A: Server-Side (Big Data)  
File: DashboardTable.tsx

* **Mode:** Manual.  
* **Configuration:**  
  TypeScript  
  manualPagination: true,  
  manualSorting: true,

* **Effect:** Sort/Page state changes trigger useEffect \-\> analyticsClient.getShipments. Table does *not* sort the data array locally.

Scenario B: Client-Side (Small Data)  
File: RegionTable.tsx

* **Mode:** Automatic.  
* **Configuration:**  
  TypeScript  
  getCoreRowModel: getCoreRowModel(),  
  getSortedRowModel: getSortedRowModel(), // Enables local sorting  
  // manualSorting is OMITTED/FALSE

* **Effect:** useEffect runs once on mount. Sort state changes are handled entirely by the JS engine via getSortedRowModel.

---

## **6\. Build & Run Instructions**

### **6.1 Backend**

**Build Requirement:** protoc (Protocol Buffers Compiler) must be in $PATH, or available via tonic-build.

Bash

cd backend  
\# Compiles .proto via build.rs and starts server  
cargo run

### **6.2 Frontend**

**Build Requirement:** Node.js v18+.

Bash

cd frontend  
\# 1\. Install Deps  
npm install  
\# 2\. Generate TS Client (if proto changed)  
npx buf generate ../proto  
\# 3\. Start Dev Server  
npm run dev

## **7\. Known Limitations (PoC)**

1. **Data Persistence:** Backend uses ephemeral in-memory Vec. Restarting the server resets data.  
2. **Error Handling:** Basic Result types. Production should map Results to specific gRPC Status Codes (e.g., Status::invalid\_argument).  
3. **Security:** No TLS/SSL implemented. Traffic is plaintext HTTP. Production requires a reverse proxy (Nginx/Envoy) or Rust TLS configuration.
