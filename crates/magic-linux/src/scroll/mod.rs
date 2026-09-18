mod event;
mod model;
mod movement;
mod sensitivity;
mod tracker;

pub use event::ScrollEvent;
pub use model::ScrollDelta;
pub use movement::ScrollMovement;
pub use sensitivity::ScrollSensitivity;
pub use tracker::ScrollTracker;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize::NormalizedPoint;

    #[test]
    fn scroll_pipeline_calculates_scales_and_emits_event() {
        let previous = NormalizedPoint { x: 0.60, y: 0.40 };

        let current = NormalizedPoint { x: 0.62, y: 0.35 };

        let delta = ScrollMovement::from_points(previous, current);

        let sensitivity = ScrollSensitivity::new(3.0);
        let scaled = sensitivity.apply(delta);

        let event = ScrollEvent::new(scaled);

        assert!((event.delta.dx - 0.06).abs() < 0.0001);
        assert!((event.delta.dy - (-0.15)).abs() < 0.0001);
        assert!(!event.is_zero());
    }
}
