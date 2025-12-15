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
