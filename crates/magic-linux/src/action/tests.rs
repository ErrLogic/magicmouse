use super::{
    Action, ActionEvent, ActionExecutor, ActionMapper, PointerActionMapper, ScrollActionMapper,
};
use crate::gesture::{Direction, Gesture, GestureEvent, GesturePhase};
use crate::pointer::{PointerDelta, PointerEvent};
use crate::scroll::{ScrollDelta, ScrollEvent};

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
fn pointer_pipeline_produces_executable_action() {
    let pointer_event = PointerEvent::new(PointerDelta::new(0.10, -0.05));

    let action = PointerActionMapper::map(pointer_event).expect("expected pointer action");

    let mut executor = MockExecutor::new();

    executor
        .execute(ActionEvent::new(action))
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
fn scroll_pipeline_produces_executable_action() {
    let scroll_event = ScrollEvent::new(ScrollDelta::new(-0.02, 0.15));

    let action = ScrollActionMapper::map(scroll_event).expect("expected scroll action");

    let mut executor = MockExecutor::new();

    executor
        .execute(ActionEvent::new(action))
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
fn gesture_pipeline_produces_workspace_previous_action() {
    let gesture_event = GestureEvent {
        gesture: Gesture::Swipe {
            fingers: 3,
            direction: Direction::Left,
        },
        phase: GesturePhase::Started,
    };

    let action_event = ActionMapper::map(gesture_event).expect("expected gesture action");

    let mut executor = MockExecutor::new();

    executor
        .execute(action_event)
        .expect("expected execution to succeed");

    assert_eq!(executor.executed, vec![Action::WorkspacePrevious]);
}

#[test]
fn gesture_pipeline_produces_workspace_next_action() {
    let gesture_event = GestureEvent {
        gesture: Gesture::Swipe {
            fingers: 3,
            direction: Direction::Right,
        },
        phase: GesturePhase::Started,
    };

    let action_event = ActionMapper::map(gesture_event).expect("expected gesture action");

    let mut executor = MockExecutor::new();

    executor
        .execute(action_event)
        .expect("expected execution to succeed");

    assert_eq!(executor.executed, vec![Action::WorkspaceNext]);
}

#[test]
fn unsupported_gesture_does_not_reach_executor() {
    let gesture_event = GestureEvent {
        gesture: Gesture::Swipe {
            fingers: 3,
            direction: Direction::Up,
        },
        phase: GesturePhase::Started,
    };

    let action_event = ActionMapper::map(gesture_event);

    assert!(action_event.is_none());
}
