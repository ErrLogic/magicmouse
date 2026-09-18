#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerSensitivity {
    factor: f32,
}

impl PointerSensitivity {
    pub fn new(factor: f32) -> Self {
        Self { factor }
    }

    pub fn factor(&self) -> f32 {
        self.factor
    }

    pub fn apply(&self, delta: super::PointerDelta) -> super::PointerDelta {
        delta.scale(self.factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pointer::PointerDelta;

    #[test]
    fn creates_pointer_sensitivity() {
        let sensitivity = PointerSensitivity::new(2.0);

        assert_eq!(sensitivity.factor(), 2.0);
    }

    #[test]
    fn applies_sensitivity_to_pointer_delta() {
        let sensitivity = PointerSensitivity::new(2.0);
        let delta = PointerDelta::new(0.10, -0.05);

        let result = sensitivity.apply(delta);

        assert!((result.dx - 0.20).abs() < 0.0001);
        assert!((result.dy - (-0.10)).abs() < 0.0001);
    }

    #[test]
    fn sensitivity_preserves_zero_delta() {
        let sensitivity = PointerSensitivity::new(2.0);
        let delta = PointerDelta::new(0.0, 0.0);

        let result = sensitivity.apply(delta);

        assert!(result.is_zero());
    }

    #[test]
    fn sensitivity_can_reduce_pointer_movement() {
        let sensitivity = PointerSensitivity::new(0.5);
        let delta = PointerDelta::new(0.10, -0.04);

        let result = sensitivity.apply(delta);

        assert!((result.dx - 0.05).abs() < 0.0001);
        assert!((result.dy - (-0.02)).abs() < 0.0001);
    }
}
