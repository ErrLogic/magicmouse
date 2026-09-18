use super::model::Gesture;
use super::movement::TrackedTouch;
use super::state::GestureState;
use super::threshold::MovementThreshold;

#[derive(Debug, Clone, Copy)]
pub struct ThreeFingerSwipeRecognizer {
    threshold: MovementThreshold,
}

impl ThreeFingerSwipeRecognizer {
    pub fn new(threshold: MovementThreshold) -> Self {
        Self { threshold }
    }

    pub fn recognize(&self, state: &GestureState) -> Option<Gesture> {
        if state.finger_count() != 3 {
            return None;
        }

        let touches = state.touches();

        let movements: Vec<_> = touches.iter().map(TrackedTouch::movement).collect();

        if !movements
            .iter()
            .all(|movement| movement.exceeds_threshold(self.threshold))
        {
            return None;
        }

        let direction = movements[0].direction()?;

        if movements
            .iter()
            .all(|movement| movement.direction() == Some(direction))
        {
            Some(Gesture::Swipe {
                fingers: 3,
                direction,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gesture::Direction;
    use crate::normalize::NormalizedPoint;

    fn point(x: f32, y: f32) -> NormalizedPoint {
        NormalizedPoint { x, y }
    }

    fn create_three_finger_state(
        positions: [(f32, f32); 3],
        movements: [(f32, f32); 3],
    ) -> GestureState {
        let mut state = GestureState::new();

        for (index, ((x, y), (dx, dy))) in positions.into_iter().zip(movements).enumerate() {
            let tracking_id = (index + 1) as i32;

            state.add_touch(TrackedTouch::new(tracking_id, point(x, y)));

            state.update_touch(tracking_id, point(x + dx, y + dy));
        }

        state
    }

    #[test]
    fn recognizes_three_finger_right_swipe() {
        let recognizer = ThreeFingerSwipeRecognizer::new(MovementThreshold::new(0.15));

        let state = create_three_finger_state(
            [(0.10, 0.30), (0.10, 0.50), (0.10, 0.70)],
            [(0.30, 0.0), (0.35, 0.0), (0.40, 0.0)],
        );

        assert_eq!(
            recognizer.recognize(&state),
            Some(Gesture::Swipe {
                fingers: 3,
                direction: Direction::Right,
            })
        );
    }

    #[test]
    fn recognizes_three_finger_left_swipe() {
        let recognizer = ThreeFingerSwipeRecognizer::new(MovementThreshold::new(0.15));

        let state = create_three_finger_state(
            [(0.70, 0.30), (0.70, 0.50), (0.70, 0.70)],
            [(-0.30, 0.0), (-0.35, 0.0), (-0.40, 0.0)],
        );

        assert_eq!(
            recognizer.recognize(&state),
            Some(Gesture::Swipe {
                fingers: 3,
                direction: Direction::Left,
            })
        );
    }

    #[test]
    fn rejects_non_three_finger_gesture() {
        let recognizer = ThreeFingerSwipeRecognizer::new(MovementThreshold::new(0.15));

        let mut state = GestureState::new();

        state.add_touch(TrackedTouch::new(1, point(0.10, 0.30)));

        state.update_touch(1, point(0.40, 0.30));

        assert_eq!(recognizer.recognize(&state), None);
    }

    #[test]
    fn rejects_three_finger_swipe_when_one_finger_is_below_threshold() {
        let recognizer = ThreeFingerSwipeRecognizer::new(MovementThreshold::new(0.15));

        let state = create_three_finger_state(
            [(0.10, 0.30), (0.10, 0.50), (0.10, 0.70)],
            [(0.30, 0.0), (0.10, 0.0), (0.40, 0.0)],
        );

        assert_eq!(recognizer.recognize(&state), None);
    }

    #[test]
    fn rejects_three_finger_swipe_when_direction_is_inconsistent() {
        let recognizer = ThreeFingerSwipeRecognizer::new(MovementThreshold::new(0.15));

        let state = create_three_finger_state(
            [(0.10, 0.30), (0.10, 0.50), (0.10, 0.70)],
            [(0.30, 0.0), (-0.30, 0.0), (0.40, 0.0)],
        );

        assert_eq!(recognizer.recognize(&state), None);
    }
}
