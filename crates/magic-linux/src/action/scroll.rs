use crate::scroll::ScrollEvent;

use super::Action;

pub struct ScrollActionMapper;

impl ScrollActionMapper {
    pub fn map(event: ScrollEvent) -> Option<Action> {
        if event.is_zero() {
            return None;
        }

        Some(Action::Scroll {
            dx: event.delta.dx,
            dy: event.delta.dy,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scroll::ScrollDelta;

    #[test]
    fn maps_scroll_event_to_scroll_action() {
        let event = ScrollEvent::new(ScrollDelta::new(0.05, -0.10));

        assert_eq!(
            ScrollActionMapper::map(event),
            Some(Action::Scroll {
                dx: 0.05,
                dy: -0.10,
            })
        );
    }

    #[test]
    fn preserves_negative_scroll_delta() {
        let event = ScrollEvent::new(ScrollDelta::new(-0.20, 0.15));

        assert_eq!(
            ScrollActionMapper::map(event),
            Some(Action::Scroll {
                dx: -0.20,
                dy: 0.15,
            })
        );
    }

    #[test]
    fn zero_scroll_event_does_not_create_action() {
        let event = ScrollEvent::new(ScrollDelta::new(0.0, 0.0));

        assert_eq!(ScrollActionMapper::map(event), None);
    }
}
