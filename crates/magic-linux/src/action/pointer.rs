use crate::pointer::PointerEvent;

use super::Action;

pub struct PointerActionMapper;

impl PointerActionMapper {
    pub fn map(event: PointerEvent) -> Option<Action> {
        if event.is_zero() {
            return None;
        }

        Some(Action::Pointer {
            dx: event.delta.dx,
            dy: event.delta.dy,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pointer::PointerDelta;

    #[test]
    fn maps_pointer_event_to_pointer_action() {
        let event = PointerEvent::new(PointerDelta::new(0.10, -0.05));

        assert_eq!(
            PointerActionMapper::map(event),
            Some(Action::Pointer {
                dx: 0.10,
                dy: -0.05,
            })
        );
    }

    #[test]
    fn preserves_negative_pointer_delta() {
        let event = PointerEvent::new(PointerDelta::new(-0.20, 0.15));

        assert_eq!(
            PointerActionMapper::map(event),
            Some(Action::Pointer {
                dx: -0.20,
                dy: 0.15,
            })
        );
    }

    #[test]
    fn zero_pointer_event_does_not_create_action() {
        let event = PointerEvent::new(PointerDelta::new(0.0, 0.0));

        assert_eq!(PointerActionMapper::map(event), None);
    }
}
