use crate::axis::TouchBounds;
use crate::input::RawInputEvent;
use crate::normalize::NormalizedTouchFrame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchState {
    Down,
    Move,
    Up,
}

impl TouchState {
    pub fn is_active(self) -> bool {
        matches!(self, Self::Down | Self::Move)
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Up)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Touch {
    pub slot: i32,
    pub tracking_id: Option<i32>,
    pub x: i32,
    pub y: i32,
    pub state: TouchState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchSlot {
    pub tracking_id: Option<i32>,
    pub x: i32,
    pub y: i32,
    pending_state: Option<TouchState>,
}

impl TouchSlot {
    pub fn empty() -> Self {
        Self {
            tracking_id: None,
            x: 0,
            y: 0,
            pending_state: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.tracking_id.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TouchFrame {
    pub touches: Vec<Touch>,
}

impl TouchFrame {
    pub fn new(touches: Vec<Touch>) -> Self {
        Self { touches }
    }

    pub fn is_empty(&self) -> bool {
        self.touches.is_empty()
    }

    pub fn len(&self) -> usize {
        self.touches.len()
    }

    pub fn normalize(&self, bounds: &TouchBounds) -> NormalizedTouchFrame {
        let touches = self
            .touches
            .iter()
            .map(|touch| touch.normalize(bounds))
            .collect();

        NormalizedTouchFrame::new(touches)
    }
}

pub struct TouchTracker {
    slots: [TouchSlot; 16],
    current_slot: usize,
}

impl TouchTracker {
    pub fn new() -> Self {
        Self {
            slots: [TouchSlot::empty(); 16],
            current_slot: 0,
        }
    }

    pub fn active_touch_count(&self) -> usize {
        self.slots.iter().filter(|slot| slot.is_active()).count()
    }

    pub fn current_slot(&self) -> usize {
        self.current_slot
    }

    pub fn slot(&self, index: usize) -> Option<&TouchSlot> {
        self.slots.get(index)
    }

    pub fn process_event(&mut self, event: RawInputEvent) -> Option<TouchFrame> {
        match (event.event_type, event.event_code) {
            (3, 47) => {
                self.set_current_slot(event.value);
                None
            }

            (3, 57) => {
                self.update_tracking_id(event.value);
                None
            }

            (3, 53) => {
                self.update_position_x(event.value);
                None
            }

            (3, 54) => {
                self.update_position_y(event.value);
                None
            }

            (0, 0) => self.commit_frame(),

            _ => None,
        }
    }

    fn set_current_slot(&mut self, slot: i32) {
        if let Ok(slot) = usize::try_from(slot)
            && slot < self.slots.len()
        {
            self.current_slot = slot;
        }
    }

    fn update_tracking_id(&mut self, tracking_id: i32) {
        let slot = &mut self.slots[self.current_slot];

        match tracking_id {
            -1 => {
                if slot.tracking_id.is_some() {
                    slot.pending_state = Some(TouchState::Up);
                }

                slot.tracking_id = None;
            }

            tracking_id => {
                slot.tracking_id = Some(tracking_id);
                slot.pending_state = Some(TouchState::Down);
            }
        }
    }

    fn update_position_x(&mut self, x: i32) {
        let slot = &mut self.slots[self.current_slot];

        slot.x = x;

        if slot.tracking_id.is_some() && slot.pending_state.is_none() {
            slot.pending_state = Some(TouchState::Move);
        }
    }

    fn update_position_y(&mut self, y: i32) {
        let slot = &mut self.slots[self.current_slot];

        slot.y = y;

        if slot.tracking_id.is_some() && slot.pending_state.is_none() {
            slot.pending_state = Some(TouchState::Move);
        }
    }

    fn commit_frame(&mut self) -> Option<TouchFrame> {
        let touches = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(slot, state)| {
                let touch_state = match state.pending_state {
                    Some(state) => state,
                    None if state.tracking_id.is_some() => TouchState::Move,
                    None => return None,
                };

                Some(Touch {
                    slot: slot as i32,
                    tracking_id: state.tracking_id,
                    x: state.x,
                    y: state.y,
                    state: touch_state,
                })
            })
            .collect::<Vec<_>>();

        for slot in &mut self.slots {
            slot.pending_state = None;
        }

        if touches.is_empty() {
            None
        } else {
            Some(TouchFrame::new(touches))
        }
    }
}

impl Default for TouchTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis::{AxisBounds, TouchBounds};
    use crate::input::RawInputEvent;

    fn event(event_type: u16, event_code: u16, value: i32) -> RawInputEvent {
        RawInputEvent {
            event_type,
            event_code,
            value,
        }
    }

    #[test]
    fn parses_single_touch_frame() {
        let mut tracker = TouchTracker::new();

        assert!(tracker.process_event(event(3, 47, 3)).is_none());
        assert!(tracker.process_event(event(3, 57, 35)).is_none());
        assert!(tracker.process_event(event(3, 53, 100)).is_none());
        assert!(tracker.process_event(event(3, 54, 500)).is_none());

        let frame = tracker
            .process_event(event(0, 0, 0))
            .expect("expected touch frame");

        assert_eq!(frame.len(), 1);

        let touch = frame.touches[0];

        assert_eq!(touch.slot, 3);
        assert_eq!(touch.tracking_id, Some(35));
        assert_eq!(touch.x, 100);
        assert_eq!(touch.y, 500);
        assert_eq!(touch.state, TouchState::Down);
    }

    #[test]
    fn tracks_multiple_slots() {
        let mut tracker = TouchTracker::new();

        // Finger 1
        tracker.process_event(event(3, 47, 2));
        tracker.process_event(event(3, 57, 41));
        tracker.process_event(event(3, 53, 100));
        tracker.process_event(event(3, 54, 200));

        // Finger 2
        tracker.process_event(event(3, 47, 8));
        tracker.process_event(event(3, 57, 42));
        tracker.process_event(event(3, 53, 300));
        tracker.process_event(event(3, 54, 400));

        let frame = tracker
            .process_event(event(0, 0, 0))
            .expect("expected touch frame");

        assert_eq!(frame.len(), 2);
        assert_eq!(tracker.active_touch_count(), 2);

        let slot_2 = frame
            .touches
            .iter()
            .find(|touch| touch.slot == 2)
            .expect("slot 2 missing");

        assert_eq!(slot_2.tracking_id, Some(41));
        assert_eq!(slot_2.x, 100);
        assert_eq!(slot_2.y, 200);

        let slot_8 = frame
            .touches
            .iter()
            .find(|touch| touch.slot == 8)
            .expect("slot 8 missing");

        assert_eq!(slot_8.tracking_id, Some(42));
        assert_eq!(slot_8.x, 300);
        assert_eq!(slot_8.y, 400);
    }

    #[test]
    fn ignores_unknown_events() {
        let mut tracker = TouchTracker::new();

        assert!(tracker.process_event(event(1, 272, 1)).is_none());
        assert!(tracker.process_event(event(2, 0, 10)).is_none());

        assert_eq!(tracker.active_touch_count(), 0);
    }

    #[test]
    fn tracks_touch_lifecycle() {
        let mut tracker = TouchTracker::new();

        // Finger appears.
        tracker.process_event(event(3, 47, 3));
        tracker.process_event(event(3, 57, 35));
        tracker.process_event(event(3, 53, 100));
        tracker.process_event(event(3, 54, 500));

        let frame = tracker
            .process_event(event(0, 0, 0))
            .expect("expected DOWN frame");

        assert_eq!(frame.len(), 1);

        let touch = frame.touches[0];

        assert_eq!(touch.slot, 3);
        assert_eq!(touch.tracking_id, Some(35));
        assert_eq!(touch.x, 100);
        assert_eq!(touch.y, 500);
        assert_eq!(touch.state, TouchState::Down);

        // Finger moves.
        tracker.process_event(event(3, 53, 120));
        tracker.process_event(event(3, 54, 480));

        let frame = tracker
            .process_event(event(0, 0, 0))
            .expect("expected MOVE frame");

        let touch = frame.touches[0];

        assert_eq!(touch.tracking_id, Some(35));
        assert_eq!(touch.x, 120);
        assert_eq!(touch.y, 480);
        assert_eq!(touch.state, TouchState::Move);

        // Finger releases.
        tracker.process_event(event(3, 57, -1));

        let frame = tracker
            .process_event(event(0, 0, 0))
            .expect("expected UP frame");

        let touch = frame.touches[0];

        assert_eq!(touch.slot, 3);
        assert_eq!(touch.tracking_id, None);
        assert_eq!(touch.x, 120);
        assert_eq!(touch.y, 480);
        assert_eq!(touch.state, TouchState::Up);

        assert_eq!(tracker.active_touch_count(), 0);
    }

    #[test]
    fn tracks_multiple_touch_lifecycles_independently() {
        let mut tracker = TouchTracker::new();

        // Finger A enters slot 2.
        tracker.process_event(event(3, 47, 2));
        tracker.process_event(event(3, 57, 41));
        tracker.process_event(event(3, 53, 100));
        tracker.process_event(event(3, 54, 200));

        // Finger B enters slot 8.
        tracker.process_event(event(3, 47, 8));
        tracker.process_event(event(3, 57, 42));
        tracker.process_event(event(3, 53, 300));
        tracker.process_event(event(3, 54, 400));

        let frame = tracker
            .process_event(event(0, 0, 0))
            .expect("expected DOWN frame");

        assert_eq!(frame.len(), 2);
        assert_eq!(tracker.active_touch_count(), 2);

        let finger_a = frame
            .touches
            .iter()
            .find(|touch| touch.slot == 2)
            .expect("finger A missing");

        assert_eq!(finger_a.tracking_id, Some(41));
        assert_eq!(finger_a.x, 100);
        assert_eq!(finger_a.y, 200);
        assert_eq!(finger_a.state, TouchState::Down);

        let finger_b = frame
            .touches
            .iter()
            .find(|touch| touch.slot == 8)
            .expect("finger B missing");

        assert_eq!(finger_b.tracking_id, Some(42));
        assert_eq!(finger_b.x, 300);
        assert_eq!(finger_b.y, 400);
        assert_eq!(finger_b.state, TouchState::Down);

        // Move only finger A.
        tracker.process_event(event(3, 47, 2));
        tracker.process_event(event(3, 53, 120));
        tracker.process_event(event(3, 54, 180));

        let frame = tracker
            .process_event(event(0, 0, 0))
            .expect("expected MOVE frame");

        let finger_a = frame
            .touches
            .iter()
            .find(|touch| touch.slot == 2)
            .expect("finger A missing");

        assert_eq!(finger_a.tracking_id, Some(41));
        assert_eq!(finger_a.x, 120);
        assert_eq!(finger_a.y, 180);
        assert_eq!(finger_a.state, TouchState::Move);

        // Finger B remains active with its previous position.
        let finger_b = frame
            .touches
            .iter()
            .find(|touch| touch.slot == 8)
            .expect("finger B missing");

        assert_eq!(finger_b.tracking_id, Some(42));
        assert_eq!(finger_b.x, 300);
        assert_eq!(finger_b.y, 400);

        // Release only finger A.
        tracker.process_event(event(3, 47, 2));
        tracker.process_event(event(3, 57, -1));

        let frame = tracker
            .process_event(event(0, 0, 0))
            .expect("expected UP frame");

        let finger_a = frame
            .touches
            .iter()
            .find(|touch| touch.slot == 2)
            .expect("finger A missing");

        assert_eq!(finger_a.tracking_id, None);
        assert_eq!(finger_a.x, 120);
        assert_eq!(finger_a.y, 180);
        assert_eq!(finger_a.state, TouchState::Up);

        assert_eq!(tracker.active_touch_count(), 1);

        // Finger B is still active.
        assert_eq!(tracker.slot(8).and_then(|slot| slot.tracking_id), Some(42));
    }

    #[test]
    fn emits_frame_only_at_syn_report() {
        let mut tracker = TouchTracker::new();

        // No frame before SYN_REPORT.
        assert!(tracker.process_event(event(3, 47, 3)).is_none());
        assert!(tracker.process_event(event(3, 57, 35)).is_none());
        assert!(tracker.process_event(event(3, 53, 100)).is_none());
        assert!(tracker.process_event(event(3, 54, 500)).is_none());

        // Frame is committed only at SYN_REPORT.
        let frame = tracker
            .process_event(event(0, 0, 0))
            .expect("expected frame at SYN_REPORT");

        assert_eq!(frame.len(), 1);

        let touch = frame.touches[0];

        assert_eq!(touch.slot, 3);
        assert_eq!(touch.tracking_id, Some(35));
        assert_eq!(touch.x, 100);
        assert_eq!(touch.y, 500);
        assert_eq!(touch.state, TouchState::Down);
    }

    #[test]
    fn ignores_move_without_active_touch() {
        let mut tracker = TouchTracker::new();

        tracker.process_event(event(3, 47, 3));
        tracker.process_event(event(3, 53, 100));
        tracker.process_event(event(3, 54, 500));

        assert!(tracker.process_event(event(0, 0, 0)).is_none());

        assert_eq!(tracker.active_touch_count(), 0);
    }

    #[test]
    fn ignores_release_without_active_touch() {
        let mut tracker = TouchTracker::new();

        tracker.process_event(event(3, 47, 3));
        tracker.process_event(event(3, 57, -1));

        assert!(tracker.process_event(event(0, 0, 0)).is_none());

        assert_eq!(tracker.active_touch_count(), 0);
    }

    #[test]
    fn new_tracking_id_is_down() {
        let mut tracker = TouchTracker::new();

        // First finger.
        tracker.process_event(event(3, 47, 3));
        tracker.process_event(event(3, 57, 35));
        tracker.process_event(event(3, 53, 100));
        tracker.process_event(event(3, 54, 500));

        let frame = tracker
            .process_event(event(0, 0, 0))
            .expect("expected first DOWN");

        assert_eq!(frame.touches[0].state, TouchState::Down);

        // Same slot receives a new tracking ID.
        tracker.process_event(event(3, 57, 36));
        tracker.process_event(event(3, 53, 200));
        tracker.process_event(event(3, 54, 600));

        let frame = tracker
            .process_event(event(0, 0, 0))
            .expect("expected second DOWN");

        let touch = frame
            .touches
            .iter()
            .find(|touch| touch.slot == 3)
            .expect("slot 3 missing");

        assert_eq!(touch.tracking_id, Some(36));
        assert_eq!(touch.x, 200);
        assert_eq!(touch.y, 600);
        assert_eq!(touch.state, TouchState::Down);
    }

    #[test]
    fn normalizes_multiple_touches_independently() {
        let bounds = TouchBounds::new(AxisBounds::new(-100, 100), AxisBounds::new(-200, 200));

        let frame = TouchFrame::new(vec![
            Touch {
                slot: 2,
                tracking_id: Some(10),
                x: -100,
                y: -200,
                state: TouchState::Down,
            },
            Touch {
                slot: 7,
                tracking_id: Some(11),
                x: 100,
                y: 200,
                state: TouchState::Move,
            },
        ]);

        let normalized = frame.normalize(&bounds);

        assert_eq!(normalized.len(), 2);

        assert_eq!(normalized.touches[0].slot, 2);
        assert_eq!(normalized.touches[0].tracking_id, Some(10));
        assert_eq!(normalized.touches[0].position.x, 0.0);
        assert_eq!(normalized.touches[0].position.y, 0.0);
        assert_eq!(normalized.touches[0].state, TouchState::Down);

        assert_eq!(normalized.touches[1].slot, 7);
        assert_eq!(normalized.touches[1].tracking_id, Some(11));
        assert_eq!(normalized.touches[1].position.x, 1.0);
        assert_eq!(normalized.touches[1].position.y, 1.0);
        assert_eq!(normalized.touches[1].state, TouchState::Move);
    }
}
