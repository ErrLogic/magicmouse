use crate::normalize::NormalizedPoint;

use super::model::PointerDelta;

pub struct PointerMovement;

impl PointerMovement {
    pub fn from_points(previous: NormalizedPoint, current: NormalizedPoint) -> PointerDelta {
        PointerDelta::new(current.x - previous.x, current.y - previous.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_pointer_delta() {
        let previous = NormalizedPoint { x: 0.40, y: 0.50 };

        let current = NormalizedPoint { x: 0.43, y: 0.48 };

        let delta = PointerMovement::from_points(previous, current);

        assert!((delta.dx - 0.03).abs() < 0.0001);
        assert!((delta.dy - (-0.02)).abs() < 0.0001);
    }

    #[test]
    fn calculates_negative_pointer_delta() {
        let previous = NormalizedPoint { x: 0.70, y: 0.60 };

        let current = NormalizedPoint { x: 0.20, y: 0.10 };

        let delta = PointerMovement::from_points(previous, current);

        assert!((delta.dx - (-0.50)).abs() < 0.0001);
        assert!((delta.dy - (-0.50)).abs() < 0.0001);
    }

    #[test]
    fn calculates_zero_pointer_delta() {
        let point = NormalizedPoint { x: 0.50, y: 0.50 };

        let delta = PointerMovement::from_points(point, point);

        assert!(delta.is_zero());
    }
}
