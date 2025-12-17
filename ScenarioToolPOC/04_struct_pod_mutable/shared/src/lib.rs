use bytemuck::{Pod, Zeroable};

/// Fixed-size state shared between server and WASM client.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PhysicsState {
    pub x: f32,
    pub y: f32,
    pub id: u32,
}

impl PhysicsState {
    /// Deterministic snapshot used for the demo stream.
    pub const fn demo() -> Self {
        Self {
            x: 10.0,
            y: 0.5,
            id: 1,
        }
    }

    pub const fn sum(&self) -> f32 {
        self.x + self.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};

    #[test]
    fn layout_is_stable() {
        assert_eq!(size_of::<PhysicsState>(), 12);
        assert_eq!(align_of::<PhysicsState>(), 4);
    }
}
