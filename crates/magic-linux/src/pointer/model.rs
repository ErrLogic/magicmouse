#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerDelta {
    pub dx: f32,
    pub dy: f32,
}

impl PointerDelta {
    pub fn new(dx: f32, dy: f32) -> Self {
        Self { dx, dy }
    }

    pub fn is_zero(&self) -> bool {
        self.dx == 0.0 && self.dy == 0.0
    }

    pub fn scale(self, factor: f32) -> Self {
        Self {
            dx: self.dx * factor,
            dy: self.dy * factor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_pointer_delta() {
        let delta = PointerDelta::new(0.25, -0.10);

        assert_eq!(delta.dx, 0.25);
        assert_eq!(delta.dy, -0.10);
    }

    #[test]
    fn detects_zero_delta() {
        let delta = PointerDelta::new(0.0, 0.0);

        assert!(delta.is_zero());
    }

    #[test]
    fn detects_non_zero_delta() {
        let delta = PointerDelta::new(0.1, 0.0);

        assert!(!delta.is_zero());
    }

    #[test]
    fn scales_pointer_delta() {
        let delta = PointerDelta::new(0.10, -0.05);

        let scaled = delta.scale(2.0);

        assert!((scaled.dx - 0.20).abs() < 0.0001);
        assert!((scaled.dy - (-0.10)).abs() < 0.0001);
    }

    #[test]
    fn scaling_preserves_direction() {
        let delta = PointerDelta::new(-0.20, 0.10);

        let scaled = delta.scale(0.5);

        assert!((scaled.dx - (-0.10)).abs() < 0.0001);
        assert!((scaled.dy - 0.05).abs() < 0.0001);
    }
}
