



Main proposal:
-
    (Rust server) - gRPC - JS with Rust server - rkyv -
    Rust/WASM


Proposed Rust + WASM approach:
    - Rust Server: Dumps memory directly to a binary stream
      (Serialization)
    - Network: Transmits raw bytes.
    - WASM Client: Loads bytes directly back into Rust Structs
      (Deserialization).
        - no deserialization if using rkyv, zero-copy

(( alternate to the current... )) ((  - Rust Server: Converts
Rust Struct -> Protobuf Binary.)) ((  - Network: Transmits
Protobuf (often wrapped in gRPC-Web text/base64 framing). ))
((
- JS Client: Parses Protobuf $\rightarrow$ JavaScript Objects.
  ))

- use/control both ends with the same language (Rust)...
- share the exact struct definitions via a common library
  (crate)...
- allows us to use **serialization** formats that are strictly
  coupled to code
    - (extremely fast, faster than gRPC)



Why is this faser han gRPC?
-
    - gRPC (Protobuf) is designed to be language agnostic
    - handles fields that are option, out of order, or
      versioned
    - whereas, rust-native formats dont always need those
      checks

Zero-Copy possible using rkyv (pronounced "archive")
-
    - Traditional (Serde/Bincode): The server turns the object
      into bytes. The client reads bytes and allocates new
      memory to recreate the object.
    - Zero-Copy (rkyv): The server aligns the data in memory
      and sends it. The client receives the buffer and casts a
      pointer to it.
    - There is effectively no deserialization step. The client
      can use the data strictly as it arrived over the network
      without the CPU doing any work to "parse" it.


Current Architecture (gRPC + JS):
-
    - Protocol - gRPC-Web
    - Serialization - Protocol Buffers - compact but requires
      parsing
    - Client - JS / TypeScript
    - Type Safety - enforced via .proto files generation
    - Bottleneck - JS parsing overhead & garbach collection

Proposed Architecture (Rust/WASM):
-
    - Protocol - WebSocket or HTTP/2 Binary Strams
    - Serialization - Bincode (small, fast) or rkyv (instant)
    - Client - Rust compiled to .wasm
    - Type Safety - enforced by Rust compiler (shared crates
      [/struct/module?]
    - Bottleneck - WASM/JS "bridge" overhead


Proposed project workspace structure:
-
// ./shared/src/lib.rs ... lib crate #[derive(Serialize,
Deserialize)] // or #[derive(Archive)] for rkyv pub struct
GameState { pub score: u64, pub positions: Vec<(f32, f32)>,}

// ./server/ ... bin crate
    - imports 'shared' (ie ./shared/src/lib.rs?)
    - uses 'bincode::serialize' to send data over a WebSocket
      or HTTP response // ./client/ ... WASM crate
    - imports 'shared'
    - uses 'fetch' or 'ws' to get the bytes, then
    - 'bincode::deserialize'

Solutions to the "bridge" tax of moving data from Rust/WASM to
JS
-
    - Canvas/WebGL ('wgpu' or 'web-sys')
        - preferred method where possible
        - draw he data to a <canvas> so WASM never needs to
          talk to JS
    - Signals
        - only pass specific strungs/numbers needed for the UI
          to JS, rather than the entire object
    - Frameworks (Leptos, Dioxus, Yew)
        - rust frameworks that handle the WASM/DOM interaction
          efficiently
        - may try these down the road, but for now do not use,
          as we are trying to keep this build minimal and
          avoid additional frameworks


Summary/notes:
-
    - Rust (server) - Rust/WASM (client)
    - transfer w/
        - zero-copy
            - rkyv
            - #[derive(Archive)]
        - serializeserialize/deserialize w/ share structs
          otherwise
            - bincode::deserialize
            - #[derive(Serialize, Deserialize)]
    - render WASM w/
        - Canvas/WebGL, wgpu or web-sys, draw data to a
          <canvas>
        - DO NOT USE: possible future frameworks: Leptos,
          Dioxus, Yew ./ ├── shared │   ├── Cargo.toml │   └──
          src │       └── lib.rs // shared structures ├──
          server │   ├── Cargo.toml │   └── src │   └── ___.rs
          ├── client │   ├── Cargo.toml │   └── src │  
          └── ___.rs └── ...





Apache Flight
-
    - may be overkill and actually slower
    - additional  archiectural complexity in the browser
    - significant hurdles when running inside a web browser
    - built on gRPC/HTTP2 - browsers can not make direct gPRC
      calls
    - server must first translate gPRC -> gPRC-Web (usually w/
      proxy)
    - serialization translation, network hop, infrastructure
      complexity vs Custom Rust Binary:
-
    - over standard **WebSockets**
    - direct TCP-like connection
    - send raw binary bytes from server
    - browser receives raw binary bytes **(ArrayBuffer)**
    - no proxies, headers, translation

!! important trap !!
-
    - official Rust 'arrow-flight' crate relies on 'tonic' (a
      gRPC library) and tokio, which generatlly **do not
      compile to 'wasm32-unknown-unknown** because they expect
      system TCP sockes, which the browser sandbox forbids
    - workaround:
    - ... to use Flight in WASM, often have to use JavaScript
      Filght client and "bind" it to the Rus code, **which
      defeats the purpose of writing our logic in Rust**
    - why our solution works better:
    - ... rkyv and bincode are no_std compatible and work
      flawlessly in WASM out of the box

(sweet spot) Arrow ICP over WebSockets
-
    - if using Polars, use the Arrow memory format, but avoid
      the "Flight" protocol wrapper

Best Hybrid Approach: Polars + Arrow IPC:
-
    - for straming raw bytes
    - Zero-Copy benefits of Arrow...  ... w/o weight of
      gRPC/Fight
    - Server: Use Polars/Arrow to create a DataFrame
    - Serialize: Dump the dataframe to Arrow IPC Streaming
      Format (raw bytes).
    - Transport: Send those bytes over a WebSocket.
    - Client (WASM): Read the bytes directly into a Polars
      DataFrame (using polars_core or arrow-wasm).


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
        - Arrow IPC over WebSocket: Polars/DataFrames in
          Browser
        - Custom Binary (rkyv): Game state / Simple structs


When sending tabular data (Polars DataFrames), do not use
Flight. Instead, stream Arrow IPC bytes over a WebSocket. This
preserves the efficient columnar layout (which Polars needs)
but avoids the gRPC/Envoy complexity that Flight requires in
the browser.

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
        - sort, filter, etc arrow kernels are not bad but
          still must be paired with additional code logic
    - Use Polars when...
        - doing more serious data access/manipulation on
          browser
        - group by is untenable in Arrow
    - Arrow is significantly lighter than Polars
    - Arrow is not as capable in terms of high-level features
    - Arrow can build/do anything, but must assemble every
      piece yourself
        - likely leads to...
        - micro optimizations in rare cases
        - more bugs and code to do the same things
        - irrelevant amount of "weight" saved over Polars
    - arrow-rs is modular, can compile only the specific
      features needed
    - Arrow likely <1MB compressed
    - Polars likely 3-5MB compressed (heavy for a web browser
      )
    - "Raw Arrow" is a low-level memory format, not a data
      analysis library.

