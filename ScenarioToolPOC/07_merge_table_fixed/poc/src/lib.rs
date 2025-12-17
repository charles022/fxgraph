use anyhow::{anyhow, Result};
use bytemuck::{Pod, Zeroable};
use std::sync::Mutex;

pub const TABLE_CAPACITY: usize = 1024;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Row {
    pub id: u32,
    pub temperature: f32,
}

static TABLE: Mutex<[Row; TABLE_CAPACITY]> =
    Mutex::new([Row { id: 0, temperature: 0.0 }; TABLE_CAPACITY]);

/// Copy a raw byte payload (array of `Row`) into the front of the table.
pub fn apply_patch(bytes: &[u8]) -> Result<usize> {
    let patch: &[Row] = bytemuck::try_cast_slice(bytes)
        .map_err(|e| anyhow!("patch payload is not aligned to Row: {e}"))?;

    let mut table = TABLE.lock().map_err(|_| anyhow!("table lock poisoned"))?;
    let written = patch.len().min(table.len());
    table[..written].copy_from_slice(&patch[..written]);
    Ok(written)
}

pub fn snapshot_front(limit: usize) -> Vec<Row> {
    let table = TABLE
        .lock()
        .unwrap_or_else(|_| panic!("table lock poisoned while snapshotting"));
    table.iter().take(limit).copied().collect()
}

pub fn reset_table() {
    if let Ok(mut table) = TABLE.lock() {
        table.fill(Row { id: 0, temperature: 0.0 });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_overwrites_rows() {
        reset_table();

        let patch_rows = vec![
            Row {
                id: 1,
                temperature: 12.5,
            },
            Row {
                id: 2,
                temperature: -1.25,
            },
            Row {
                id: 3,
                temperature: 33.0,
            },
        ];
        let bytes = bytemuck::cast_slice(&patch_rows).to_vec();

        let written = apply_patch(&bytes).expect("patch should write");
        assert_eq!(written, 3);

        let snapshot = snapshot_front(3);
        assert_eq!(snapshot[0].id, 1);
        assert!((snapshot[1].temperature + 1.25).abs() < f32::EPSILON);
        assert_eq!(snapshot[2].temperature, 33.0);
    }
}
