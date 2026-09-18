use crate::normalize::NormalizedPoint;

use super::movement::{MovementVector, TrackedTouch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FingerCountChange {
    pub previous: usize,
    pub current: usize,
}

impl FingerCountChange {
    pub fn new(previous: usize, current: usize) -> Self {
        Self { previous, current }
    }

    pub fn increased(&self) -> bool {
        self.current > self.previous
    }

    pub fn decreased(&self) -> bool {
        self.current < self.previous
    }

    pub fn unchanged(&self) -> bool {
        self.current == self.previous
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GestureState {
    touches: Vec<TrackedTouch>,
}

impl GestureState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn touches(&self) -> &[TrackedTouch] {
        &self.touches
    }

    pub fn finger_count(&self) -> usize {
        self.touches.len()
    }

    pub fn is_empty(&self) -> bool {
        self.touches.is_empty()
    }

    pub fn add_touch(&mut self, touch: TrackedTouch) {
        self.touches.push(touch);
    }

    pub fn update_touch(&mut self, tracking_id: i32, position: NormalizedPoint) {
        if let Some(touch) = self
            .touches
            .iter_mut()
            .find(|touch| touch.tracking_id == tracking_id)
        {
            touch.update(position);
        }
    }

    pub fn remove_touch(&mut self, tracking_id: i32) {
        self.touches
            .retain(|touch| touch.tracking_id != tracking_id);
    }

    pub fn remove_touch_by_slot(&mut self, slot: i32) {
        self.touches.retain(|touch| touch.slot != Some(slot));
    }

    pub fn finger_count_change(&self, previous: usize) -> FingerCountChange {
        FingerCountChange {
            previous,
            current: self.finger_count(),
        }
    }

    pub fn centroid_movement(&self) -> Option<MovementVector> {
        if self.touches.is_empty() {
            return None;
        }

        let start_x =
            self.touches.iter().map(|touch| touch.start.x).sum::<f32>() / self.touches.len() as f32;

        let start_y =
            self.touches.iter().map(|touch| touch.start.y).sum::<f32>() / self.touches.len() as f32;

        let current_x = self
            .touches
            .iter()
            .map(|touch| touch.current.x)
            .sum::<f32>()
            / self.touches.len() as f32;

        let current_y = self
            .touches
            .iter()
            .map(|touch| touch.current.y)
            .sum::<f32>()
            / self.touches.len() as f32;

        Some(MovementVector::new(
            current_x - start_x,
            current_y - start_y,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(slot: i32, tracking_id: i32, x: f32, y: f32) -> TrackedTouch {
        TrackedTouch::with_slot(slot, tracking_id, NormalizedPoint { x, y })
    }

    #[test]
    fn tracks_finger_count() {
        let mut state = GestureState::new();

        assert_eq!(state.finger_count(), 0);
        assert!(state.is_empty());

        state.add_touch(touch(0, 1, 0.1, 0.1));

        assert_eq!(state.finger_count(), 1);
        assert!(!state.is_empty());

        state.add_touch(touch(1, 2, 0.2, 0.2));

        assert_eq!(state.finger_count(), 2);
    }

    #[test]
    fn updates_touch_by_tracking_id() {
        let mut state = GestureState::new();

        state.add_touch(touch(2, 42, 0.1, 0.2));

        state.update_touch(42, NormalizedPoint { x: 0.8, y: 0.9 });

        let tracked = &state.touches()[0];

        assert_eq!(tracked.current, NormalizedPoint { x: 0.8, y: 0.9 });
    }

    #[test]
    fn removes_touch_by_tracking_id() {
        let mut state = GestureState::new();

        state.add_touch(touch(0, 1, 0.1, 0.1));
        state.add_touch(touch(1, 2, 0.2, 0.2));

        state.remove_touch(1);

        assert_eq!(state.finger_count(), 1);
        assert_eq!(state.touches()[0].tracking_id, 2);
    }

    #[test]
    fn removes_touch_by_slot() {
        let mut state = GestureState::new();

        state.add_touch(touch(2, 100, 0.1, 0.1));
        state.add_touch(touch(4, 101, 0.2, 0.2));
        state.add_touch(touch(9, 102, 0.3, 0.3));

        state.remove_touch_by_slot(4);

        assert_eq!(state.finger_count(), 2);

        assert_eq!(state.touches()[0].slot, Some(2));
        assert_eq!(state.touches()[1].slot, Some(9));
    }

    #[test]
    fn tracks_finger_count_transitions() {
        let mut state = GestureState::new();

        state.add_touch(touch(0, 1, 0.1, 0.1));

        let change = state.finger_count_change(0);

        assert_eq!(change.previous, 0);
        assert_eq!(change.current, 1);
        assert!(change.increased());
        assert!(!change.decreased());
        assert!(!change.unchanged());
    }

    #[test]
    fn detects_increased_finger_count() {
        let change = FingerCountChange::new(2, 3);

        assert!(change.increased());
        assert!(!change.decreased());
        assert!(!change.unchanged());
    }

    #[test]
    fn detects_decreased_finger_count() {
        let change = FingerCountChange::new(3, 2);

        assert!(!change.increased());
        assert!(change.decreased());
        assert!(!change.unchanged());
    }

    #[test]
    fn detects_unchanged_finger_count() {
        let change = FingerCountChange::new(3, 3);

        assert!(!change.increased());
        assert!(!change.decreased());
        assert!(change.unchanged());
    }

    fn point(x: f32, y: f32) -> NormalizedPoint {
        NormalizedPoint { x, y }
    }

    #[test]
    fn centroid_movement_returns_none_when_state_is_empty() {
        let state = GestureState::new();

        assert_eq!(state.centroid_movement(), None);
    }

    #[test]
    fn centroid_movement_calculates_collective_right_movement() {
        let mut state = GestureState::new();

        state.add_touch(touch(1, 1, 0.1, 0.5));
        state.add_touch(touch(2, 2, 0.2, 0.5));
        state.add_touch(touch(3, 3, 0.3, 0.5));

        state.update_touch(1, point(0.2, 0.5));
        state.update_touch(2, point(0.3, 0.5));
        state.update_touch(3, point(0.4, 0.5));

        let movement = state.centroid_movement().unwrap();

        assert!((movement.dx - 0.1).abs() < 0.0001);
        assert!(movement.dy.abs() < 0.0001);
        assert_eq!(movement.direction(), Some(crate::gesture::Direction::Right));
    }

    #[test]
    fn centroid_movement_ignores_individual_finger_noise() {
        let mut state = GestureState::new();

        state.add_touch(touch(1, 1, 0.1, 0.5));
        state.add_touch(touch(2, 2, 0.2, 0.5));
        state.add_touch(touch(3, 3, 0.3, 0.5));

        state.update_touch(1, point(0.3, 0.5));
        state.update_touch(2, point(0.2, 0.5));
        state.update_touch(3, point(0.4, 0.5));

        let movement = state.centroid_movement().unwrap();

        assert!((movement.dx - 0.1).abs() < 0.0001);
        assert!(movement.dy.abs() < 0.0001);
        assert_eq!(movement.direction(), Some(crate::gesture::Direction::Right));
    }
}