arrow-rs v Polars: code comparison:
-
    Filter: (Polars)
    - df.filter(col("age").gt(30)) (arrow)
    - arrow::compute::filter kernel
    - compute a boolean mask array [true, false, ...], then
      use the filter kernel to create a new array using that
      mask
    - ...
    - ...  Sort: (Polars)
    - df.sort("name") (arrow)
    - arrow::compute::sort/sort_to_indices kernel
    - use the 'sort_to_indices' kernel to get a list of index
      positions, then use the 'take' kernel to reorder every
      column manually based on those indices.
    - ...
    - ...  Group by (Polars)
    - df.group_by("city").agg(sum("salary")) (Arrow)
    - **Extremely difficult**
    - manually hash the "city" column, map indices to buckets,
      and iterate through the "salary" column to sum values
      into those buckets


Apache DataFusion (avoid for this project)
-
    - SQL engine built on Arrow
    - similar to Polars
    - much heavier, not optimized for browser like Polars is
    - may use this later on for backend table database-esk
      actionsa... may revisit at that point



Arrow Purist: Streaming Raw Arrow / Arrow IPC Ingestion:
-
    - open a WebSocket and stream raw bytes
    - do not need 100% of data to arrive before we can access
      it
    - 1) Arrow Schema arrives (the first 1KB)
    -    Polars (which uses Arrow under the hood) knows the
         column names
    - 2) First N rows arrive (the next nKb)
    -    Polars can render these rows on the screen
    - ...
    - (server): serializes a RecordBatch into Arrow IPC
      Streaming format basically a raw memory dump
    - (network): sends bytes
    - (client): receives the Uint8Array. Pass this array into
      arrow::ipc::reader::StreamReader.

Arrow Purist: Storage
-
    - In Rust, our table would look like this...  struct
      UserTable { ids: Int32Array,      // [1, 2, 3, ...]
      names: StringArray,   // ["Alice", "Bob", "Charlie",
      ...] scores: Float32Array, // [95.5, 88.0, 42.0, ...]}


Arrow Purist: code for manual manipulation
-
    - build exactly the features we need using the low level
      kernels: 'filter', 'take', 'sort_to_indices'
    - 'arrow::compute' module as the toolkit for SQL-esk
      actions
    - SQL: SELECT * WHERE scores > 90
    - arrow-rs: 
        - use 'gt_scalar' kernel on 'scores' array to return a
          bool bitmask [False, False, True, ...]
        - use 'filter' kernel on **every column** w/ bitmask
          filter(ids, mask) -> [1] filter(names, mask) ->
          ["Alice"] filter(scores, mask) -> [95.5]

Arrow Purist: render to UI
-
    - Strategy A: (preferred) only send the rows currently
      visible on the screen to JS (negligible cost)
        - user scrolls to rows 100-120
        - rust slices the arrays at 100-120
        - Rust converts just those rows to a JS Array/Object
        - JS renders 20 'div's
        - negligible cost
    - Strategy B: (advanced) shared buffer, pass a pointer to
      the Arrow memory directly to JS
        - JS uses the apache-arrow library to wrap a view
          arount the WASM memory
        - JS reads the data without Rust doing any work


Arrow Purist: manual_arrowrs_examples.rs
-
    - Why this demonstrates the "Lightweight" Advantage
    - Dependencies: This script only requires arrow. It does
      not require polars, polars-core, polars-lazy, sqlparser,
      or serde. The resulting WASM binary will be
      significantly smaller.
    - 
    - No "Black Box":
    - In the group_by_example, you chose to use a HashMap. You
      could have chosen to sort and slice if memory was tight.
      In Polars, the engine chooses for you.
    - In unique_operations_example, you can see how we built a
      HashMap<&str, Vec<usize>>.  This &str references the
      actual bytes inside the Arrow array. We didn't allocate
      new strings for the keys. This level of memory control
      is difficult in high-level DataFrame libraries.
    - 
    - Direct Access:
    - The loop for i in 0..batch.num_rows() with
      sales.value(i) compiles down to a very tight CPU loop,
      similar to C++. There is no "Expression Evaluation"
      overhead checking data types at runtime for every row.


Arrow Purist: sending large compressed v small pieces over network
-
    - both capabilities built directly into the IPC
    - all in the 'IpcWriteOptions' library

Arrow Purist: compressed data transfers
-
    - Format: 'Arrow IPC File Format' or 'Stream Format' with
      ZSTD
    - use ZSTD, natively supported by Arrow, high compression
      ratio

Arrow Purist: server side - compressed data transfer - Rust code
-


use arrow::ipc::writer::{FileWriter, IpcWriteOptions};
use arrow::ipc::compression::CompressionType;

fn send_large_compressed_data(batch: &RecordBatch) -> Vec<u8> {
    let mut buffer = Vec::new();

    // 1. Configure Compression (ZSTD is usually best for size)
    let options = IpcWriteOptions::try_new()
        .with_compression(Some(CompressionType::ZSTD))
        .unwrap();

    // 2. Create the Writer with these options
    let mut writer = FileWriter::try_new_with_options(
        &mut buffer, 
        batch.schema(), 
        options
    ).unwrap();

    // 3. Write and Finish
    writer.write(batch).unwrap();
    writer.finish().unwrap();

    buffer // This Vec<u8> is now highly compressed
}

// Client Cargo.toml
[dependencies]
arrow = { version = "53.0", features = ["ipc_compression"] }



Arrow Purist: client side - compressed data transfer - Rust code
-
    - reader auto-detects compression as long as the feature
      is enabled in Cargo.toml, dont need to change reading
      logic // client Cargo.toml [dependencies] arrow = {
          version = "53.0", features = ["ipc_compression"] }




Arrow Purist: uncompressed real-time streaming
-
    format: 'Arrow IPC Streaming Format'

// SCENARIO: Low Latency (Arrow IPC)
// Best for: WebSocket updates, small chunks, instant visualization
pub fn get_realtime_stream_chunk() -> Vec<u8> {
    let mut df = create_dummy_df();
    let mut buffer = Cursor::new(Vec::new());

    // IpcStreamWriter corresponds to "Stream Format" (not File format)
    // It creates the raw Arrow bytes exactly as they exist in memory
    IpcStreamWriter::new(&mut buffer)
        .finish(&mut df)
        .unwrap();

    println!("IPC Stream Size: {} bytes", buffer.get_ref().len());
    buffer.into_inner()
}


