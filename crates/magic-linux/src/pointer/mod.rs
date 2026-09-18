mod event;
mod model;
mod movement;
mod sensitivity;
mod tracker;

pub use event::PointerEvent;
pub use model::PointerDelta;
pub use movement::PointerMovement;
pub use sensitivity::PointerSensitivity;
pub use tracker::PointerTracker;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize::NormalizedPoint;

    #[test]
    fn pointer_pipeline_calculates_scales_and_emits_event() {
        let previous = NormalizedPoint { x: 0.40, y: 0.50 };

        let current = NormalizedPoint { x: 0.45, y: 0.47 };

        let delta = PointerMovement::from_points(previous, current);

        let sensitivity = PointerSensitivity::new(2.0);
        let scaled = sensitivity.apply(delta);

        let event = PointerEvent::new(scaled);

        assert!((event.delta.dx - 0.10).abs() < 0.0001);
        assert!((event.delta.dy - (-0.06)).abs() < 0.0001);
        assert!(!event.is_zero());
    }
}
