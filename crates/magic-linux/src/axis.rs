use crate::normalize::NormalizedPoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxisBounds {
    pub min: i32,
    pub max: i32,
}

impl AxisBounds {
    pub fn new(min: i32, max: i32) -> Self {
        Self { min, max }
    }

    pub fn range(&self) -> i32 {
        self.max - self.min
    }

    pub fn contains(&self, value: i32) -> bool {
        value >= self.min && value <= self.max
    }

    pub fn normalize(&self, value: i32) -> f32 {
        let range = self.range();

        if range == 0 {
            return 0.0;
        }

        (value - self.min) as f32 / range as f32
    }

    pub fn clamp(&self, value: i32) -> i32 {
        value.clamp(self.min, self.max)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchBounds {
    pub x: AxisBounds,
    pub y: AxisBounds,
}

impl TouchBounds {
    pub fn new(x: AxisBounds, y: AxisBounds) -> Self {
        Self { x, y }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        self.x.contains(x) && self.y.contains(y)
    }

    pub fn normalize_point(&self, x: i32, y: i32) -> NormalizedPoint {
        NormalizedPoint {
            x: self.x.normalize(x),
            y: self.y.normalize(y),
        }
    }

    pub fn clamp_point(&self, x: i32, y: i32) -> (i32, i32) {
        (self.x.clamp(x), self.y.clamp(y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_value_to_axis_bounds() {
        let bounds = AxisBounds::new(-100, 100);

        assert_eq!(bounds.clamp(-200), -100);
        assert_eq!(bounds.clamp(-50), -50);
        assert_eq!(bounds.clamp(0), 0);
        assert_eq!(bounds.clamp(80), 80);
        assert_eq!(bounds.clamp(200), 100);
    }

    #[test]
    fn clamps_point_to_touch_bounds() {
        let bounds = TouchBounds::new(AxisBounds::new(-100, 100), AxisBounds::new(-200, 200));

        assert_eq!(bounds.clamp_point(-150, 300), (-100, 200));
        assert_eq!(bounds.clamp_point(50, -50), (50, -50));
        assert_eq!(bounds.clamp_point(150, -300), (100, -200));
    }

    #[test]
    fn normalizes_zero_length_axis_to_zero() {
        let bounds = AxisBounds::new(100, 100);

        assert_eq!(bounds.normalize(100), 0.0);
    }

    #[test]
    fn normalizes_axis_boundaries() {
        let bounds = AxisBounds::new(-100, 100);

        assert_eq!(bounds.normalize(-100), 0.0);
        assert_eq!(bounds.normalize(100), 1.0);
    }

    #[test]
    fn normalization_preserves_out_of_range_values() {
        let bounds = AxisBounds::new(-100, 100);

        assert_eq!(bounds.normalize(-200), -0.5);
        assert_eq!(bounds.normalize(200), 1.5);
    }

    #[test]
    fn clamp_then_normalize_stays_within_normalized_range() {
        let bounds = AxisBounds::new(-100, 100);

        let below = bounds.clamp(-200);
        let above = bounds.clamp(200);

        assert_eq!(bounds.normalize(below), 0.0);
        assert_eq!(bounds.normalize(above), 1.0);
    }
}
