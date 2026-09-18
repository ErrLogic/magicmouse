use crate::gesture::Direction;
use crate::normalize::NormalizedTouchFrame;
use crate::touch::TouchState;

const DEFAULT_MOVEMENT_THRESHOLD: f32 = 0.15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OneFingerSwipeEvent {
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
pub struct OneFingerSwipeRecognizer {
    movement_threshold: f32,
    state: SwipeState,
    start_position: Option<(f32, f32)>,
}

impl OneFingerSwipeRecognizer {
    pub fn new() -> Self {
        Self {
            movement_threshold: DEFAULT_MOVEMENT_THRESHOLD,
            state: SwipeState::Idle,
            start_position: None,
        }
    }

    pub fn process(&mut self, frame: &NormalizedTouchFrame) -> Option<OneFingerSwipeEvent> {
        let active_touches = frame
            .touches
            .iter()
            .filter(|touch| {
                touch.tracking_id.is_some()
                    && matches!(touch.state, TouchState::Down | TouchState::Move)
            })
            .collect::<Vec<_>>();

        if active_touches.len() != 1 {
            self.reset_if_released(frame);
            return None;
        }

        let touch = active_touches[0];

        match self.state {
            SwipeState::Idle => {
                if touch.state == TouchState::Down {
                    self.start_position = Some((touch.position.x, touch.position.y));
                    self.state = SwipeState::Tracking;
                }

                None
            }

            SwipeState::Tracking => {
                let Some((start_x, start_y)) = self.start_position else {
                    self.cancel();
                    return None;
                };

                let dx = touch.position.x - start_x;
                let dy = touch.position.y - start_y;

                if dx.abs() < self.movement_threshold && dy.abs() < self.movement_threshold {
                    return None;
                }

                if dy.abs() >= dx.abs() {
                    self.cancel();
                    return None;
                }

                let direction = if dx > 0.0 {
                    Direction::Right
                } else {
                    Direction::Left
                };

                self.state = SwipeState::Recognized;

                Some(OneFingerSwipeEvent { direction })
            }

            SwipeState::Recognized | SwipeState::Cancelled => None,
        }
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
            self.start_position = None;
        }
    }
}

impl Default for OneFingerSwipeRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize::{NormalizedPoint, NormalizedTouch, NormalizedTouchFrame};

    fn touch(slot: i32, x: f32, y: f32, state: TouchState) -> NormalizedTouch {
        NormalizedTouch {
            slot,
            tracking_id: if state == TouchState::Up {
                None
            } else {
                Some(slot)
            },
            position: NormalizedPoint { x, y },
            state,
        }
    }

    fn frame(touches: Vec<NormalizedTouch>) -> NormalizedTouchFrame {
        NormalizedTouchFrame::new(touches)
    }

    #[test]
    fn first_touch_does_not_emit_swipe() {
        let mut recognizer = OneFingerSwipeRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );
    }

    #[test]
    fn recognizes_right_swipe() {
        let mut recognizer = OneFingerSwipeRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.7, 0.5, TouchState::Move),])),
            Some(OneFingerSwipeEvent {
                direction: Direction::Right,
            })
        );
    }

    #[test]
    fn recognizes_left_swipe() {
        let mut recognizer = OneFingerSwipeRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.3, 0.5, TouchState::Move),])),
            Some(OneFingerSwipeEvent {
                direction: Direction::Left,
            })
        );
    }

    #[test]
    fn does_not_recognize_vertical_movement_as_horizontal_swipe() {
        let mut recognizer = OneFingerSwipeRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.7, TouchState::Move),])),
            None
        );
    }

    #[test]
    fn does_not_recognize_movement_below_threshold() {
        let mut recognizer = OneFingerSwipeRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.6, 0.5, TouchState::Move),])),
            None
        );
    }

    #[test]
    fn recognizes_movement_just_above_threshold() {
        let mut recognizer = OneFingerSwipeRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );

        let threshold = DEFAULT_MOVEMENT_THRESHOLD;
        let movement = threshold + f32::EPSILON;

        assert_eq!(
            recognizer.process(&frame(vec![touch(
                1,
                0.5 + movement,
                0.5,
                TouchState::Move
            ),])),
            Some(OneFingerSwipeEvent {
                direction: Direction::Right,
            })
        );
    }

    #[test]
    fn emits_only_once_per_swipe() {
        let mut recognizer = OneFingerSwipeRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.7, 0.5, TouchState::Move),])),
            Some(OneFingerSwipeEvent {
                direction: Direction::Right,
            })
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.8, 0.5, TouchState::Move),])),
            None
        );
    }

    #[test]
    fn release_resets_swipe_lifecycle() {
        let mut recognizer = OneFingerSwipeRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.7, 0.5, TouchState::Move),])),
            Some(OneFingerSwipeEvent {
                direction: Direction::Right,
            })
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.7, 0.5, TouchState::Up),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.3, 0.5, TouchState::Move),])),
            Some(OneFingerSwipeEvent {
                direction: Direction::Left,
            })
        );
    }

    #[test]
    fn multiple_fingers_cancel_swipe() {
        let mut recognizer = OneFingerSwipeRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.6, 0.5, TouchState::Move),
                touch(2, 0.6, 0.5, TouchState::Down),
            ])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.8, 0.5, TouchState::Move),
                touch(2, 0.6, 0.5, TouchState::Move),
            ])),
            None
        );
    }

    #[test]
    fn cancelled_swipe_does_not_resume_before_release() {
        let mut recognizer = OneFingerSwipeRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.7, TouchState::Move),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.8, 0.5, TouchState::Move),])),
            None
        );
    }

    #[test]
    fn tracking_id_change_does_not_create_new_swipe() {
        let mut recognizer = OneFingerSwipeRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.7, 0.5, TouchState::Move),])),
            Some(OneFingerSwipeEvent {
                direction: Direction::Right,
            })
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.8, 0.5, TouchState::Move),])),
            None
        );
    }

    #[test]
    fn equal_movement_prefers_vertical_axis() {
        let mut recognizer = OneFingerSwipeRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.7, 0.7, TouchState::Move),])),
            None
        );
    }

    #[test]
    fn new_swipe_can_start_after_cancelled_gesture() {
        let mut recognizer = OneFingerSwipeRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.7, TouchState::Move),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.7, TouchState::Up),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.3, 0.5, TouchState::Move),])),
            Some(OneFingerSwipeEvent {
                direction: Direction::Left,
            })
        );
    }
}
