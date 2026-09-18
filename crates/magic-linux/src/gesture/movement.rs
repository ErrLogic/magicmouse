use crate::normalize::NormalizedPoint;

use super::model::Direction;
use super::threshold::MovementThreshold;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackedTouch {
    pub slot: Option<i32>,
    pub tracking_id: i32,
    pub start: NormalizedPoint,
    pub current: NormalizedPoint,
}

impl TrackedTouch {
    pub fn new(tracking_id: i32, position: NormalizedPoint) -> Self {
        Self {
            slot: None,
            tracking_id,
            start: position,
            current: position,
        }
    }

    pub fn with_slot(slot: i32, tracking_id: i32, position: NormalizedPoint) -> Self {
        Self {
            slot: Some(slot),
            tracking_id,
            start: position,
            current: position,
        }
    }

    pub fn update(&mut self, position: NormalizedPoint) {
        self.current = position;
    }

    pub fn movement(&self) -> MovementVector {
        MovementVector::new(self.current.x - self.start.x, self.current.y - self.start.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovementVector {
    pub dx: f32,
    pub dy: f32,
}

impl MovementVector {
    pub fn new(dx: f32, dy: f32) -> Self {
        Self { dx, dy }
    }

    pub fn direction(&self) -> Option<Direction> {
        if self.dx.abs() > self.dy.abs() {
            if self.dx > 0.0 {
                Some(Direction::Right)
            } else if self.dx < 0.0 {
                Some(Direction::Left)
            } else {
                None
            }
        } else if self.dy > 0.0 {
            Some(Direction::Down)
        } else if self.dy < 0.0 {
            Some(Direction::Up)
        } else {
            None
        }
    }

    pub fn magnitude(&self) -> f32 {
        self.dx.hypot(self.dy)
    }

    pub fn exceeds_threshold(&self, threshold: MovementThreshold) -> bool {
        self.magnitude() >= threshold.distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize::NormalizedPoint;

    #[test]
    fn creates_tracked_touch_from_start_position() {
        let position = NormalizedPoint { x: 0.25, y: 0.5 };

        let touch = TrackedTouch::new(42, position);

        assert_eq!(touch.slot, None);
        assert_eq!(touch.tracking_id, 42);
        assert_eq!(touch.start, position);
        assert_eq!(touch.current, position);
    }

    #[test]
    fn creates_tracked_touch_with_slot() {
        let position = NormalizedPoint { x: 0.25, y: 0.5 };

        let touch = TrackedTouch::with_slot(3, 42, position);

        assert_eq!(touch.slot, Some(3));
        assert_eq!(touch.tracking_id, 42);
        assert_eq!(touch.start, position);
        assert_eq!(touch.current, position);
    }

    #[test]
    fn updates_current_position_without_changing_start() {
        let start = NormalizedPoint { x: 0.25, y: 0.5 };
        let current = NormalizedPoint { x: 0.75, y: 0.55 };

        let mut touch = TrackedTouch::new(42, start);

        touch.update(current);

        assert_eq!(touch.start, start);
        assert_eq!(touch.current, current);
    }

    #[test]
    fn calculates_movement_vector() {
        let start = NormalizedPoint { x: 0.25, y: 0.50 };
        let current = NormalizedPoint { x: 0.75, y: 0.55 };

        let touch = TrackedTouch::new(42, start);
        let mut touch = touch;

        touch.update(current);

        let movement = touch.movement();

        assert!((movement.dx - 0.50).abs() < 0.0001);
        assert!((movement.dy - 0.05).abs() < 0.0001);
    }

    #[test]
    fn calculates_movement_magnitude() {
        let movement = MovementVector::new(3.0, 4.0);

        assert!((movement.magnitude() - 5.0).abs() < 0.0001);
    }

    #[test]
    fn detects_right_direction() {
        let movement = MovementVector::new(1.0, 0.0);

        assert_eq!(movement.direction(), Some(Direction::Right));
    }

    #[test]
    fn detects_left_direction() {
        let movement = MovementVector::new(-1.0, 0.0);

        assert_eq!(movement.direction(), Some(Direction::Left));
    }

    #[test]
    fn detects_down_direction() {
        let movement = MovementVector::new(0.0, 1.0);

        assert_eq!(movement.direction(), Some(Direction::Down));
    }

    #[test]
    fn detects_up_direction() {
        let movement = MovementVector::new(0.0, -1.0);

        assert_eq!(movement.direction(), Some(Direction::Up));
    }

    #[test]
    fn detects_no_direction_without_movement() {
        let movement = MovementVector::new(0.0, 0.0);

        assert_eq!(movement.direction(), None);
    }

    #[test]
    fn movement_vector_can_be_negative() {
        let movement = MovementVector::new(-0.5, -0.25);

        assert_eq!(movement.dx, -0.5);
        assert_eq!(movement.dy, -0.25);
    }

    #[test]
    fn movement_vector_is_zero_without_movement() {
        let movement = MovementVector::new(0.0, 0.0);

        assert_eq!(movement.dx, 0.0);
        assert_eq!(movement.dy, 0.0);
    }

    #[test]
    fn dominant_axis_determines_direction() {
        let movement = MovementVector::new(0.8, 0.2);

        assert_eq!(movement.direction(), Some(Direction::Right));
    }

    #[test]
    fn equal_movement_components_prefer_vertical() {
        let movement = MovementVector::new(0.5, 0.5);

        assert_eq!(movement.direction(), Some(Direction::Down));
    }

    #[test]
    fn detects_movement_above_threshold() {
        let movement = MovementVector::new(0.5, 0.0);
        let threshold = MovementThreshold::new(0.25);

        assert!(movement.exceeds_threshold(threshold));
    }

    #[test]
    fn detects_movement_below_threshold() {
        let movement = MovementVector::new(0.1, 0.0);
        let threshold = MovementThreshold::new(0.25);

        assert!(!movement.exceeds_threshold(threshold));
    }

    #[test]
    fn threshold_boundary_is_inclusive() {
        let movement = MovementVector::new(0.25, 0.0);
        let threshold = MovementThreshold::new(0.25);

        assert!(movement.exceeds_threshold(threshold));
    }

    #[test]
    fn zero_movement_does_not_exceed_threshold() {
        let movement = MovementVector::new(0.0, 0.0);
        let threshold = MovementThreshold::new(0.25);

        assert!(!movement.exceeds_threshold(threshold));
    }
}
