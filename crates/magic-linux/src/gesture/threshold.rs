#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovementThreshold {
    pub distance: f32,
}

impl MovementThreshold {
    pub fn new(distance: f32) -> Self {
        Self { distance }
    }
}
