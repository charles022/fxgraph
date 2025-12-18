# Scenario 5.4 – Compression/encoding variations (Arrow IPC + ZSTD)

Server writes Arrow IPC File with ZSTD compression.
```rust
use arrow::ipc::writer::{FileWriter, IpcWriteOptions};
use arrow::ipc::compression::CompressionType;

let opts = IpcWriteOptions::try_new().unwrap().with_compression(Some(CompressionType::ZSTD));
let mut buf = Vec::new();
let mut w = FileWriter::try_new_with_options(&mut buf, batch.schema(), opts).unwrap();
w.write(&batch).unwrap();
w.finish().unwrap();
```

Client fetches bytes over HTTP and hands them to `arrow::ipc::reader::FileReader`; decompression happens automatically in the reader before Polars ingestion.
```rust
let mut reader = arrow::ipc::reader::FileReader::try_new(std::io::Cursor::new(bytes), None).unwrap();
let mut batches = Vec::new();
for batch in reader { batches.push(batch.unwrap()); }
let df = polars::prelude::DataFrame::try_from(batches).unwrap();
```
