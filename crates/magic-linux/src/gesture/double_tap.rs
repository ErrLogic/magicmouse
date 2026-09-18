use std::time::{Duration, Instant};

use crate::gesture::TapRecognizer;
use crate::normalize::NormalizedTouchFrame;

const DEFAULT_MAX_INTERVAL: Duration = Duration::from_millis(300);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoubleTapEvent {
    pub fingers: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FirstTap {
    fingers: usize,
    completed_at: Instant,
}

#[derive(Debug)]
pub struct DoubleTapRecognizer {
    max_interval: Duration,
    first_tap: Option<FirstTap>,
    tap_recognizer: TapRecognizer,
}

impl DoubleTapRecognizer {
    pub fn new() -> Self {
        Self {
            max_interval: DEFAULT_MAX_INTERVAL,
            first_tap: None,
            tap_recognizer: TapRecognizer::new(),
        }
    }

    pub fn process(
        &mut self,
        frame: &NormalizedTouchFrame,
        timestamp: Instant,
    ) -> Option<DoubleTapEvent> {
        let tap = self.tap_recognizer.process(frame)?;

        let Some(first_tap) = self.first_tap else {
            self.first_tap = Some(FirstTap {
                fingers: tap.fingers,
                completed_at: timestamp,
            });

            return None;
        };

        let interval = timestamp.saturating_duration_since(first_tap.completed_at);

        self.first_tap = None;

        if interval > self.max_interval {
            self.first_tap = Some(FirstTap {
                fingers: tap.fingers,
                completed_at: timestamp,
            });

            return None;
        }

        if tap.fingers != first_tap.fingers {
            return None;
        }

        Some(DoubleTapEvent {
            fingers: tap.fingers,
        })
    }
}

impl Default for DoubleTapRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize::{NormalizedPoint, NormalizedTouch, NormalizedTouchFrame};
    use crate::touch::TouchState;

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
    fn first_tap_does_not_emit_double_tap() {
        let mut recognizer = DoubleTapRecognizer::new();
        let timestamp = Instant::now();

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]),
                timestamp,
            ),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Up)]), timestamp,),
            None
        );
    }

    #[test]
    fn recognizes_single_finger_double_tap() {
        let mut recognizer = DoubleTapRecognizer::new();
        let start = Instant::now();

        // First tap
        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]), start,),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Up)]),
                start + Duration::from_millis(50),
            ),
            None
        );

        // Second tap
        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]),
                start + Duration::from_millis(100),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Up)]),
                start + Duration::from_millis(150),
            ),
            Some(DoubleTapEvent { fingers: 1 })
        );
    }

    #[test]
    fn recognizes_two_finger_double_tap() {
        let mut recognizer = DoubleTapRecognizer::new();
        let start = Instant::now();

        // First tap
        assert_eq!(
            recognizer.process(
                &frame(vec![
                    touch(1, 0.4, 0.5, TouchState::Down),
                    touch(2, 0.6, 0.5, TouchState::Down),
                ]),
                start,
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![
                    touch(1, 0.4, 0.5, TouchState::Up),
                    touch(2, 0.6, 0.5, TouchState::Move),
                ]),
                start + Duration::from_millis(30),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(2, 0.6, 0.5, TouchState::Up),]),
                start + Duration::from_millis(50),
            ),
            None
        );

        // Second tap
        assert_eq!(
            recognizer.process(
                &frame(vec![
                    touch(1, 0.4, 0.5, TouchState::Down),
                    touch(2, 0.6, 0.5, TouchState::Down),
                ]),
                start + Duration::from_millis(100),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![
                    touch(1, 0.4, 0.5, TouchState::Up),
                    touch(2, 0.6, 0.5, TouchState::Move),
                ]),
                start + Duration::from_millis(130),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(2, 0.6, 0.5, TouchState::Up),]),
                start + Duration::from_millis(150),
            ),
            Some(DoubleTapEvent { fingers: 2 })
        );
    }

    #[test]
    fn rejects_double_tap_after_timeout() {
        let mut recognizer = DoubleTapRecognizer::new();
        let start = Instant::now();

        // First tap
        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]), start,),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Up)]),
                start + Duration::from_millis(50),
            ),
            None
        );

        // Second tap happens after the 300ms window.
        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]),
                start + Duration::from_millis(400),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Up)]),
                start + Duration::from_millis(450),
            ),
            None
        );
    }

    #[test]
    fn rejects_double_tap_with_different_finger_count() {
        let mut recognizer = DoubleTapRecognizer::new();
        let start = Instant::now();

        // First tap: 1 finger
        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]), start,),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Up)]),
                start + Duration::from_millis(50),
            ),
            None
        );

        // Second tap: 2 fingers
        assert_eq!(
            recognizer.process(
                &frame(vec![
                    touch(1, 0.4, 0.5, TouchState::Down),
                    touch(2, 0.6, 0.5, TouchState::Down),
                ]),
                start + Duration::from_millis(100),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![
                    touch(1, 0.4, 0.5, TouchState::Up),
                    touch(2, 0.6, 0.5, TouchState::Move),
                ]),
                start + Duration::from_millis(130),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(2, 0.6, 0.5, TouchState::Up)]),
                start + Duration::from_millis(150),
            ),
            None
        );
    }

    #[test]
    fn rejects_double_tap_with_reduced_finger_count() {
        let mut recognizer = DoubleTapRecognizer::new();
        let start = Instant::now();

        // First tap: 2 fingers
        assert_eq!(
            recognizer.process(
                &frame(vec![
                    touch(1, 0.4, 0.5, TouchState::Down),
                    touch(2, 0.6, 0.5, TouchState::Down),
                ]),
                start,
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![
                    touch(1, 0.4, 0.5, TouchState::Up),
                    touch(2, 0.6, 0.5, TouchState::Move),
                ]),
                start + Duration::from_millis(30),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(2, 0.6, 0.5, TouchState::Up)]),
                start + Duration::from_millis(50),
            ),
            None
        );

        // Second tap: 1 finger
        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]),
                start + Duration::from_millis(100),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Up)]),
                start + Duration::from_millis(150),
            ),
            None
        );
    }

    #[test]
    fn starts_new_double_tap_sequence_after_success() {
        let mut recognizer = DoubleTapRecognizer::new();
        let start = Instant::now();

        // First tap
        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]), start,),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Up)]),
                start + Duration::from_millis(50),
            ),
            None
        );

        // Second tap → completes double tap
        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]),
                start + Duration::from_millis(100),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Up)]),
                start + Duration::from_millis(150),
            ),
            Some(DoubleTapEvent { fingers: 1 })
        );

        // Third tap → should become the first tap of a new sequence.
        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]),
                start + Duration::from_millis(200),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Up)]),
                start + Duration::from_millis(250),
            ),
            None
        );
    }

    #[test]
    fn accepts_double_tap_at_exact_interval_boundary() {
        let mut recognizer = DoubleTapRecognizer::new();
        let start = Instant::now();

        // First tap
        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]), start,),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Up)]),
                start + Duration::from_millis(50),
            ),
            None
        );

        // Second tap completes exactly 300ms after first tap.
        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]),
                start + Duration::from_millis(300),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Up)]),
                start + Duration::from_millis(350),
            ),
            Some(DoubleTapEvent { fingers: 1 })
        );
    }

    #[test]
    fn rejects_double_tap_just_after_interval_boundary() {
        let mut recognizer = DoubleTapRecognizer::new();
        let start = Instant::now();

        // First tap
        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]), start,),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Up)]),
                start + Duration::from_millis(50),
            ),
            None
        );

        // Second tap completes 301ms after first tap.
        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]),
                start + Duration::from_millis(301),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Up)]),
                start + Duration::from_millis(351),
            ),
            None
        );
    }

    #[test]
    fn rejects_double_tap_when_second_tap_moves_too_far() {
        let mut recognizer = DoubleTapRecognizer::new();
        let start = Instant::now();

        // First tap
        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]), start,),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Up)]),
                start + Duration::from_millis(50),
            ),
            None
        );

        // Second tap starts.
        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]),
                start + Duration::from_millis(100),
            ),
            None
        );

        // Movement exceeds the tap threshold.
        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.7, 0.5, TouchState::Move)]),
                start + Duration::from_millis(120),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.7, 0.5, TouchState::Up)]),
                start + Duration::from_millis(150),
            ),
            None
        );
    }

    #[test]
    fn rejects_double_tap_when_first_tap_moves_too_far() {
        let mut recognizer = DoubleTapRecognizer::new();
        let start = Instant::now();

        // First tap starts.
        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]), start,),
            None
        );

        // Excessive movement invalidates the first tap.
        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.7, 0.5, TouchState::Move)]),
                start + Duration::from_millis(20),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.7, 0.5, TouchState::Up)]),
                start + Duration::from_millis(50),
            ),
            None
        );

        // A later tap must not accidentally complete a double tap.
        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.7, 0.5, TouchState::Down)]),
                start + Duration::from_millis(100),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![touch(1, 0.7, 0.5, TouchState::Up)]),
                start + Duration::from_millis(150),
            ),
            None
        );
    }

    #[test]
    fn rejects_three_finger_double_tap() {
        let mut recognizer = DoubleTapRecognizer::new();
        let start = Instant::now();

        assert_eq!(
            recognizer.process(
                &frame(vec![
                    touch(1, 0.3, 0.5, TouchState::Down),
                    touch(2, 0.5, 0.5, TouchState::Down),
                    touch(3, 0.7, 0.5, TouchState::Down),
                ]),
                start,
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![
                    touch(1, 0.3, 0.5, TouchState::Up),
                    touch(2, 0.5, 0.5, TouchState::Move),
                    touch(3, 0.7, 0.5, TouchState::Move),
                ]),
                start + Duration::from_millis(30),
            ),
            None
        );

        assert_eq!(
            recognizer.process(
                &frame(vec![
                    touch(2, 0.5, 0.5, TouchState::Up),
                    touch(3, 0.7, 0.5, TouchState::Up),
                ]),
                start + Duration::from_millis(50),
            ),
            None
        );
    }
}
