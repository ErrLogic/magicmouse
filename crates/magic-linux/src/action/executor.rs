use super::ActionEvent;

pub trait ActionExecutor {
    type Error;

    fn execute(&mut self, event: ActionEvent) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;

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
    fn executor_receives_action_event() {
        let mut executor = MockExecutor::new();

        let event = ActionEvent::new(Action::WorkspaceNext);

        executor.execute(event).unwrap();

        assert_eq!(executor.executed, vec![Action::WorkspaceNext]);
    }

    #[test]
    fn executor_preserves_pointer_action() {
        let mut executor = MockExecutor::new();

        let action = Action::Pointer {
            dx: 0.10,
            dy: -0.05,
        };

        executor.execute(ActionEvent::new(action)).unwrap();

        assert_eq!(executor.executed, vec![action]);
    }

    #[test]
    fn executor_preserves_scroll_action() {
        let mut executor = MockExecutor::new();

        let action = Action::Scroll {
            dx: -0.02,
            dy: 0.15,
        };

        executor.execute(ActionEvent::new(action)).unwrap();

        assert_eq!(executor.executed, vec![action]);
    }
}
