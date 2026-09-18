use std::collections::HashMap;

use crate::normalize::{NormalizedPoint, NormalizedTouchFrame};
use crate::touch::TouchState;

const DEFAULT_MAX_MOVEMENT: f32 = 0.03;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TapEvent {
    pub fingers: usize,
}

#[derive(Debug)]
pub struct TapRecognizer {
    max_movement: f32,
    candidate_fingers: Option<usize>,
    start_positions: HashMap<i32, NormalizedPoint>,
    cancelled: bool,
}

impl TapRecognizer {
    pub fn new() -> Self {
        Self::with_max_movement(DEFAULT_MAX_MOVEMENT)
    }

    pub fn with_max_movement(max_movement: f32) -> Self {
        Self {
            max_movement,
            candidate_fingers: None,
            start_positions: HashMap::new(),
            cancelled: false,
        }
    }

    pub fn process(&mut self, frame: &NormalizedTouchFrame) -> Option<TapEvent> {
        let has_active_touches = frame.touches.iter().any(|touch| {
            touch.tracking_id.is_some()
                && matches!(touch.state, TouchState::Down | TouchState::Move)
        });

        if !has_active_touches && self.start_positions.is_empty() {
            self.cancelled = false;
            self.candidate_fingers = None;
        }

        for touch in &frame.touches {
            let slot = touch.slot;

            match touch.state {
                TouchState::Down => {
                    if !self.cancelled {
                        self.start_positions.entry(slot).or_insert(touch.position);
                    }
                }
                TouchState::Move => {
                    if self.cancelled {
                        continue;
                    }

                    if let Some(start) = self.start_positions.get(&slot) {
                        let movement = distance(*start, touch.position);

                        if movement > self.max_movement {
                            self.cancel();
                        }
                    }
                }
                TouchState::Up => {
                    if self.cancelled {
                        continue;
                    }

                    if let Some(start) = self.start_positions.remove(&slot) {
                        let movement = distance(start, touch.position);

                        if movement > self.max_movement {
                            self.cancel();
                        }
                    }
                }
            }
        }

        if !self.cancelled {
            self.update_candidate(frame);
        }

        if self.start_positions.is_empty() && !has_active_touches {
            if self.cancelled {
                self.cancelled = false;
                self.candidate_fingers = None;
                return None;
            }

            let fingers = self.candidate_fingers.take()?;

            if (1..=2).contains(&fingers) {
                return Some(TapEvent { fingers });
            }
        }

        None
    }

    fn update_candidate(&mut self, frame: &NormalizedTouchFrame) {
        if self.cancelled {
            return;
        }

        let started_fingers = frame
            .touches
            .iter()
            .filter(|touch| touch.state == TouchState::Down)
            .count();

        if started_fingers == 0 {
            return;
        }

        let candidate_fingers = self.candidate_fingers.unwrap_or(0);
        let total_fingers = candidate_fingers + started_fingers;

        if total_fingers > 2 {
            self.cancel();
            return;
        }

        self.candidate_fingers = Some(total_fingers);
    }

    fn cancel(&mut self) {
        self.candidate_fingers = None;
        self.start_positions.clear();
        self.cancelled = true;
    }
}

impl Default for TapRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

fn distance(a: NormalizedPoint, b: NormalizedPoint) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;

    (dx * dx + dy * dy).sqrt()
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
        NormalizedTouchFrame::new(touches)
    }

    #[test]
    fn recognizes_single_finger_tap() {
        let mut recognizer = TapRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Up),])),
            Some(TapEvent { fingers: 1 })
        );
    }

    #[test]
    fn recognizes_two_finger_tap() {
        let mut recognizer = TapRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.4, 0.5, TouchState::Down),
                touch(2, 0.6, 0.5, TouchState::Down),
            ])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.4, 0.5, TouchState::Up),
                touch(2, 0.6, 0.5, TouchState::Up),
            ])),
            Some(TapEvent { fingers: 2 })
        );
    }

    #[test]
    fn recognizes_tap_with_small_movement() {
        let mut recognizer = TapRecognizer::new();

        recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]));

        recognizer.process(&frame(vec![touch(1, 0.51, 0.505, TouchState::Move)]));

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.51, 0.505, TouchState::Up),])),
            Some(TapEvent { fingers: 1 })
        );
    }

    #[test]
    fn rejects_tap_after_excessive_movement() {
        let mut recognizer = TapRecognizer::new();

        recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down)]));

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.6, 0.5, TouchState::Move),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.6, 0.5, TouchState::Up),])),
            None
        );
    }

    #[test]
    fn rejects_three_finger_tap() {
        let mut recognizer = TapRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.3, 0.5, TouchState::Down),
                touch(2, 0.5, 0.5, TouchState::Down),
                touch(3, 0.7, 0.5, TouchState::Down),
            ])),
            None
        );
    }

    #[test]
    fn allows_second_finger_to_join_tap() {
        let mut recognizer = TapRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.5, 0.5, TouchState::Down),])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.5, 0.5, TouchState::Move),
                touch(2, 0.6, 0.5, TouchState::Down),
            ])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.5, 0.5, TouchState::Up),
                touch(2, 0.6, 0.5, TouchState::Up),
            ])),
            Some(TapEvent { fingers: 2 })
        );
    }

    #[test]
    fn rejects_changing_from_two_to_one_finger() {
        let mut recognizer = TapRecognizer::new();

        recognizer.process(&frame(vec![
            touch(1, 0.4, 0.5, TouchState::Down),
            touch(2, 0.6, 0.5, TouchState::Down),
        ]));

        assert_eq!(
            recognizer.process(&frame(vec![touch(1, 0.4, 0.5, TouchState::Up),])),
            None
        );
    }

    #[test]
    fn allows_two_finger_tap_when_fingers_release_separately() {
        let mut recognizer = TapRecognizer::new();

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.4, 0.5, TouchState::Down),
                touch(2, 0.6, 0.5, TouchState::Down),
            ])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![
                touch(1, 0.4, 0.5, TouchState::Up),
                touch(2, 0.6, 0.5, TouchState::Move),
            ])),
            None
        );

        assert_eq!(
            recognizer.process(&frame(vec![touch(2, 0.6, 0.5, TouchState::Up),])),
            Some(TapEvent { fingers: 2 })
        );
    }
}
