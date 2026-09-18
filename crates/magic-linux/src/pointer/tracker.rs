use crate::normalize::{NormalizedPoint, NormalizedTouchFrame};

use super::{PointerEvent, PointerMovement, PointerSensitivity};

#[derive(Debug, Clone, Copy)]
pub struct PointerTracker {
    sensitivity: PointerSensitivity,
    previous: Option<(i32, NormalizedPoint)>,
}

impl PointerTracker {
    pub fn new(sensitivity: PointerSensitivity) -> Self {
        Self {
            sensitivity,
            previous: None,
        }
    }

    pub fn reset(&mut self) {
        self.previous = None;
    }

    pub fn process(&mut self, frame: &NormalizedTouchFrame) -> Option<PointerEvent> {
        if frame.len() != 1 {
            self.reset();
            return None;
        }

        let touch = frame.touches[0];

        if !touch.state.is_active() {
            self.reset();
            return None;
        }

        let current = touch.position;

        let tracking_id = touch.tracking_id?;
        let previous = self.previous.replace((tracking_id, current));

        let (previous_tracking_id, previous) = previous?;

        if previous_tracking_id != tracking_id {
            return None;
        }

        let delta = PointerMovement::from_points(previous, current);
        let delta = self.sensitivity.apply(delta);

        let event = PointerEvent::new(delta);

        if event.is_zero() {
            return None;
        }

        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize::{NormalizedTouch, NormalizedTouchFrame};
    use crate::touch::TouchState;

    fn point(x: f32, y: f32) -> NormalizedPoint {
        NormalizedPoint { x, y }
    }

    fn frame(x: f32, y: f32, state: TouchState) -> NormalizedTouchFrame {
        NormalizedTouchFrame::new(vec![crate::normalize::NormalizedTouch {
            slot: 0,
            tracking_id: Some(1),
            position: point(x, y),
            state,
        }])
    }

    #[test]
    fn first_touch_does_not_emit_pointer_event() {
        let sensitivity = PointerSensitivity::new(1.0);
        let mut tracker = PointerTracker::new(sensitivity);

        let result = tracker.process(&frame(0.50, 0.50, TouchState::Down));

        assert!(result.is_none());
    }

    #[test]
    fn movement_emits_pointer_event() {
        let sensitivity = PointerSensitivity::new(1.0);
        let mut tracker = PointerTracker::new(sensitivity);

        tracker.process(&frame(0.50, 0.50, TouchState::Down));

        let event = tracker
            .process(&frame(0.60, 0.45, TouchState::Move))
            .expect("expected pointer event");

        assert!((event.delta.dx - 0.10).abs() < 0.0001);
        assert!((event.delta.dy - (-0.05)).abs() < 0.0001);
    }

    #[test]
    fn applies_pointer_sensitivity() {
        let sensitivity = PointerSensitivity::new(2.0);
        let mut tracker = PointerTracker::new(sensitivity);

        tracker.process(&frame(0.50, 0.50, TouchState::Down));

        let event = tracker
            .process(&frame(0.60, 0.45, TouchState::Move))
            .expect("expected pointer event");

        assert!((event.delta.dx - 0.20).abs() < 0.0001);
        assert!((event.delta.dy - (-0.10)).abs() < 0.0001);
    }

    #[test]
    fn multiple_fingers_disable_pointer_tracking() {
        let sensitivity = PointerSensitivity::new(1.0);
        let mut tracker = PointerTracker::new(sensitivity);

        tracker.process(&frame(0.50, 0.50, TouchState::Down));

        let multi_frame = NormalizedTouchFrame::new(vec![
            crate::normalize::NormalizedTouch {
                slot: 0,
                tracking_id: Some(1),
                position: point(0.60, 0.45),
                state: TouchState::Move,
            },
            crate::normalize::NormalizedTouch {
                slot: 1,
                tracking_id: Some(2),
                position: point(0.40, 0.45),
                state: TouchState::Down,
            },
        ]);

        assert!(tracker.process(&multi_frame).is_none());
    }

    #[test]
    fn pointer_tracking_resumes_without_position_jump() {
        let sensitivity = PointerSensitivity::new(1.0);
        let mut tracker = PointerTracker::new(sensitivity);

        tracker.process(&frame(0.50, 0.50, TouchState::Down));

        let multi_frame = NormalizedTouchFrame::new(vec![
            crate::normalize::NormalizedTouch {
                slot: 0,
                tracking_id: Some(1),
                position: point(0.70, 0.70),
                state: TouchState::Move,
            },
            crate::normalize::NormalizedTouch {
                slot: 1,
                tracking_id: Some(2),
                position: point(0.30, 0.30),
                state: TouchState::Down,
            },
        ]);

        assert!(tracker.process(&multi_frame).is_none());

        let result = tracker.process(&frame(0.80, 0.80, TouchState::Move));

        assert!(result.is_none());
    }

    #[test]
    fn finger_release_resets_pointer_tracking() {
        let sensitivity = PointerSensitivity::new(1.0);
        let mut tracker = PointerTracker::new(sensitivity);

        tracker.process(&frame(0.50, 0.50, TouchState::Down));

        let release = frame(0.60, 0.60, TouchState::Up);

        assert!(tracker.process(&release).is_none());

        let result = tracker.process(&frame(0.80, 0.80, TouchState::Down));

        assert!(result.is_none());
    }

    #[test]
    fn zero_movement_does_not_emit_pointer_event() {
        let mut tracker = PointerTracker::new(PointerSensitivity::new(1.0));

        tracker.process(&frame(0.50, 0.50, TouchState::Down));

        let result = tracker.process(&frame(0.50, 0.50, TouchState::Move));

        assert!(result.is_none());
    }

    #[test]
    fn tracking_id_change_does_not_emit_position_jump() {
        let mut tracker = PointerTracker::new(PointerSensitivity::new(1.0));

        let first = NormalizedTouchFrame::new(vec![NormalizedTouch {
            slot: 0,
            tracking_id: Some(10),
            position: NormalizedPoint { x: 0.30, y: 0.40 },
            state: TouchState::Down,
        }]);

        assert!(tracker.process(&first).is_none());

        let movement = NormalizedTouchFrame::new(vec![NormalizedTouch {
            slot: 0,
            tracking_id: Some(10),
            position: NormalizedPoint { x: 0.40, y: 0.40 },
            state: TouchState::Move,
        }]);

        let event = tracker
            .process(&movement)
            .expect("expected pointer movement");

        assert!((event.delta.dx - 0.10).abs() < 0.0001);

        let new_finger = NormalizedTouchFrame::new(vec![NormalizedTouch {
            slot: 0,
            tracking_id: Some(11),
            position: NormalizedPoint { x: 0.80, y: 0.40 },
            state: TouchState::Down,
        }]);

        assert!(tracker.process(&new_finger).is_none());

        let new_finger_movement = NormalizedTouchFrame::new(vec![NormalizedTouch {
            slot: 0,
            tracking_id: Some(11),
            position: NormalizedPoint { x: 0.82, y: 0.40 },
            state: TouchState::Move,
        }]);

        let event = tracker
            .process(&new_finger_movement)
            .expect("expected pointer movement from new finger");

        assert!((event.delta.dx - 0.02).abs() < 0.0001);
    }
}