// ALTERNATE IMPLEMENTATION of Low Latency (Arrow IPC)
use polars::prelude::*;
use std::io::Cursor;

pub fn stream_update(df: &mut DataFrame) -> Vec<u8> {
    let mut buffer = Cursor::new(Vec::new());
    
    // IpcWriter writes the "Stream" format (no file footer, optimized for streams)
    IpcStreamWriter::new(&mut buffer)
        .finish(df)
        .unwrap();

    buffer.into_inner()
}



Arrow Purist: server side - compressed data transfer - Rust code
-
    - keep the writer open and 'flush' immediately after every
      write

use arrow::ipc::writer::{StreamWriter, IpcWriteOptions};

// Imagine this function runs inside a WebSocket loop
fn stream_realtime_update(batch: &RecordBatch) -> Vec<u8> {
    let mut buffer = Vec::new();

    // 1. No Compression Options (Default is None)
    let options = IpcWriteOptions::default();

    // 2. Create Stream Writer
    // Note: We use StreamWriter, not FileWriter. 
    // StreamWriter is optimized for sequential processing.
    let mut writer = StreamWriter::try_new_with_options(
        &mut buffer, 
        batch.schema(), 
        options
    ).unwrap();

    // 3. Write the batch
    writer.write(batch).unwrap();
    
    // 4. IMPORTANT: Finish/Flush explicitly if sending discrete messages
    writer.finish().unwrap(); 

    buffer // This is raw, uncompressed Arrow bytes ready for the wire
}


Arrow Purist: Note on Web Socket Streaming Architecture
-
    - 2 choices...
    - Discrete Messages: You create a new StreamWriter for
      every WebSocket message.  Each message contains the
      Schema + 1 RecordBatch. This is easiest to implement but
      adds a tiny overhead (sending the schema every time).
    - 
    - Continuous Stream: You send the Schema once when the
      WebSocket connects.  Then, for every update, you only
      send the bytes for the RecordBatch. On the client, you
      feed these chunks into a continuous StreamDecoder.


Arrow Purist: Stream v Compression: when to use:
-
    - use compressed data transfer approach for initial login
      etc
    - use streaming for real-time updates


Advantages of Server: Arrow/Polars -> Arrow IPC -> Client: Arrow/Polars
-
    - WASM Optimization: The Polars WASM library (polars on
      npm) is optimized to accept Uint8Array buffers
      containing Arrow IPC data.
    - Zero-Conversion: Polars DataFrame objects are backed by
      arrow::RecordBatch. Writing to IPC is just a memcpy of
      the underlying buffers.
    - Shared Semantics: Polars-to-Polars via Arrow preserves
      all data types perfectly (including Nulls, Categoricals,
      and Timezones).

Use Parquet for compression when using Polars/Arrow
-
    - Parquet is the native "disk" format in the Arrow/Polars
      ecosystem
    - thus, has much better compression ratios than
      ZSTD-compressed IPC
    - because, it uses column-specific encodings
    - like, Run-Length Encoding and Delta Encoding before
      applying the compression algoritm

Process for Parquet as the compressed data transfer
-
    - server writes data to parquet with compression directly
      into the buffer
    - client uses pl.readParquet(buffer)



(revisit note section)
-
    - conflicting information: which is better for sending compressed data over the network?
    - A) arrow ZSTD compression + Arrow IPC
    - B) buffer = parquet_table.to_parquet(compression = ZSTD)

start of A (revisit note section)
-
use arrow::ipc::writer::{FileWriter, IpcWriteOptions};
use arrow::ipc::compression::CompressionType;

fn send_large_compressed_data(batch: &RecordBatch) -> Vec<u8> {
    let mut buffer = Vec::new();

    // 1. Configure Compression (ZSTD is usually best for size)
    let options = IpcWriteOptions::try_new()
        .with_compression(Some(CompressionType::ZSTD))
        .unwrap();

    // 2. Create the Writer with these options
    let mut writer = FileWriter::try_new_with_options(
        &mut buffer, 
        batch.schema(), 
        options
    ).unwrap();

    // 3. Write and Finish
    writer.write(batch).unwrap();
    writer.finish().unwrap();

    buffer // This Vec<u8> is now highly compressed
}

-------------------------
end of A (revisit note section)
-------------------------

    
-------------------------
start of B (revisit note section)
-------------------------
use polars::prelude::*;
use std::io::Cursor;

pub fn send_large_history(df: &mut DataFrame) -> Vec<u8> {
    let mut buffer = Cursor::new(Vec::new());
    
    ParquetWriter::new(&mut buffer)
        .with_compression(ParquetCompression::Zstd(None))
        .finish(df)
        .unwrap();

    buffer.into_inner()
}

-------------------------
end of B (revisit note section)
-------------------------




'Inserting' additional data into client table
-
    - Polars DataFrame/Arrow: immutable
    - can only 'append' by linking existing table to the start
      of a new table segment
    - columns are 'ChunkedArray', not necessarily contiguous
    - column can effectively be a list of pointers to several
      distinct memory buffers
    - to insert new data to an existing table...
        Vertical Stack ('vstack')
    - causes chunking, causes performance cost
        - querying fragmented data means CPU can not use SIMD
          (vectorization) efficiently across boundaries
    - fix: periodically trigger a 'rechunk' operation
    - 'rechunk': allocate and copy to a new, large, contiguous block
    - trigger rechunk:
        a) after every ~100 inserts
        b) before large operations like sort/groupby

Arrow Builders: Potential Hybrid Efficient Approach to aggregating data cleanly on the client
-
    - Standard approach w/o builders:
    1) Receive: Client receives bytes from Server.
    2) Deserialize: Bytes become a temporary SmallDataFrame.
    3) Validate: Ensure SmallDataFrame schema matches MainDataFrame schema
    3) Stack: MainDataFrame.vstack(SmallDataFrame). (Instant, Zero-Copy).
    4) Monitor: check the number of chunks, trigger MainDataFrame.rechunk()
    - .
    - ***Arrow Builders approach***: Double Buffering / Batching via Builders
        - avoid 'vstack' fragmenting many small incoming rows
        - use this with small, repeated, real-time updates to avoid
          fragmentation
        - set fragmentation chunk size and buffer latency
        - hybrid memory model:
            - create a hot buffer (standard) for incoming data
            - systematically flush to 'cold' well formatted
              data held by client
        - detriment: otherwise, row-by-row incoming data would
          be immediatly visible to the UI, rather than being
          stuck in the Builder buffer until it reaches 500ms
          or 1000 rows
    1) Allocate a Builder ('Int32Builder' or 'StringBuilder')
        - call '.reserve(1000)'
        (allocates contiguous block of memory)
        - data streams from server into this builder on the
          client
        - extremely fast because memory is already allocated
        - cpu writes to next memory address, increments the
          address index
        - cpu cache local to L1/L2 because it is constantly
          written to and contiguous
    2) 'freeze' the buffer
        - once the counter hits the overflow point (set to
          1000 rows earlier) or 500ms, perform a Finish operation
        - call 'builder.finish()'
        - buffer *gives up ownership* of the memory block
        - flips permission on the memory block from a
          **mutable** list to an **immutable** Arrow Array
        - zero-copy
    3) stack pointer swapping
        - continue with standard approach using these more
          appropriately:
        1) 'vstack' (append) the 'RecordBatch' / Polars
        DataFrame
        2) Polars adds the pointer to the new block to the
        list of chunks
        *3)* allocate a new Builder
        4) repeat




