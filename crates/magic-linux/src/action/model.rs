#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Pointer { dx: f32, dy: f32 },
    Scroll { dx: f32, dy: f32 },
    WorkspacePrevious,
    WorkspaceNext,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_pointer_action() {
        let action = Action::Pointer {
            dx: 0.10,
            dy: -0.05,
        };

        assert_eq!(
            action,
            Action::Pointer {
                dx: 0.10,
                dy: -0.05,
            }
        );
    }

    #[test]
    fn creates_scroll_action() {
        let action = Action::Scroll {
            dx: 0.02,
            dy: -0.10,
        };

        assert_eq!(
            action,
            Action::Scroll {
                dx: 0.02,
                dy: -0.10,
            }
        );
    }

    #[test]
    fn creates_workspace_previous_action() {
        let action = Action::WorkspacePrevious;

        assert_eq!(action, Action::WorkspacePrevious);
    }

    #[test]
    fn creates_workspace_next_action() {
        let action = Action::WorkspaceNext;

        assert_eq!(action, Action::WorkspaceNext);
    }

    #[test]
    fn actions_are_distinct() {
        assert_ne!(Action::WorkspacePrevious, Action::WorkspaceNext);
    }

    #[test]
    fn action_preserves_pointer_delta() {
        let action = Action::Pointer {
            dx: 0.125,
            dy: -0.250,
        };

        assert_eq!(
            action,
            Action::Pointer {
                dx: 0.125,
                dy: -0.250,
            }
        );
    }

    #[test]
    fn action_preserves_scroll_delta() {
        let action = Action::Scroll {
            dx: -0.050,
            dy: 0.300,
        };

        assert_eq!(
            action,
            Action::Scroll {
                dx: -0.050,
                dy: 0.300,
            }
        );
    }
}
