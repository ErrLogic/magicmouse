#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModifierState {
    ctrl: bool,
}

impl ModifierState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ctrl_pressed(&self) -> bool {
        self.ctrl
    }

    pub fn set_ctrl(&mut self, pressed: bool) {
        self.ctrl = pressed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_is_not_pressed_by_default() {
        let state = ModifierState::new();

        assert!(!state.ctrl_pressed());
    }

    #[test]
    fn can_press_ctrl() {
        let mut state = ModifierState::new();

        state.set_ctrl(true);

        assert!(state.ctrl_pressed());
    }

    #[test]
    fn can_release_ctrl() {
        let mut state = ModifierState::new();

        state.set_ctrl(true);
        state.set_ctrl(false);

        assert!(!state.ctrl_pressed());
    }

    #[test]
    fn default_state_has_ctrl_released() {
        let state = ModifierState::default();

        assert!(!state.ctrl_pressed());
    }
}
