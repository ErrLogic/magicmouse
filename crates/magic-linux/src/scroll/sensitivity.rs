use super::ScrollDelta;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollSensitivity {
    factor: f32,
}

impl ScrollSensitivity {
    pub fn new(factor: f32) -> Self {
        Self { factor }
    }

    pub fn factor(&self) -> f32 {
        self.factor
    }

    pub fn apply(&self, delta: ScrollDelta) -> ScrollDelta {
        delta.scale(self.factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_scroll_sensitivity() {
        let sensitivity = ScrollSensitivity::new(2.0);

        assert_eq!(sensitivity.factor(), 2.0);
    }

    #[test]
    fn applies_sensitivity_to_scroll_delta() {
        let sensitivity = ScrollSensitivity::new(2.0);
        let delta = ScrollDelta::new(0.10, -0.05);

        let result = sensitivity.apply(delta);

        assert!((result.dx - 0.20).abs() < 0.0001);
        assert!((result.dy - (-0.10)).abs() < 0.0001);
    }

    #[test]
    fn sensitivity_preserves_zero_delta() {
        let sensitivity = ScrollSensitivity::new(2.0);
        let delta = ScrollDelta::new(0.0, 0.0);

        let result = sensitivity.apply(delta);

        assert!(result.is_zero());
    }

    #[test]
    fn sensitivity_can_reduce_scroll_movement() {
        let sensitivity = ScrollSensitivity::new(0.5);
        let delta = ScrollDelta::new(0.10, -0.04);

        let result = sensitivity.apply(delta);

        assert!((result.dx - 0.05).abs() < 0.0001);
        assert!((result.dy - (-0.02)).abs() < 0.0001);
    }
}
