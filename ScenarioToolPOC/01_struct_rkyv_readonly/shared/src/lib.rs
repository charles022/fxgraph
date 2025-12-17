use rkyv::{Archive, Deserialize, Serialize};

/// Basic world snapshot shared between server and wasm client.
#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[archive_attr(derive(Debug))]
#[archive(check_bytes)]
pub struct GameState {
    pub tick: u64,
    pub score: u64,
    pub positions: Vec<(f32, f32)>,
}

impl GameState {
    pub fn new(tick: u64, score: u64, positions: Vec<(f32, f32)>) -> Self {
        Self {
            tick,
            score,
            positions,
        }
    }

    /// Create a predictable snapshot with a few moving points.
    pub fn moving_points(tick: u64) -> Self {
        let score = 40 + (tick % 10);
        let positions = (0..4)
            .map(|i| {
                let phase = (tick as f32 * 0.2) + i as f32;
                (phase.sin() * 50.0, phase.cos() * 50.0)
            })
            .collect();
        Self::new(tick, score, positions)
    }
}