Server API Endpoint: how to receive and use requests from the client
-
    alpha) client sends a struct holding values in fields to
    the WebSocket rust server
    1) rust server sits an an infinite loop waiting for
    network packets (WebSocket messages)
    2) enter the action of the loop when a message is received
    3) use the variables/values inside the message/struct, pass to
    polars fxn etc
    ie
    // server receives 'request' instance
    df.filter(
        col(**&request.col**).eq(
        lit(**&request.val**)))
    .unwrap()
    .head(
        Some(**request.limit**));
    // end of example
    4) return appropriate response

    OR
    - let client send SQL context to the server
    - use polars_sql::SQLContext
    - SQL engine parses the string at runtime and executes the
      correct Polars functions
    

# Comparison: DataFusion vs. Polars DataFrame  
*(Consolidated Notes)*

---

## 1) Strengths / Capabilities of DataFusion over Polars DataFrame
- Cooperative, async-aware execution engine
    - *Polars DataFrame does not yield during long operations.*
- Handles workloads that must pause/resume on I/O boundaries (disk, remote
  sources)
    - *Polars DataFrame does not incorporate async waiting or latency-hiding
      mechanisms.*
- Fair scheduling across many heterogeneous queries
    - *Polars DataFrame executes run-to-completion and does not interleave
      workloads.*
- Supports distributed / multi-node execution models
    - *Polars DataFrame does not coordinate distributed or external fetch
      operations.*
- Streaming and incremental batch processing
    - *Polars DataFrame does not natively support
      streaming/partial-materialization workflows.*

---

## 2) Strengths / Capabilities of Polars DataFrame over DataFusion
    - Maximum single-node, in-memory performance via SIMD-optimized synchronous
      execution
        - *DataFusion introduces async scheduling overhead.*
    - Tight run-to-completion loops for heavy numeric workloads
        - *DataFusion yields frequently, adding coordination costs.*
    - Predictable throughput when the entire dataset is in RAM
        - *DataFusion is built around the possibility of I/O waits.*
    - Ideal when paired with a bounded Rayon compute pool
        - *DataFusion mixes compute with async scheduling logic.*
    - Best choice for CPU-bound workloads that never wait on external resources
        - *DataFusion optimizes for environments where waiting is expected.*

---

## 3) Scenario Where DataFusion Is the Better Choice Use DataFusion when:
    - The engine must request data from outside sources (disk, object stores,
      remote nodes).  
    - Execution involves pauses, remote fetches, or multi-stage coordination.  
    - You need workload fairness, preventing long-running queries from
      monopolizing resources.  
    - You operate in or plan for a distributed or multi-node architecture.  
    - Query execution includes natural waiting, allowing async yielding to hide
      latency.

---

## 4) Why Polars Is Not Ideal in This Scenario
    - Polars assumes data is already available and gains no benefit from async
      I/O handling.  
    - Synchronous run-to-completion can block compute threads during long
      waits.  
    - Polars cannot pause/resume during external fetches.  
    - It cannot leverage distributed execution or remote orchestration.

---

## 5) Scenario Where Polars DataFrame Is the Better Choice Use Polars when:
    - All data is in memory on a single node.  
    - Workloads are purely CPU-bound, with no external dependencies.  
    - Many clients issue requests, but each query is self-contained.  
    - A bounded Rayon thread pool handles compute, while Tokio handles
      networking.  
    - The objective is maximum raw throughput per core, not distributed
      scheduling fairness.

---

## 6) Why DataFusion Is Not Advantageous in This Scenario
    - Its async engine adds a coordination tax without providing any benefit
    - Yielding slows down heavy computations that would run best synchronously.  
    - Scheduling fairness is unnecessary when all queries operate solely
      in-memory.  
    - Distributed and I/O-aware capabilities are overkill, making DataFusion
      slower than Polars’ optimized single-node model.

---



-------------------------
Apache DataFusion v Polars DataFrame
-------------------------
    - Server: use Polars DataFrame + rayon
    - DataFusion advantage:...
    - for when sending requests for data from other nodes / over the network
        - uses tokio for async processing to do other things while waiting for
          responses
        - assumes data size from source is unknown
    - DataFrame: advantage...
        - use when data exists on just that node
        - focuses all resources on task at hand
        - optimized things like compression by assuming a known size of data
        - wrap in tokio / rayon for when RECEIVING many requests


-------------------------
rayon v tokio (brief)
-------------------------
    - rayon:
    ...
    - parallel execution
    - for CPU bound tasks
    - execute an iterator in parallel
    - includes balancing, if one finishes a task first, etc
    ...
    - tokio:
    ...
    - asynchronous execution
    - for IO-bound / "waiting" tasks
    - if waiting for something to finish, pause, work on something else, come back
    - ie dealing with waiting for network responses


-------------------------
Detour... Apache DataFusion
-------------------------
    - Appache Arrow: data structure
    - Apache DataFusion: query engine designed for database developers
    - Apache Spark: multi-server distributed big-data processing (PBytes)
    - ... Apache DataFusion
    - built to be a "Lego set" for creating database systems (like Spark or
      Snowflake)
    - used to build data tools
    - built around Tokio for network/IO adjacent queries, thus async operations
    - DataFusion uses/provides lazy evaluation (so does Polars DataFrame)
        - fxn1:
            - input: user defined query
            - action/output: a calculated/optimized plan for executing the query
    - opposed to Polars DataFrame:
        - single machine RAM
        - multi-threaded
        *not sure if this is actually a good comparison...*
        * both use lazy evaluation (create query plan before executing)*
        * both are probably multi-threaded and using RAM *


