# Scenario 2.3 – Merge into persistent state (struct, fixed)

Server emits small POD deltas; client mutates a single fixed struct in place.
```rust
use bytemuck::{Pod, Zeroable};
use wasm_bindgen::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Telemetry { pub cpu: f32, pub mem: f32 }

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct TelemetryDelta { pub cpu_inc: f32, pub mem_inc: f32 }

static mut TELEMETRY: Telemetry = Telemetry { cpu: 0.0, mem: 0.0 };

#[wasm_bindgen]
pub fn apply_telemetry(bytes: &mut [u8]) {
    let delta: &TelemetryDelta = bytemuck::from_bytes(bytes);
    unsafe {
        TELEMETRY.cpu += delta.cpu_inc;
        TELEMETRY.mem += delta.mem_inc;
    }
}
```

## Proof of concept in this folder

`poc/` is a small Rust demo that mirrors the snippet above. A WebSocket server emits raw `TelemetryDelta` bytes; the client casts them with `bytemuck::try_from_bytes`, mutates a long-lived `Telemetry`, and prints the updated numbers.

Run from this folder:
1) Start the server that streams binary deltas every ~950 ms:
```
cd poc
cargo run -- server
```
2) In another shell, connect the client (defaults to `ws://127.0.0.1:3000/ws`):
```
cd poc
cargo run -- client
```

You should see CPU/memory counters grow steadily as each POD delta is merged directly into the fixed struct without any deserialization overhead.
