use crate::gesture::Direction;
use crate::normalize::NormalizedTouchFrame;
use crate::touch::TouchState;

const DEFAULT_MOVEMENT_THRESHOLD: f32 = 0.15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TwoFingerSwipeEvent {
    pub direction: Direction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SwipeState {
    Idle,
    Tracking,
    Recognized,
    Cancelled,
}

#[derive(Debug)]
pub struct TwoFingerSwipeRecognizer {
    movement_threshold: f32,
    state: SwipeState,
    start_positions: [(i32, f32, f32); 2],
}

impl TwoFingerSwipeRecognizer {
    pub fn new() -> Self {
        Self {
            movement_threshold: DEFAULT_MOVEMENT_THRESHOLD,
            state: SwipeState::Idle,
            start_positions: [(0, 0.0, 0.0); 2],
        }
    }

    pub fn process(&mut self, frame: &NormalizedTouchFrame) -> Option<TwoFingerSwipeEvent> {
        let active_touches = frame
            .touches
            .iter()
            .filter(|touch| {
                touch.tracking_id.is_some()
                    && matches!(touch.state, TouchState::Down | TouchState::Move)
            })
            .collect::<Vec<_>>();

        if active_touches.len() != 2 {
            self.reset_if_released(frame);
            return None;
        }

        let first = active_touches[0];
        let second = active_touches[1];

        match self.state {
            SwipeState::Idle => {
                if first.state == TouchState::Down || second.state == TouchState::Down {
                    self.start_positions = [
                        (
                            first.tracking_id.unwrap(),
                            first.position.x,
                            first.position.y,
                        ),
                        (
                            second.tracking_id.unwrap(),
                            second.position.x,
                            second.position.y,
                        ),
                    ];

                    self.state = SwipeState::Tracking;
                }

                None
            }

            SwipeState::Tracking => {
                let Some((first_start_x, first_start_y)) =
                    self.start_position_for(first.tracking_id.unwrap())
                else {
                    self.cancel();
                    return None;
                };

                let Some((second_start_x, second_start_y)) =
                    self.start_position_for(second.tracking_id.unwrap())
                else {
                    self.cancel();
                    return None;
                };

                let first_dx = first.position.x - first_start_x;
                let first_dy = first.position.y - first_start_y;

                let second_dx = second.position.x - second_start_x;
                let second_dy = second.position.y - second_start_y;

                let first_horizontal = first_dx.abs();
                let second_horizontal = second_dx.abs();

                let first_vertical = first_dy.abs();
                let second_vertical = second_dy.abs();

                let same_horizontal_direction =
                    (first_dx >= 0.0 && second_dx >= 0.0) || (first_dx <= 0.0 && second_dx <= 0.0);

                if !same_horizontal_direction {
                    self.cancel();
                    return None;
                }

                let horizontal_movement = (first_dx + second_dx) / 2.0;
                let vertical_movement = (first_vertical + second_vertical) / 2.0;

                // A sufficiently large vertical movement means this
                // gesture is not a horizontal swipe.
                if vertical_movement >= self.movement_threshold
                    && vertical_movement >= horizontal_movement.abs()
                {
                    self.cancel();
                    return None;
                }

                let minimum_horizontal_movement = first_horizontal.min(second_horizontal);

                if minimum_horizontal_movement < self.movement_threshold {
                    return None;
                }

                let direction = if horizontal_movement > 0.0 {
                    Direction::Right
                } else {
                    Direction::Left
                };

                self.state = SwipeState::Recognized;

                Some(TwoFingerSwipeEvent { direction })
            }

            SwipeState::Recognized | SwipeState::Cancelled => None,
        }
    }

    fn start_position_for(&self, tracking_id: i32) -> Option<(f32, f32)> {
        self.start_positions
            .iter()
            .find(|(id, _, _)| *id == tracking_id)
            .map(|(_, x, y)| (*x, *y))
    }

    fn cancel(&mut self) {
        self.state = SwipeState::Cancelled;
    }

    fn reset_if_released(&mut self, frame: &NormalizedTouchFrame) {
        let has_release = frame
            .touches
            .iter()
            .any(|touch| touch.state == TouchState::Up);

        if has_release {
            self.state = SwipeState::Idle;
            self.start_positions = [(0, 0.0, 0.0); 2];
        }
    }
}

impl Default for TwoFingerSwipeRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize::{NormalizedPoint, NormalizedTouch};
    use crate::touch::TouchState;

    fn touch(tracking_id: i32, x: f32, y: f32, state: TouchState) -> NormalizedTouch {
        NormalizedTouch {
            slot: tracking_id,
            tracking_id: Some(tracking_id),
            position: NormalizedPoint { x, y },
            state,
        }
    }

    fn frame(touches: Vec<NormalizedTouch>) -> NormalizedTouchFrame {
        NormalizedTouchFrame { touches }
    }

    #[test]
    fn first_two_finger_frame_does_not_emit_swipe() {
        let mut recognizer = TwoFingerSwipeRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.3, 0.5, TouchState::Down),
                touch(2, 0.4, 0.5, TouchState::Down),
            ])),
            None
        );
    }

    #[test]
    fn recognizes_right_swipe() {
        let mut recognizer = TwoFingerSwipeRecognizer::new();

        recognizer.process(&frame(vec![
            touch(1, 0.3, 0.5, TouchState::Down),
            touch(2, 0.4, 0.5, TouchState::Down),
        ]));

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.5, 0.5, TouchState::Move),
                touch(2, 0.6, 0.5, TouchState::Move),
            ])),
            Some(TwoFingerSwipeEvent {
                direction: Direction::Right,
            })
        );
    }

    #[test]
    fn recognizes_left_swipe() {
        let mut recognizer = TwoFingerSwipeRecognizer::new();

        recognizer.process(&frame(vec![
            touch(1, 0.5, 0.5, TouchState::Down),
            touch(2, 0.6, 0.5, TouchState::Down),
        ]));

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.3, 0.5, TouchState::Move),
                touch(2, 0.4, 0.5, TouchState::Move),
            ])),
            Some(TwoFingerSwipeEvent {
                direction: Direction::Left,
            })
        );
    }

    #[test]
    fn does_not_recognize_movement_below_threshold() {
        let mut recognizer = TwoFingerSwipeRecognizer::new();

        recognizer.process(&frame(vec![
            touch(1, 0.3, 0.5, TouchState::Down),
            touch(2, 0.4, 0.5, TouchState::Down),
        ]));

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.36, 0.5, TouchState::Move),
                touch(2, 0.46, 0.5, TouchState::Move),
            ])),
            None
        );
    }

    #[test]
    fn does_not_recognize_vertical_movement_as_horizontal_swipe() {
        let mut recognizer = TwoFingerSwipeRecognizer::new();

        recognizer.process(&frame(vec![
            touch(1, 0.3, 0.5, TouchState::Down),
            touch(2, 0.4, 0.5, TouchState::Down),
        ]));

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.3, 0.7, TouchState::Move),
                touch(2, 0.4, 0.7, TouchState::Move),
            ])),
            None
        );
    }

    #[test]
    fn equal_movement_prefers_vertical_axis() {
        let mut recognizer = TwoFingerSwipeRecognizer::new();

        recognizer.process(&frame(vec![
            touch(1, 0.3, 0.5, TouchState::Down),
            touch(2, 0.4, 0.5, TouchState::Down),
        ]));

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.46, 0.66, TouchState::Move),
                touch(2, 0.56, 0.66, TouchState::Move),
            ])),
            None
        );
    }

    #[test]
    fn emits_only_once_per_swipe() {
        let mut recognizer = TwoFingerSwipeRecognizer::new();

        recognizer.process(&frame(vec![
            touch(1, 0.3, 0.5, TouchState::Down),
            touch(2, 0.4, 0.5, TouchState::Down),
        ]));

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.5, 0.5, TouchState::Move),
                touch(2, 0.6, 0.5, TouchState::Move),
            ])),
            Some(TwoFingerSwipeEvent {
                direction: Direction::Right,
            })
        );

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.7, 0.5, TouchState::Move),
                touch(2, 0.8, 0.5, TouchState::Move),
            ])),
            None
        );
    }

    #[test]
    fn release_resets_swipe_lifecycle() {
        let mut recognizer = TwoFingerSwipeRecognizer::new();

        recognizer.process(&frame(vec![
            touch(1, 0.3, 0.5, TouchState::Down),
            touch(2, 0.4, 0.5, TouchState::Down),
        ]));

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.5, 0.5, TouchState::Move),
                touch(2, 0.6, 0.5, TouchState::Move),
            ])),
            Some(TwoFingerSwipeEvent {
                direction: Direction::Right,
            })
        );

        recognizer.process(&frame(vec![
            touch(1, 0.5, 0.5, TouchState::Up),
            touch(2, 0.6, 0.5, TouchState::Up),
        ]));

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.3, 0.5, TouchState::Down),
                touch(2, 0.4, 0.5, TouchState::Down),
            ])),
            None
        );
    }

    #[test]
    fn multiple_fingers_cancel_swipe() {
        let mut recognizer = TwoFingerSwipeRecognizer::new();

        recognizer.process(&frame(vec![
            touch(1, 0.3, 0.5, TouchState::Down),
            touch(2, 0.4, 0.5, TouchState::Down),
        ]));

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.4, 0.5, TouchState::Move),
                touch(2, 0.5, 0.5, TouchState::Move),
                touch(3, 0.6, 0.5, TouchState::Down),
            ])),
            None
        );
    }

    #[test]
    fn new_swipe_can_start_after_cancelled_gesture() {
        let mut recognizer = TwoFingerSwipeRecognizer::new();

        recognizer.process(&frame(vec![
            touch(1, 0.3, 0.5, TouchState::Down),
            touch(2, 0.4, 0.5, TouchState::Down),
        ]));

        recognizer.process(&frame(vec![
            touch(1, 0.3, 0.7, TouchState::Move),
            touch(2, 0.4, 0.7, TouchState::Move),
        ]));

        recognizer.process(&frame(vec![
            touch(1, 0.3, 0.7, TouchState::Up),
            touch(2, 0.4, 0.7, TouchState::Up),
        ]));

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(3, 0.3, 0.5, TouchState::Down),
                touch(4, 0.4, 0.5, TouchState::Down),
            ])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(3, 0.5, 0.5, TouchState::Move),
                touch(4, 0.6, 0.5, TouchState::Move),
            ])),
            Some(TwoFingerSwipeEvent {
                direction: Direction::Right,
            })
        );
    }

    #[test]
    fn tracking_id_change_does_not_create_new_swipe() {
        let mut recognizer = TwoFingerSwipeRecognizer::new();

        recognizer.process(&frame(vec![
            touch(1, 0.3, 0.5, TouchState::Down),
            touch(2, 0.4, 0.5, TouchState::Down),
        ]));

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.5, 0.5, TouchState::Move),
                touch(2, 0.6, 0.5, TouchState::Move),
            ])),
            Some(TwoFingerSwipeEvent {
                direction: Direction::Right,
            })
        );

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(3, 0.7, 0.5, TouchState::Move),
                touch(2, 0.8, 0.5, TouchState::Move),
            ])),
            None
        );
    }

    #[test]
    fn recognizes_movement_just_above_threshold() {
        let mut recognizer = TwoFingerSwipeRecognizer::new();

        recognizer.process(&frame(vec![
            touch(1, 0.3, 0.5, TouchState::Down),
            touch(2, 0.4, 0.5, TouchState::Down),
        ]));

        let threshold = DEFAULT_MOVEMENT_THRESHOLD;
        let movement = threshold + f32::EPSILON;

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.3 + movement, 0.5, TouchState::Move),
                touch(2, 0.4 + movement, 0.5, TouchState::Move),
            ])),
            Some(TwoFingerSwipeEvent {
                direction: Direction::Right,
            })
        );
    }

    #[test]
    fn does_not_recognize_opposite_horizontal_movement() {
        let mut recognizer = TwoFingerSwipeRecognizer::new();

        recognizer.process(&frame(vec![
            touch(1, 0.30, 0.50, TouchState::Down),
            touch(2, 0.40, 0.50, TouchState::Down),
        ]));

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.50, 0.50, TouchState::Move),
                touch(2, 0.20, 0.50, TouchState::Move),
            ])),
            None
        );
    }

    #[test]
    fn recognizes_swipe_when_both_fingers_move_beyond_threshold() {
        let mut recognizer = TwoFingerSwipeRecognizer::new();

        recognizer.process(&frame(vec![
            touch(1, 0.30, 0.50, TouchState::Down),
            touch(2, 0.40, 0.50, TouchState::Down),
        ]));

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.47, 0.50, TouchState::Move),
                touch(2, 0.57, 0.50, TouchState::Move),
            ])),
            Some(TwoFingerSwipeEvent {
                direction: Direction::Right,
            })
        );
    }

    #[test]
    fn recognizes_swipe_with_asymmetric_finger_movement() {
        let mut recognizer = TwoFingerSwipeRecognizer::new();

        recognizer.process(&frame(vec![
            touch(1, 0.30, 0.50, TouchState::Down),
            touch(2, 0.40, 0.50, TouchState::Down),
        ]));

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.47, 0.50, TouchState::Move),
                touch(2, 0.58, 0.50, TouchState::Move),
            ])),
            Some(TwoFingerSwipeEvent {
                direction: Direction::Right,
            })
        );
    }

    #[test]
    fn cancelled_swipe_does_not_resume_before_release() {
        let mut recognizer = TwoFingerSwipeRecognizer::new();

        // Start first gesture.
        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.3, 0.5, TouchState::Down),
                touch(2, 0.4, 0.5, TouchState::Down),
            ])),
            None
        );

        // Vertical movement cancels the gesture.
        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.3, 0.7, TouchState::Move),
                touch(2, 0.4, 0.7, TouchState::Move),
            ])),
            None
        );

        // Horizontal movement after cancellation must NOT resume it.
        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.6, 0.7, TouchState::Move),
                touch(2, 0.7, 0.7, TouchState::Move),
            ])),
            None
        );

        // Release the cancelled gesture.
        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.6, 0.7, TouchState::Up),
                touch(2, 0.7, 0.7, TouchState::Up),
            ])),
            None
        );

        // A new gesture starts, but its initial frame must not emit.
        assert_eq!(
            recognizer.process(&frame(vec![
                touch(3, 0.3, 0.5, TouchState::Down),
                touch(4, 0.4, 0.5, TouchState::Down),
            ])),
            None
        );

        // The new gesture can now be recognized normally.
        assert_eq!(
            recognizer.process(&frame(vec![
                touch(3, 0.5, 0.5, TouchState::Move),
                touch(4, 0.6, 0.5, TouchState::Move),
            ])),
            Some(TwoFingerSwipeEvent {
                direction: Direction::Right,
            })
        );
    }
}