-------------------------
Choose Polars DataFrame if: TL;RD: faster/est
-------------------------
    - coming from Pandas and want that experience in Rust
    - writing a CLI tool or a data processing script
    - do not want to deal with async/await, Tokio, or Futures complexity
    - need the absolute fastest performance on a single machine
    - Polars is currently faster than Apache DataFrame in most benchmarks
    - ...
    - in single-node, in-memory processing tasks
    - tasks like:
        - reading a 50GB CSV file
        - grouping by a column
        - calculating the mean
    - benchmarks like TPC-H standard, H2O.ai Database Benchmarks
    - Polars consistently outperforms almost every other tool, including:
        - DataFusion
        - Spark
        - Pandas
    - DataFusion is built on Tokio, which adds small amount of overhead
    - Polars uses a custom synchronous thread pool - grabs all CPU cores and
      hammers at 100% until the math is done, avoids context-switching overhead
      of async runtimes used to manage network latency and keeping a server
      responsive
    - DataFusion uses arrow-rs compute kernels
    - Polars uses custom, often unsafe, compute kernels over the same arrow format
    - ...
    - Polars' lazy evaluation optimizer is aggressively tuned for local execution:
        - incredibly good at:
            - predicate pushdown: filtering data while reading the file so you
              don't load unnecessary rows into RAM
            - reordering joins to minimize the size of intermediate tables


-------------------------
Choose Apache DataFusion if:
-------------------------
    - REST API, Tokio, Async I/O
    - building a server (e.g., a REST API) that serves data and needs to handle
      many concurrent requests (Async I/O is crucial here).
    - excellent SQL support out of the box
    - need to query data sitting in Cloud Storage (S3, GCS) directly without
      downloading it first (DataFusion's ObjectStore integration is superior).



Pure Rust -> UI: egui
-
    - ie https://www.egui.rs/#clock
    - https://docs.rs/egui_extras/latest/egui_extras/struct.TableBuilder.html    
    - ...
    Pros:
    - high performance dashboards, trading terminals
    - workflow for when a user sorts a table that is displayed:
        1) Loop: The generic update() loop runs 60 times a second
        2) Immediate Mode: In every frame, you say ui.label(row_value)
        3) Interaction: You write if ui.button("Sort").clicked() { df.sort(...) }
    - can display intense ammounts of data cleanly
        ie https://www.egui.rs/#clock
    - draws everything (text, borders, buttons) onto a WebGL Canvas using GPU
      acceleration, bypasses the browser DOM entirely.
    - does not use HTML <table> or <div>
    - insanely fast, can THEORETICALLY render 10,000 rows at 60FPS because it
      is just pixels
      on a GPU texture
    - no "Javascript/DOM" overhead.

    Cons:
    - rigid
    - abilities would be lost on a table because creating 50,000 HTML <tr>
      elements will freeze the browser (because the browser's rendering engine
      is the bottleneck, not Rust). If your table is larger than ~500 rows, you
      must implement Virtualization (or "Windowing") in Rust


    Examples:
    - https://www.egui.rs/#clock
    - https://emilk.github.io/egui_plot/
    - https://github.com/emilk/egui_plot
    - https://github.com/rerun-io/rerun

Pure Rust -> UI: leptos (+polars)
-
    Pros: looks like a normal web app
    Cons: not quite as high performance as egpu
    Action: treat the Polars DataFrame as a Signal that triggers a Rust fxn
    that reruns the Polars query then updates the HTML

Pure Rust -> UI: virtualization
-
    - Calculate scroll position.
    - Use Polars df.slice(offset, 20) to get only the 20 visible rows.
    - Render only those 20 rows to HTML.


# Island Architecture / Micro-Frontend
-
    - integrate rust-native UI elements alongside traditional JS
      elements
    - rust/WASM ui compenents are embeded inside a specific <div>


# Island Architecture # example
-
    - display rust visual (ie table) alongside/within a JS focused page
    - treat the Rust/WASM component exactly like a YouTube video embed or a
      Google Maps widget...
        - lives inside a specific <div> on your page
        - manages its own internal pixels
        - coexists happily with the rest of the site


# Island Architecture: HTML Mount Point
-------------------------
    - the existing HTML (or React/Vue template)
    - designate a container where the Rust table will live
    - index.html

// index.html

<body>
  <!-- Your existing JavaScript Navbar -->
  <nav id="js-navbar">...</nav>

  <!-- Your existing JS Content -->
  <div class="sidebar">...</div>

  <!-- THE RUST ISLAND -->
  <!-- The Rust WASM will hunt for this ID and take it over -->
  <div id="rust-table-root" style="height: 500px; width: 100%;"></div>

  <!-- Your existing JS Footer -->
  <footer>...</footer>

  <!-- Load the WASM glue code -->
  <script type="module">
      import init, { mount_table } from './pkg/my_rust_app.js';
      
      async function run() {
          await init(); // Initialize WASM memory
          mount_table("rust-table-root"); // Tell Rust where to live
      }
      run();
  </script>
</body>



-------------------------
Island Architecture: Rust, taking ownership
-------------------------
// mount to the specific element id from the HTML setup

use leptos::*;
use wasm_bindgen::prelude::*;

#[component]
fn DataViewer() -> impl IntoView {
    view! {
        <table class="my-cool-rust-table">
            // Polars rendering logic here...
        </table>
    }
}

// This function is exported to JavaScript
#[wasm_bindgen]
pub fn mount_table(element_id: &str) {
    // Leptos helper to mount to a specific Div ID
    mount_to(
        document().get_element_by_id(element_id).unwrap(),
        || view! { <DataViewer /> }
    )
}



-------------------------
Island Architecture: js->rs Input Bridge
-------------------------
    - You likely want the JS side to control the Rust side ((e.g., a JS
dropdown filters the Rust table) or vice versa
    - expose public functions in Rust that manipulate a global or static signal

-------------------------
Island Architecture: js->rs Input Bridge (Rust)
-------------------------

// Create a global signal for the filter
static FILTER_SIGNAL: RwLock<Option<String>> = RwLock::new(None);

#[wasm_bindgen]
pub fn apply_js_filter(region_name: String) {
    // When JS calls this, update the signal
    // The Leptos UI will automatically re-render the table
    let mut write = FILTER_SIGNAL.write().unwrap();
    *write = Some(region_name);
}


-------------------------
Island Architecture: js->rs Input Bridge (JS)
-------------------------

import { apply_js_filter } from './pkg/my_rust_app.js';

// Your existing JS button
document.getElementById("filter-btn").onclick = () => {
    apply_js_filter("US-East"); // Instantly updates the Rust UI
};


-------------------------
Island Architecture: Rust -> JS Output Bridge
-------------------------
// If a user clicks a row in the Rust table, you might want your
React/JS app to show a modal. Rust can dispatch standard browser
events that JS listens for.
// Rust

-------------------------
Island Architecture: Rust -> JS Output (Rust)
-------------------------

let on_row_click = move |row_id| {
    // Create a native browser CustomEvent
    let event = web_sys::CustomEvent::new("rust-row-selected").unwrap();
    // Attach data
    // Dispatch to the window
    window().dispatch_event(&event).unwrap();
};


-------------------------
Island Architecture: Rust -> JS Output (js)
-------------------------

window.addEventListener("rust-row-selected", (e) => {
    console.log("Rust told me the user clicked a row!");
    myReactApp.openModal();
});
};


