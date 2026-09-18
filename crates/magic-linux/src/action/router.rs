use crate::{gesture::GestureEvent, pointer::PointerEvent, scroll::ScrollEvent};

use super::{ActionEvent, ActionMapper, PointerActionMapper, ScrollActionMapper};

pub struct ActionRouter;

impl ActionRouter {
    pub fn from_pointer(event: PointerEvent) -> Option<ActionEvent> {
        PointerActionMapper::map(event).map(ActionEvent::new)
    }

    pub fn from_scroll(event: ScrollEvent) -> Option<ActionEvent> {
        ScrollActionMapper::map(event).map(ActionEvent::new)
    }

    pub fn from_gesture(event: GestureEvent) -> Option<ActionEvent> {
        ActionMapper::map(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::ActionExecutor;
    use crate::{
        action::Action,
        gesture::{Direction, Gesture, GesturePhase},
        pointer::PointerDelta,
        scroll::ScrollDelta,
    };

    struct MockExecutor {
        executed: Vec<Action>,
    }

    impl MockExecutor {
        fn new() -> Self {
            Self {
                executed: Vec::new(),
            }
        }
    }

    impl ActionExecutor for MockExecutor {
        type Error = ();

        fn execute(&mut self, event: ActionEvent) -> Result<(), Self::Error> {
            self.executed.push(event.action);
            Ok(())
        }
    }

    #[test]
    fn routes_pointer_event_to_action_event() {
        let event = PointerEvent::new(PointerDelta::new(0.10, -0.05));

        let action_event = ActionRouter::from_pointer(event).expect("expected pointer action");

        assert_eq!(
            action_event,
            ActionEvent::new(Action::Pointer {
                dx: 0.10,
                dy: -0.05,
            })
        );
    }

    #[test]
    fn routes_scroll_event_to_action_event() {
        let event = ScrollEvent::new(ScrollDelta::new(-0.02, 0.15));

        let action_event = ActionRouter::from_scroll(event).expect("expected scroll action");

        assert_eq!(
            action_event,
            ActionEvent::new(Action::Scroll {
                dx: -0.02,
                dy: 0.15,
            })
        );
    }

    #[test]
    fn routes_left_gesture_to_workspace_previous() {
        let event = GestureEvent {
            gesture: Gesture::Swipe {
                fingers: 3,
                direction: Direction::Left,
            },
            phase: GesturePhase::Started,
        };

        let action_event = ActionRouter::from_gesture(event).expect("expected gesture action");

        assert_eq!(action_event, ActionEvent::new(Action::WorkspacePrevious));
    }

    #[test]
    fn routes_right_gesture_to_workspace_next() {
        let event = GestureEvent {
            gesture: Gesture::Swipe {
                fingers: 3,
                direction: Direction::Right,
            },
            phase: GesturePhase::Started,
        };

        let action_event = ActionRouter::from_gesture(event).expect("expected gesture action");

        assert_eq!(action_event, ActionEvent::new(Action::WorkspaceNext));
    }

    #[test]
    fn zero_pointer_event_is_not_routed() {
        let event = PointerEvent::new(PointerDelta::new(0.0, 0.0));

        assert!(ActionRouter::from_pointer(event).is_none());
    }

    #[test]
    fn zero_scroll_event_is_not_routed() {
        let event = ScrollEvent::new(ScrollDelta::new(0.0, 0.0));

        assert!(ActionRouter::from_scroll(event).is_none());
    }

    #[test]
    fn unsupported_gesture_is_not_routed() {
        let event = GestureEvent {
            gesture: Gesture::Swipe {
                fingers: 3,
                direction: Direction::Up,
            },
            phase: GesturePhase::Started,
        };

        assert!(ActionRouter::from_gesture(event).is_none());
    }

    #[test]
    fn updated_gesture_is_not_routed() {
        let event = GestureEvent {
            gesture: Gesture::Swipe {
                fingers: 3,
                direction: Direction::Left,
            },
            phase: GesturePhase::Updated,
        };

        assert!(ActionRouter::from_gesture(event).is_none());
    }

    #[test]
    fn routed_pointer_action_can_be_executed() {
        let pointer_event = PointerEvent::new(PointerDelta::new(0.10, -0.05));

        let action_event =
            ActionRouter::from_pointer(pointer_event).expect("expected pointer action event");

        let mut executor = MockExecutor::new();

        executor
            .execute(action_event)
            .expect("expected execution to succeed");

        assert_eq!(
            executor.executed,
            vec![Action::Pointer {
                dx: 0.10,
                dy: -0.05,
            }]
        );
    }

    #[test]
    fn routed_scroll_action_can_be_executed() {
        let scroll_event = ScrollEvent::new(ScrollDelta::new(-0.02, 0.15));

        let action_event =
            ActionRouter::from_scroll(scroll_event).expect("expected scroll action event");

        let mut executor = MockExecutor::new();

        executor
            .execute(action_event)
            .expect("expected execution to succeed");

        assert_eq!(
            executor.executed,
            vec![Action::Scroll {
                dx: -0.02,
                dy: 0.15,
            }]
        );
    }

    #[test]
    fn routed_gesture_action_can_be_executed() {
        let gesture_event = GestureEvent {
            gesture: Gesture::Swipe {
                fingers: 3,
                direction: Direction::Right,
            },
            phase: GesturePhase::Started,
        };

        let action_event =
            ActionRouter::from_gesture(gesture_event).expect("expected gesture action event");

        let mut executor = MockExecutor::new();

        executor
            .execute(action_event)
            .expect("expected execution to succeed");

        assert_eq!(executor.executed, vec![Action::WorkspaceNext]);
    }
}
