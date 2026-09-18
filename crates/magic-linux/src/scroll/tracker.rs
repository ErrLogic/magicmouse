use crate::normalize::{NormalizedPoint, NormalizedTouchFrame};

use super::{ScrollEvent, ScrollMovement, ScrollSensitivity};

#[derive(Debug, Clone, Copy)]
pub struct ScrollTracker {
    sensitivity: ScrollSensitivity,
    previous: Option<(NormalizedPoint, NormalizedPoint)>,
}

impl ScrollTracker {
    pub fn new(sensitivity: ScrollSensitivity) -> Self {
        Self {
            sensitivity,
            previous: None,
        }
    }

    pub fn reset(&mut self) {
        self.previous = None;
    }

    pub fn process(&mut self, frame: &NormalizedTouchFrame) -> Option<ScrollEvent> {
        if frame.len() != 2 {
            self.reset();
            return None;
        }

        if frame.touches.iter().any(|touch| !touch.state.is_active()) {
            self.reset();
            return None;
        }

        let first = frame.touches[0].position;
        let second = frame.touches[1].position;

        let previous = self.previous.replace((first, second))?;

        let previous_center = Self::centroid(previous.0, previous.1);
        let current_center = Self::centroid(first, second);

        let delta = ScrollMovement::from_points(previous_center, current_center);
        let delta = self.sensitivity.apply(delta);

        let event = ScrollEvent::new(delta);

        if event.is_zero() {
            return None;
        }

        Some(event)
    }

    fn centroid(first: NormalizedPoint, second: NormalizedPoint) -> NormalizedPoint {
        NormalizedPoint {
            x: (first.x + second.x) / 2.0,
            y: (first.y + second.y) / 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize::{NormalizedTouch, NormalizedTouchFrame};
    use crate::touch::TouchState;

    fn frame(first: (f32, f32), second: (f32, f32), state: TouchState) -> NormalizedTouchFrame {
        NormalizedTouchFrame::new(vec![
            NormalizedTouch {
                slot: 0,
                tracking_id: Some(1),
                position: NormalizedPoint {
                    x: first.0,
                    y: first.1,
                },
                state,
            },
            NormalizedTouch {
                slot: 1,
                tracking_id: Some(2),
                position: NormalizedPoint {
                    x: second.0,
                    y: second.1,
                },
                state,
            },
        ])
    }

    #[test]
    fn first_two_finger_frame_does_not_emit_scroll_event() {
        let mut tracker = ScrollTracker::new(ScrollSensitivity::new(1.0));

        let result = tracker.process(&frame((0.30, 0.40), (0.50, 0.40), TouchState::Down));

        assert!(result.is_none());
    }

    #[test]
    fn two_finger_movement_emits_scroll_event() {
        let mut tracker = ScrollTracker::new(ScrollSensitivity::new(1.0));

        tracker.process(&frame((0.30, 0.40), (0.50, 0.40), TouchState::Down));

        let result = tracker.process(&frame((0.32, 0.44), (0.52, 0.44), TouchState::Move));

        let event = result.expect("expected scroll event");

        assert!((event.delta.dx - 0.02).abs() < 0.0001);
        assert!((event.delta.dy - 0.04).abs() < 0.0001);
    }

    #[test]
    fn applies_scroll_sensitivity() {
        let mut tracker = ScrollTracker::new(ScrollSensitivity::new(2.0));

        tracker.process(&frame((0.30, 0.40), (0.50, 0.40), TouchState::Down));

        let result = tracker.process(&frame((0.32, 0.44), (0.52, 0.44), TouchState::Move));

        let event = result.expect("expected scroll event");

        assert!((event.delta.dx - 0.04).abs() < 0.0001);
        assert!((event.delta.dy - 0.08).abs() < 0.0001);
    }

    #[test]
    fn non_two_finger_frame_disables_scroll_tracking() {
        let mut tracker = ScrollTracker::new(ScrollSensitivity::new(1.0));

        tracker.process(&frame((0.30, 0.40), (0.50, 0.40), TouchState::Down));

        let one_finger = NormalizedTouchFrame::new(vec![NormalizedTouch {
            slot: 0,
            tracking_id: Some(1),
            position: NormalizedPoint { x: 0.40, y: 0.40 },
            state: TouchState::Move,
        }]);

        assert!(tracker.process(&one_finger).is_none());

        let result = tracker.process(&frame((0.40, 0.45), (0.60, 0.45), TouchState::Move));

        assert!(result.is_none());
    }

    #[test]
    fn zero_movement_does_not_emit_scroll_event() {
        let mut tracker = ScrollTracker::new(ScrollSensitivity::new(1.0));

        tracker.process(&frame((0.30, 0.40), (0.50, 0.40), TouchState::Down));

        let result = tracker.process(&frame((0.30, 0.40), (0.50, 0.40), TouchState::Move));

        assert!(result.is_none());
    }
}