-------------------------
Island Architecture: Summary
-------------------------
    - React/Vue sees: A black box <div>. It doesn't care what's
      inside.
    - The Browser sees: Just another part of the DOM tree. The HTML
      generated by Rust is indistinguishable from HTML generated by
      React.
    - The User sees: A single cohesive page, where one specific table
      happens to sort 100x faster than the rest of the site.





























    - handle massive datasets without overwhelming the browser
    - while maintaining instant interactivity for smaller,
      aggregated datasets

    - "Hybrid Data Strategy"... seamlessly switches between:
      **Client-Side Processing** (for summary data)
      **Server-Side Processing** (for massive granular data)
      within the same user interface.


## **2\. Architecture Overview**

### **The Tech Stack**

* **Backend:** **Rust** (High-performance systems language)
  * *Framework:* Tonic (gRPC implementation) \+ Tonic-Web
    (Browser compatibility).
  * *Role:* Data generation, heavy sorting, filtering, and
    aggregation.
* **Protocol:** **gRPC-Web** (Binary over HTTP)
  * *Role:* Strongly typed, bandwidth-efficient communication
    between browser and server.
  * *Schema:* Defined via Protocol Buffers (.proto).
* **Frontend:** **TypeScript** \+ **Next.js** (React)
  * *Client:* ConnectRPC (Modern gRPC client for web).
  * *State/UI:* TanStack Table (Headless UI library for data
    grids).

### **Architecture Diagram**

Code snippet

graph LR subgraph Browser \[Frontend: Next.js\] UI\[User
Interface\] RegionTable\[Region Table \<br/\> (Client-Side
Sort)\] ShipmentTable\[Shipment Table \<br/\> (Server-Side
Sort)\] Connect\[ConnectRPC Client\] end

    subgraph Network \[gRPC-Web\] Proto\[Binary Stream\] end

    subgraph Server \[Backend: Rust\] Tonic\[Tonic gRPC
    Server\] MockDB\[Mock Database \<br/\> (10k Rows RAM)\]
    end

    UI \--\> RegionTable UI \--\> ShipmentTable RegionTable
    \-- "Fetch Once (All Data)" \--\> Connect ShipmentTable
    \-- "Fetch Page/Sort" \--\> Connect Connect \<--\> Proto
    \<--\> Tonic Tonic \<--\> MockDB

---

## **3\. The Hybrid Data Strategy**

This PoC specifically demonstrates two distinct patterns for
handling data, proving that we are not locked into a single
approach.

### **Pattern A: "Big Data" (Shipment Manifest)**

* **Use Case:** Granular records, logs, or datasets \> 10,000
  rows (potentially millions).
* **Mechanism:** The browser acts as a "remote control." It
  holds only the data currently visible (e.g., 50 rows).
* **User Experience:** When the user sorts by "Weight," the
  browser sends a request to the server. The server sorts the
  full dataset and returns only the top 50 results. This
  ensures the browser never crashes from memory overload.

### **Pattern B: "Small Data" (Region Stats)**

* **Use Case:** High-level aggregations, summaries, or
  datasets \< 5,000 rows.
* **Mechanism:** The server calculates the aggregation *once*
  and sends the complete list to the browser.
* **User Experience:** Sorting and filtering are **instant**
  (0ms latency) because they occur in the browser's CPU. No
  network requests are made after the initial load.

---

## **4\. Project Structure**

Plaintext

/fxgraph ├── /proto  │   └── dashboard.proto \# Single Source
of Truth (Schema)  ├── /backend │   ├── build.rs
\# Pre-build script: Compiles .proto to Rust  │   ├──
Cargo.toml            \# Rust dependencies  │ └── src/main.rs
\# Server entry point, Mock DB, Business Logic  └── /frontend
├── buf.gen.yaml          \# Buf configuration for TS
generation  ├── package.json         \# Dependencies
(ConnectRPC, TanStack Table) ├── app/page.tsx         \# Main
UI combining both tables ├── lib/client.ts         \#
ConnectRPC Transport singleton  ├── components/  │   ├──
columns.ts       \# TanStack Column Definitions │ ├──
DashboardTable.tsx \# Server-Side Sort Implementation  │   └──
RegionTable.tsx    \# Client-Side Sort Implementation  └──
gen/ \# Auto-generated TS types (DO NOT EDIT)

---

## **5\. Implementation Details**

### **The Schema (dashboard.proto)**

This file is the single source of truth. If this file changes,
both the Backend and Frontend builds will fail, ensuring type
safety.

Protocol Buffers

service AnalyticsService { // Pattern A: Big Data (Server does
the work) rpc GetShipments (ViewRequest) returns
(ViewResponse);

  // Pattern B: Small Data (Client does the work) rpc
  GetRegionStats (Empty) returns (RegionList); }

### **The Rust Backend (backend/src/main.rs)**

* **Mock Data:** Generates 10,000 random shipments on startup.
* **CORS:** Configured using tower-http to allow the Next.js
  frontend to read gRPC headers (grpc-status).
* **Logic:** Implements a custom sort comparator to handle
  dynamic column sorting (e.g., sorting by Timestamp vs.
  Weight) on the server.

### **The Frontend Client (frontend/lib/client.ts)**

Uses **ConnectRPC** to communicate. This abstracts away the
binary parsing.

TypeScript

// Example usage const response \= await client.getShipments({
    pageNumber: 1, sort: { columnId: "cargoWeightKg",
    isAscending: false } });

---

## **6\. Setup & Run Guide**

### **Prerequisites**

1. **Rust:** Install via rustup.
2. **Node.js:** Install Node v18+.
3. **Buf:** (Optional but recommended) for efficient code
generation, though npm scripts handle it in this PoC.

### **Step 1: Start the Backend**

The backend must be running to serve gRPC requests.

Bash

cd backend cargo run \# Expected Output: Server listening on
0.0.0.0:50051

### **Step 2: Start the Frontend**

Open a new terminal window.

Bash

cd frontend npm install npm run dev \# Expected Output: Ready
in 2.3s. Listening on http://localhost:3000

### **Step 3: Verify**

Navigate to http://localhost:3000.

