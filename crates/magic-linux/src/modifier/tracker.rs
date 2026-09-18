use crate::input::RawInputEvent;

use super::ModifierState;

const EV_KEY: u16 = 0x01;
const KEY_LEFTCTRL: u16 = 29;
const KEY_RIGHTCTRL: u16 = 97;

#[derive(Debug, Default)]
pub struct ModifierTracker {
    state: ModifierState,
}

impl ModifierTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> ModifierState {
        self.state
    }

    pub fn process(&mut self, event: RawInputEvent) -> Option<ModifierState> {
        if event.event_type != EV_KEY {
            return None;
        }

        if event.event_code != KEY_LEFTCTRL && event.event_code != KEY_RIGHTCTRL {
            return None;
        }

        match event.value {
            1 => {
                self.state.set_ctrl(true);
                Some(self.state)
            }
            0 => {
                self.state.set_ctrl(false);
                Some(self.state)
            }
            2 => None,
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_event(code: u16, value: i32) -> RawInputEvent {
        RawInputEvent {
            event_type: EV_KEY,
            event_code: code,
            value,
        }
    }

    #[test]
    fn recognizes_left_ctrl_press() {
        let mut tracker = ModifierTracker::new();

        let state = tracker
            .process(key_event(KEY_LEFTCTRL, 1))
            .expect("expected modifier state");

        assert!(state.ctrl_pressed());
    }

    #[test]
    fn recognizes_right_ctrl_press() {
        let mut tracker = ModifierTracker::new();

        let state = tracker
            .process(key_event(KEY_RIGHTCTRL, 1))
            .expect("expected modifier state");

        assert!(state.ctrl_pressed());
    }

    #[test]
    fn recognizes_ctrl_release() {
        let mut tracker = ModifierTracker::new();

        tracker.process(key_event(KEY_LEFTCTRL, 1));

        let state = tracker
            .process(key_event(KEY_LEFTCTRL, 0))
            .expect("expected modifier state");

        assert!(!state.ctrl_pressed());
    }

    #[test]
    fn ignores_key_repeat() {
        let mut tracker = ModifierTracker::new();

        tracker.process(key_event(KEY_LEFTCTRL, 1));

        assert!(tracker.process(key_event(KEY_LEFTCTRL, 2)).is_none());
        assert!(tracker.state().ctrl_pressed());
    }

    #[test]
    fn ignores_non_ctrl_key() {
        let mut tracker = ModifierTracker::new();

        assert!(tracker.process(key_event(30, 1)).is_none());
        assert!(!tracker.state().ctrl_pressed());
    }

    #[test]
    fn ignores_non_key_event() {
        let mut tracker = ModifierTracker::new();

        let event = RawInputEvent {
            event_type: 0x03,
            event_code: KEY_LEFTCTRL,
            value: 1,
        };

        assert!(tracker.process(event).is_none());
        assert!(!tracker.state().ctrl_pressed());
    }
}
