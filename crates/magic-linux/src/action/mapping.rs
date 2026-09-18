use crate::gesture::{Direction, Gesture, GestureEvent, GesturePhase};

use super::{Action, ActionEvent};

pub struct ActionMapper;

impl ActionMapper {
    pub fn map(event: GestureEvent) -> Option<ActionEvent> {
        if event.phase != GesturePhase::Started {
            return None;
        }

        let action = match event.gesture {
            Gesture::Swipe {
                fingers: 3,
                direction: Direction::Left,
            } => Action::WorkspacePrevious,

            Gesture::Swipe {
                fingers: 3,
                direction: Direction::Right,
            } => Action::WorkspaceNext,

            _ => return None,
        };

        Some(ActionEvent::new(action))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_three_finger_swipe_left_to_workspace_previous() {
        let event = GestureEvent {
            gesture: Gesture::Swipe {
                fingers: 3,
                direction: Direction::Left,
            },
            phase: GesturePhase::Started,
        };

        assert_eq!(
            ActionMapper::map(event),
            Some(ActionEvent::new(Action::WorkspacePrevious))
        );
    }

    #[test]
    fn maps_three_finger_swipe_right_to_workspace_next() {
        let event = GestureEvent {
            gesture: Gesture::Swipe {
                fingers: 3,
                direction: Direction::Right,
            },
            phase: GesturePhase::Started,
        };

        assert_eq!(
            ActionMapper::map(event),
            Some(ActionEvent::new(Action::WorkspaceNext))
        );
    }

    #[test]
    fn does_not_map_three_finger_swipe_up() {
        let event = GestureEvent {
            gesture: Gesture::Swipe {
                fingers: 3,
                direction: Direction::Up,
            },
            phase: GesturePhase::Started,
        };

        assert_eq!(ActionMapper::map(event), None);
    }

    #[test]
    fn does_not_map_three_finger_swipe_down() {
        let event = GestureEvent {
            gesture: Gesture::Swipe {
                fingers: 3,
                direction: Direction::Down,
            },
            phase: GesturePhase::Started,
        };

        assert_eq!(ActionMapper::map(event), None);
    }

    #[test]
    fn does_not_map_non_three_finger_swipe() {
        let event = GestureEvent {
            gesture: Gesture::Swipe {
                fingers: 2,
                direction: Direction::Left,
            },
            phase: GesturePhase::Started,
        };

        assert_eq!(ActionMapper::map(event), None);
    }

    #[test]
    fn does_not_map_updated_gesture_to_action() {
        let event = GestureEvent {
            gesture: Gesture::Swipe {
                fingers: 3,
                direction: Direction::Left,
            },
            phase: GesturePhase::Updated,
        };

        assert_eq!(ActionMapper::map(event), None);
    }

    #[test]
    fn does_not_map_ended_gesture_to_action() {
        let event = GestureEvent {
            gesture: Gesture::Swipe {
                fingers: 3,
                direction: Direction::Left,
            },
            phase: GesturePhase::Ended,
        };

        assert_eq!(ActionMapper::map(event), None);
    }

    #[test]
    fn does_not_map_cancelled_gesture_to_action() {
        let event = GestureEvent {
            gesture: Gesture::Swipe {
                fingers: 3,
                direction: Direction::Left,
            },
            phase: GesturePhase::Cancelled,
        };

        assert_eq!(ActionMapper::map(event), None);
    }
}