1. **Test Region Table:** Click the "Revenue" header. It
should sort instantly. Check the Backend terminal—you should
see **no** new logs.
2. **Test Shipment Table:** Click the "Weight" header. It
should take a brief moment. Check the Backend terminal—you
should see a log entry: Request received for page: 1\.

---

## **7\. Future Considerations (Post-PoC)**

* **Database:** Replace the in-memory Vec\<MockShipment\> with
  a connection pool to PostgreSQL (using sqlx) or ClickHouse
  for analytics.
* **Filtering:** The .proto file already supports
  FilterCriteria. We can implement SQL WHERE clause generation
  in Rust to support complex filtering.
* **Streaming:** For data exports (e.g., "Download CSV"), we
  can utilize gRPC Server Streaming to send data in chunks
  without buffering.




TECHNICAL




# **Technical Specification: Logistics Analytics Dashboard
(PoC)**

Version: 1.0 (PoC)  Date: November 23, 2025 Architecture:
Hybrid Client/Server Data Processing via gRPC-Web.

---

## **1\. Technology Stack**

### **Backend (Service Layer)**

* **Runtime:** Rust (Edition 2021\)  
* **Framework:** tonic (v0.12) \- Native gRPC implementation
  over HTTP/2.  
* **Web Interop:** tonic-web (v0.12) \- gRPC-Web translation
  layer for browser compatibility.  
* **Async Runtime:** tokio (v1.0+) \- Multi-threaded
  asynchronous execution.  
* **Middleware:** tower-http (v0.5) \- Handles CORS and HTTP
  layer instrumentation.  
* **Serialization:** prost (v0.13) \- Protocol Buffers
  implementation for Rust.

### **Frontend (Presentation Layer)**

* **Framework:** Next.js 14+ (React 18\)  
* **Language:** TypeScript 5+  
* **Network Client:** ConnectRPC (@connectrpc/connect) \-
  Type-safe gRPC client.  
* **Transport:** @connectrpc/connect-web \- Implements the
  gRPC-Web protocol over HTTP/1.1.  
* **State/UI Logic:** TanStack Table v8
  (@tanstack/react-table) \- Headless data grid.

### **Interface Definition**

* **IDL:** Protocol Buffers v3 (proto3)  
* **Transport Protocol:** gRPC-Web
  (application/grpc-web+proto)

---

## **2\. Directory Structure & Key Files**

Plaintext

/fxgraph ├── /proto  │   └── dashboard.proto \# Single Source
of Truth (Schema)  ├── /backend │   ├── build.rs
\# Pre-build script: Compiles .proto to Rust  │   ├──
Cargo.toml            \# Rust dependencies  │ └── src/main.rs
\# Server entry point, Mock DB, Business Logic  └── /frontend
├── buf.gen.yaml          \# Buf configuration for TS
generation  ├── package.json         \# Dependencies
(ConnectRPC, TanStack Table) ├── app/page.tsx         \# Main
UI combining both tables ├── lib/client.ts         \#
ConnectRPC Transport singleton  ├── components/  │   ├──
columns.ts       \# TanStack Column Definitions │ ├──
DashboardTable.tsx \# Server-Side Sort Implementation  │   └──
RegionTable.tsx    \# Client-Side Sort Implementation  └──
gen/ \# Auto-generated TS types (DO NOT EDIT)

---

## **3\. Interface Specification (dashboard.proto)**

The API defines two distinct data access patterns.

Protocol Buffers

syntax \= "proto3";  package dashboard;

service AnalyticsService {  // Pattern A: Server-Side
Processing (Pagination/Sorting)  rpc GetShipments
(ViewRequest) returns (ViewResponse);

  // Pattern B: Client-Side Processing (Snapshot) rpc
  GetRegionStats (Empty) returns (RegionList);  }

message ViewRequest {  int32 page\_number \= 1; int32
items\_per\_page \= 2;  SortOption sort \= 3;        //
Optional: if null, default sort applies  repeated
FilterCriteria filters \= 4;  }

message SortOption {  string column\_id \= 1; bool
is\_ascending \= 2;  }

// ... (See source for full message definitions)

---

## **4\. Backend Implementation Details**

### **4.1 Server Configuration (main.rs)**

The server enables the GrpcWebLayer to translate browser
HTTP/1.1 requests into HTTP/2 gRPC calls.

Crucial Layer Ordering:  Middleware layers are wrapped
outside-in. CORS must be the outermost layer to handle
preflight OPTIONS requests before they reach the gRPC logic.

Rust

Server::builder()  .accept\_http1(true) // Required for
gRPC-Web  .layer(cors\_layer) // 1\. Handle CORS
.layer(GrpcWebLayer::new()) // 2\. Translate gRPC-Web \-\>
gRPC .add\_service(AnalyticsServiceServer::new(service))
.serve(addr)  .await?;

### **4.2 CORS Configuration**

Browsers enforce strict CORS for gRPC-Web. The tower-http
configuration must expose specific gRPC headers, otherwise the
client will throw generic protocol errors even on successful
requests.

* **Allowed Origin:** MirrorRequest (or explicit
  localhost:3000)  
* **Exposed Headers:** grpc-status, grpc-message,
  grpc-status-details-bin

### **4.3 Data Logic**

* **Mock Data:** Vec\<MockShipment\> generated via rand at
  startup (10k rows).  
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

Scenario A: Server-Side (Big Data)  File: DashboardTable.tsx

* **Mode:** Manual.  
* **Configuration:**  TypeScript manualPagination: true,
  manualSorting: true,

* **Effect:** Sort/Page state changes trigger useEffect \-\>
  analyticsClient.getShipments.  Table does *not* sort the
  data array locally.

Scenario B: Client-Side (Small Data)  File: RegionTable.tsx

* **Mode:** Automatic.  
* **Configuration:**  TypeScript getCoreRowModel:
  getCoreRowModel(), getSortedRowModel: getSortedRowModel(),
  // Enables local sorting  // manualSorting is OMITTED/FALSE

* **Effect:** useEffect runs once on mount. Sort state changes
  are handled entirely by the JS engine via getSortedRowModel.

---

## **6\. Build & Run Instructions**

### **6.1 Backend**

**Build Requirement:** protoc (Protocol Buffers Compiler) must
be in $PATH, or available via tonic-build.

Bash

cd backend  \# Compiles .proto via build.rs and starts server
cargo run

### **6.2 Frontend**

**Build Requirement:** Node.js v18+.

Bash

cd frontend  \# 1\. Install Deps  npm install  \# 2\. Generate
TS Client (if proto changed)  npx buf generate ../proto  \#
3\. Start Dev Server npm run dev

## **7\. Known Limitations (PoC)**

