use super::model::{Gesture, GestureEvent, GesturePhase};
use super::state::GestureState;
use super::swipe::ThreeFingerSwipeRecognizer;
use super::threshold::MovementThreshold;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureRecognitionState {
    Idle,
    Active { gesture: Gesture },
}

#[derive(Debug, Clone, Copy)]
pub struct GestureRecognizer {
    swipe: ThreeFingerSwipeRecognizer,
    state: GestureRecognitionState,
}

impl GestureRecognizer {
    pub fn new(threshold: MovementThreshold) -> Self {
        Self {
            swipe: ThreeFingerSwipeRecognizer::new(threshold),
            state: GestureRecognitionState::Idle,
        }
    }

    pub fn state(&self) -> GestureRecognitionState {
        self.state
    }

    pub fn process(&mut self, state: &GestureState) -> Option<GestureEvent> {
        match self.state {
            GestureRecognitionState::Idle => {
                let gesture = self.swipe.recognize(state)?;

                self.state = GestureRecognitionState::Active { gesture };

                Some(GestureEvent {
                    gesture,
                    phase: GesturePhase::Started,
                })
            }

            GestureRecognitionState::Active { gesture } => {
                if state.finger_count() != 3 {
                    self.state = GestureRecognitionState::Idle;

                    return Some(GestureEvent {
                        gesture,
                        phase: GesturePhase::Ended,
                    });
                }

                Some(GestureEvent {
                    gesture,
                    phase: GesturePhase::Updated,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gesture::Direction;
    use crate::gesture::TrackedTouch;
    use crate::normalize::NormalizedPoint;

    fn point(x: f32, y: f32) -> NormalizedPoint {
        NormalizedPoint { x, y }
    }

    fn create_three_finger_right_state() -> GestureState {
        let mut state = GestureState::new();

        state.add_touch(TrackedTouch::new(1, point(0.10, 0.30)));
        state.add_touch(TrackedTouch::new(2, point(0.10, 0.50)));
        state.add_touch(TrackedTouch::new(3, point(0.10, 0.70)));

        state.update_touch(1, point(0.40, 0.30));
        state.update_touch(2, point(0.45, 0.50));
        state.update_touch(3, point(0.50, 0.70));

        state
    }

    fn expected_right_swipe() -> Gesture {
        Gesture::Swipe {
            fingers: 3,
            direction: Direction::Right,
        }
    }

    #[test]
    fn gesture_lifecycle_starts_when_three_finger_swipe_is_recognized() {
        let mut recognizer = GestureRecognizer::new(MovementThreshold::new(0.15));
        let state = create_three_finger_right_state();

        let event = recognizer.process(&state);

        assert_eq!(
            event,
            Some(GestureEvent {
                gesture: expected_right_swipe(),
                phase: GesturePhase::Started,
            })
        );

        assert_eq!(
            recognizer.state(),
            GestureRecognitionState::Active {
                gesture: expected_right_swipe(),
            }
        );
    }

    #[test]
    fn gesture_lifecycle_updates_active_gesture() {
        let mut recognizer = GestureRecognizer::new(MovementThreshold::new(0.15));
        let state = create_three_finger_right_state();

        recognizer.process(&state);

        let event = recognizer.process(&state);

        assert_eq!(
            event,
            Some(GestureEvent {
                gesture: expected_right_swipe(),
                phase: GesturePhase::Updated,
            })
        );
    }

    #[test]
    fn gesture_lifecycle_ends_when_finger_count_changes() {
        let mut recognizer = GestureRecognizer::new(MovementThreshold::new(0.15));
        let mut state = create_three_finger_right_state();

        recognizer.process(&state);

        state.remove_touch(3);

        let event = recognizer.process(&state);

        assert_eq!(
            event,
            Some(GestureEvent {
                gesture: expected_right_swipe(),
                phase: GesturePhase::Ended,
            })
        );

        assert_eq!(recognizer.state(), GestureRecognitionState::Idle);
    }

    #[test]
    fn gesture_does_not_start_below_threshold() {
        let mut recognizer = GestureRecognizer::new(MovementThreshold::new(0.15));
        let mut state = GestureState::new();

        state.add_touch(TrackedTouch::new(1, point(0.10, 0.30)));
        state.add_touch(TrackedTouch::new(2, point(0.10, 0.50)));
        state.add_touch(TrackedTouch::new(3, point(0.10, 0.70)));

        state.update_touch(1, point(0.20, 0.30));
        state.update_touch(2, point(0.20, 0.50));
        state.update_touch(3, point(0.20, 0.70));

        assert_eq!(recognizer.process(&state), None);
        assert_eq!(recognizer.state(), GestureRecognitionState::Idle);
    }

    #[test]
    fn gesture_emits_started_only_once() {
        let mut recognizer = GestureRecognizer::new(MovementThreshold::new(0.15));
        let state = create_three_finger_right_state();

        let first = recognizer.process(&state);
        let second = recognizer.process(&state);

        assert_eq!(first.map(|event| event.phase), Some(GesturePhase::Started));

        assert_eq!(second.map(|event| event.phase), Some(GesturePhase::Updated));
    }
}
