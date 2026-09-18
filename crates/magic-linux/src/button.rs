use crate::input::RawInputEvent;

const EV_KEY: u16 = 1;
const BTN_LEFT: u16 = 272;
const BTN_RIGHT: u16 = 273;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalButton {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonPhase {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonEvent {
    pub button: PhysicalButton,
    pub phase: ButtonPhase,
}

#[derive(Debug, Default)]
pub struct PhysicalButtonTracker;

impl PhysicalButtonTracker {
    pub fn new() -> Self {
        Self
    }

    pub fn process(&mut self, event: RawInputEvent) -> Option<ButtonEvent> {
        if event.event_type != EV_KEY {
            return None;
        }

        let button = match event.event_code {
            BTN_LEFT => PhysicalButton::Left,
            BTN_RIGHT => PhysicalButton::Right,
            _ => return None,
        };

        let phase = match event.value {
            1 => ButtonPhase::Pressed,
            0 => ButtonPhase::Released,
            _ => return None,
        };

        Some(ButtonEvent { button, phase })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(event_code: u16, value: i32) -> RawInputEvent {
        RawInputEvent {
            event_type: EV_KEY,
            event_code,
            value,
        }
    }

    #[test]
    fn recognizes_left_button_press() {
        let mut tracker = PhysicalButtonTracker::new();

        assert_eq!(
            tracker.process(event(BTN_LEFT, 1)),
            Some(ButtonEvent {
                button: PhysicalButton::Left,
                phase: ButtonPhase::Pressed,
            })
        );
    }

    #[test]
    fn recognizes_left_button_release() {
        let mut tracker = PhysicalButtonTracker::new();

        assert_eq!(
            tracker.process(event(BTN_LEFT, 0)),
            Some(ButtonEvent {
                button: PhysicalButton::Left,
                phase: ButtonPhase::Released,
            })
        );
    }

    #[test]
    fn recognizes_right_button_press() {
        let mut tracker = PhysicalButtonTracker::new();

        assert_eq!(
            tracker.process(event(BTN_RIGHT, 1)),
            Some(ButtonEvent {
                button: PhysicalButton::Right,
                phase: ButtonPhase::Pressed,
            })
        );
    }

    #[test]
    fn recognizes_right_button_release() {
        let mut tracker = PhysicalButtonTracker::new();

        assert_eq!(
            tracker.process(event(BTN_RIGHT, 0)),
            Some(ButtonEvent {
                button: PhysicalButton::Right,
                phase: ButtonPhase::Released,
            })
        );
    }

    #[test]
    fn ignores_key_repeat() {
        let mut tracker = PhysicalButtonTracker::new();

        assert_eq!(tracker.process(event(BTN_LEFT, 2)), None);
    }

    #[test]
    fn ignores_unknown_button() {
        let mut tracker = PhysicalButtonTracker::new();

        assert_eq!(tracker.process(event(274, 1)), None);
    }

    #[test]
    fn ignores_non_key_event() {
        let mut tracker = PhysicalButtonTracker::new();

        let event = RawInputEvent {
            event_type: 3,
            event_code: 53,
            value: 100,
        };

        assert_eq!(tracker.process(event), None);
    }
}
