use crate::axis::TouchBounds;
use crate::touch::{Touch, TouchState};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalizedPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalizedTouch {
    pub slot: i32,
    pub tracking_id: Option<i32>,
    pub position: NormalizedPoint,
    pub state: TouchState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedTouchFrame {
    pub touches: Vec<NormalizedTouch>,
}

impl NormalizedTouchFrame {
    pub fn new(touches: Vec<NormalizedTouch>) -> Self {
        Self { touches }
    }

    pub fn is_empty(&self) -> bool {
        self.touches.is_empty()
    }

    pub fn len(&self) -> usize {
        self.touches.len()
    }
}

impl Touch {
    pub fn normalize(&self, bounds: &TouchBounds) -> NormalizedTouch {
        NormalizedTouch {
            slot: self.slot,
            tracking_id: self.tracking_id,
            position: bounds.normalize_point(self.x, self.y),
            state: self.state,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::axis::{AxisBounds, TouchBounds};
    use crate::touch::{Touch, TouchFrame, TouchState};

    #[test]
    fn preserves_touch_lifecycle_during_normalization() {
        let bounds = TouchBounds::new(AxisBounds::new(-100, 100), AxisBounds::new(-200, 200));

        let frames = [
            TouchFrame::new(vec![Touch {
                slot: 3,
                tracking_id: Some(42),
                x: -50,
                y: -100,
                state: TouchState::Down,
            }]),
            TouchFrame::new(vec![Touch {
                slot: 3,
                tracking_id: Some(42),
                x: 0,
                y: 0,
                state: TouchState::Move,
            }]),
            TouchFrame::new(vec![Touch {
                slot: 3,
                tracking_id: Some(42),
                x: 50,
                y: 100,
                state: TouchState::Up,
            }]),
        ];

        let normalized: Vec<_> = frames
            .iter()
            .map(|frame| frame.normalize(&bounds))
            .collect();

        assert_eq!(normalized[0].touches[0].state, TouchState::Down);
        assert_eq!(normalized[1].touches[0].state, TouchState::Move);
        assert_eq!(normalized[2].touches[0].state, TouchState::Up);
    }
}
