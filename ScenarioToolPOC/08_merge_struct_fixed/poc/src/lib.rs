use std::mem::size_of;
use std::sync::Mutex;

use anyhow::{anyhow, Result};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Telemetry {
    pub cpu: f32,
    pub mem: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TelemetryDelta {
    pub cpu_inc: f32,
    pub mem_inc: f32,
}

static TELEMETRY: Mutex<Telemetry> = Mutex::new(Telemetry { cpu: 0.0, mem: 0.0 });

/// Apply a raw POD delta to the persistent telemetry struct.
/// The payload must be exactly `size_of::<TelemetryDelta>()` bytes long.
pub fn apply_telemetry(bytes: &[u8]) -> Result<Telemetry> {
    let delta: &TelemetryDelta = bytemuck::try_from_bytes(bytes)
        .map_err(|e| anyhow!("delta payload must be {} bytes: {e}", size_of::<TelemetryDelta>()))?;

    let mut telemetry = TELEMETRY
        .lock()
        .map_err(|_| anyhow!("telemetry lock poisoned"))?;

    telemetry.cpu += delta.cpu_inc;
    telemetry.mem += delta.mem_inc;
    Ok(*telemetry)
}

pub fn snapshot() -> Telemetry {
    TELEMETRY
        .lock()
        .unwrap_or_else(|_| panic!("telemetry lock poisoned while snapshotting"))
        .to_owned()
}

pub fn reset() {
    if let Ok(mut telemetry) = TELEMETRY.lock() {
        *telemetry = Telemetry::zeroed();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_delta_from_bytes() {
        reset();
        let delta = TelemetryDelta {
            cpu_inc: 1.25,
            mem_inc: -4.0,
        };
        let bytes = bytemuck::bytes_of(&delta);

        let updated = apply_telemetry(bytes).expect("should accept aligned delta bytes");
        assert!((updated.cpu - 1.25).abs() < f32::EPSILON);
        assert!((updated.mem + 4.0).abs() < f32::EPSILON);

        let after = snapshot();
        assert_eq!(after.cpu, updated.cpu);
        assert_eq!(after.mem, updated.mem);
    }

    #[test]
    fn rejects_wrong_length() {
        reset();
        let bad_bytes = [0_u8; 3];
        let err = apply_telemetry(&bad_bytes).unwrap_err();
        assert!(err.to_string().contains("delta payload"));
    }
}