1. **Data Persistence:** Backend uses ephemeral in-memory Vec.
Restarting the server resets data.  
2. **Error Handling:** Basic Result types.  Production should
map Results to specific gRPC Status Codes (e.g.,
Status::invalid\_argument).  
3. **Security:** No TLS/SSL implemented. Traffic is plaintext
HTTP. Production requires a reverse proxy (Nginx/Envoy) or
Rust TLS configuration.




INIT



# **Technical Specification: Logistics Analytics Dashboard
(PoC)**

Version: 1.0 (PoC)  Date: November 23, 2025 Architecture:
Hybrid Client/Server Data Processing via gRPC-Web.

---

## **1\. Technology Stack**

### **Backend (Service Layer)**

* **Runtime:** Rust (Edition 2021\)  
* **Framework:** tonic (v0.12) \- Native gRPC implementation
  over HTTP/2.  
* **Web Interop:** tonic-web (v0.12) \- gRPC-Web translation
  layer for browser compatibility.  
* **Async Runtime:** tokio (v1.0+) \- Multi-threaded
  asynchronous execution.  
* **Middleware:** tower-http (v0.5) \- Handles CORS and HTTP
  layer instrumentation.  
* **Serialization:** prost (v0.13) \- Protocol Buffers
  implementation for Rust.

### **Frontend (Presentation Layer)**

* **Framework:** Next.js 14+ (React 18\)  
* **Language:** TypeScript 5+  
* **Network Client:** ConnectRPC (@connectrpc/connect) \-
  Type-safe gRPC client.  
* **Transport:** @connectrpc/connect-web \- Implements the
  gRPC-Web protocol over HTTP/1.1.  
* **State/UI Logic:** TanStack Table v8
  (@tanstack/react-table) \- Headless data grid.

### **Interface Definition**

* **IDL:** Protocol Buffers v3 (proto3)  
* **Transport Protocol:** gRPC-Web
  (application/grpc-web+proto)

---

## **2\. Directory Structure & Key Files**

Plaintext

/fxgraph ├── /proto  │   └── dashboard.proto \# Single Source
of Truth (Schema)  ├── /backend │   ├── build.rs
\# Pre-build script: Compiles .proto to Rust  │   ├──
Cargo.toml            \# Rust dependencies  │ └── src/main.rs
\# Server entry point, Mock DB, Business Logic  └── /frontend
├── buf.gen.yaml          \# Buf configuration for TS
generation  ├── package.json         \# Dependencies
(ConnectRPC, TanStack Table) ├── app/page.tsx         \# Main
UI combining both tables ├── lib/client.ts         \#
ConnectRPC Transport singleton  ├── components/  │   ├──
columns.ts       \# TanStack Column Definitions │ ├──
DashboardTable.tsx \# Server-Side Sort Implementation  │   └──
RegionTable.tsx    \# Client-Side Sort Implementation  └──
gen/ \# Auto-generated TS types (DO NOT EDIT)

---

## **3\. Interface Specification (dashboard.proto)**

The API defines two distinct data access patterns.

Protocol Buffers

syntax \= "proto3";  package dashboard;

service AnalyticsService {  // Pattern A: Server-Side
Processing (Pagination/Sorting)  rpc GetShipments
(ViewRequest) returns (ViewResponse);

  // Pattern B: Client-Side Processing (Snapshot) rpc
  GetRegionStats (Empty) returns (RegionList);  }

message ViewRequest {  int32 page\_number \= 1; int32
items\_per\_page \= 2;  SortOption sort \= 3;        //
Optional: if null, default sort applies  repeated
FilterCriteria filters \= 4;  }

message SortOption {  string column\_id \= 1; bool
is\_ascending \= 2;  }

// ... (See source for full message definitions)

---

## **4\. Backend Implementation Details**

### **4.1 Server Configuration (main.rs)**

The server enables the GrpcWebLayer to translate browser
HTTP/1.1 requests into HTTP/2 gRPC calls.

Crucial Layer Ordering:  Middleware layers are wrapped
outside-in. CORS must be the outermost layer to handle
preflight OPTIONS requests before they reach the gRPC logic.

Rust

Server::builder()  .accept\_http1(true) // Required for
gRPC-Web  .layer(cors\_layer) // 1\. Handle CORS
.layer(GrpcWebLayer::new()) // 2\. Translate gRPC-Web \-\>
gRPC .add\_service(AnalyticsServiceServer::new(service))
.serve(addr)  .await?;

### **4.2 CORS Configuration**

Browsers enforce strict CORS for gRPC-Web. The tower-http
configuration must expose specific gRPC headers, otherwise the
client will throw generic protocol errors even on successful
requests.

* **Allowed Origin:** MirrorRequest (or explicit
  localhost:3000)  
* **Exposed Headers:** grpc-status, grpc-message,
  grpc-status-details-bin

### **4.3 Data Logic**

* **Mock Data:** Vec\<MockShipment\> generated via rand at
  startup (10k rows).  
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

Scenario A: Server-Side (Big Data)  File: DashboardTable.tsx

* **Mode:** Manual.  
* **Configuration:**  TypeScript manualPagination: true,
  manualSorting: true,

* **Effect:** Sort/Page state changes trigger useEffect \-\>
  analyticsClient.getShipments.  Table does *not* sort the
  data array locally.

Scenario B: Client-Side (Small Data)  File: RegionTable.tsx

* **Mode:** Automatic.  
* **Configuration:**  TypeScript getCoreRowModel:
  getCoreRowModel(), getSortedRowModel: getSortedRowModel(),
  // Enables local sorting  // manualSorting is OMITTED/FALSE

* **Effect:** useEffect runs once on mount. Sort state changes
  are handled entirely by the JS engine via getSortedRowModel.

---

## **6\. Build & Run Instructions**

### **6.1 Backend**

**Build Requirement:** protoc (Protocol Buffers Compiler) must
be in $PATH, or available via tonic-build.

Bash

cd backend  \# Compiles .proto via build.rs and starts server
cargo run

### **6.2 Frontend**

**Build Requirement:** Node.js v18+.

Bash

cd frontend  \# 1\. Install Deps  npm install  \# 2\. Generate
TS Client (if proto changed)  npx buf generate ../proto  \#
3\. Start Dev Server npm run dev

## **7\. Known Limitations (PoC)**

1. **Data Persistence:** Backend uses ephemeral in-memory Vec.
Restarting the server resets data.  
2. **Error Handling:** Basic Result types.  Production should
map Results to specific gRPC Status Codes (e.g.,
Status::invalid\_argument).  
3. **Security:** No TLS/SSL implemented. Traffic is plaintext
HTTP. Production requires a reverse proxy (Nginx/Envoy) or
Rust TLS configuration.
