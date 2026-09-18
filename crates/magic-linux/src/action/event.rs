use super::Action;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionEvent {
    pub action: Action,
}

impl ActionEvent {
    pub fn new(action: Action) -> Self {
        Self { action }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_action_event() {
        let event = ActionEvent::new(Action::WorkspaceNext);

        assert_eq!(
            event,
            ActionEvent {
                action: Action::WorkspaceNext,
            }
        );
    }

    #[test]
    fn preserves_pointer_action() {
        let action = Action::Pointer {
            dx: 0.10,
            dy: -0.05,
        };

        let event = ActionEvent::new(action);

        assert_eq!(event.action, action);
    }

    #[test]
    fn preserves_scroll_action() {
        let action = Action::Scroll {
            dx: -0.02,
            dy: 0.15,
        };

        let event = ActionEvent::new(action);

        assert_eq!(event.action, action);
    }
}
