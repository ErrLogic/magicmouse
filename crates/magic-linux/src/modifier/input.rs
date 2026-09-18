use crate::input::RawInputEvent;

use super::{ModifierState, ModifierTracker};

#[derive(Debug, Default)]
pub struct ModifierInput {
    tracker: ModifierTracker,
}

impl ModifierInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&mut self, event: RawInputEvent) -> Option<ModifierState> {
        self.tracker.process(event)
    }

    pub fn state(&self) -> ModifierState {
        self.tracker.state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EV_KEY: u16 = 0x01;
    const KEY_LEFTCTRL: u16 = 29;
    const KEY_RIGHTCTRL: u16 = 97;

    fn key_event(code: u16, value: i32) -> RawInputEvent {
        RawInputEvent {
            event_type: EV_KEY,
            event_code: code,
            value,
        }
    }

    #[test]
    fn starts_with_ctrl_released() {
        let input = ModifierInput::new();

        assert!(!input.state().ctrl_pressed());
    }

    #[test]
    fn updates_state_when_ctrl_is_pressed() {
        let mut input = ModifierInput::new();

        let state = input
            .process(key_event(KEY_LEFTCTRL, 1))
            .expect("expected modifier state");

        assert!(state.ctrl_pressed());
        assert!(input.state().ctrl_pressed());
    }

    #[test]
    fn updates_state_when_ctrl_is_released() {
        let mut input = ModifierInput::new();

        input.process(key_event(KEY_LEFTCTRL, 1));

        let state = input
            .process(key_event(KEY_LEFTCTRL, 0))
            .expect("expected modifier state");

        assert!(!state.ctrl_pressed());
        assert!(!input.state().ctrl_pressed());
    }

    #[test]
    fn left_and_right_ctrl_share_the_same_logical_state() {
        let mut input = ModifierInput::new();

        input.process(key_event(KEY_LEFTCTRL, 1));
        assert!(input.state().ctrl_pressed());

        input.process(key_event(KEY_RIGHTCTRL, 1));
        assert!(input.state().ctrl_pressed());
    }

    #[test]
    fn ignores_unrelated_events() {
        let mut input = ModifierInput::new();

        let event = RawInputEvent {
            event_type: 0x03,
            event_code: 53,
            value: 100,
        };

        assert!(input.process(event).is_none());
        assert!(!input.state().ctrl_pressed());
    }
}
