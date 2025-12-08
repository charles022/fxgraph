
# Main proposal:
- custom JavaScript UI <-> Rust/WASM (client) <-> Rust server
- rust server
    - polars dataframe to hold and work with data
    - single node machine
    - all data exists in a Polars table in memory
        - does not need to request/pull data from external sources like
          drive/disk, other nodes, or from over the network
    - (phase 2) wrap in rayon for handling multiple client requests
- rust/wasm client
    - interacts with the rust server and JS UI
    - receives and interprets requests from JavaScript for all updated data to display (ie when scrolling, filtering, aggregating data)
    - sends the request to the rust server as custom ClientQuery struct 

# future
    - add a proxy/router ahead of the server to distibute requests to the first
      available server, as all data will live on each server




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

