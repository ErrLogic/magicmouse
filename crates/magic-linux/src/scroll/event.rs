use super::ScrollDelta;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollEvent {
    pub delta: ScrollDelta,
}

impl ScrollEvent {
    pub fn new(delta: ScrollDelta) -> Self {
        Self { delta }
    }

    pub fn is_zero(&self) -> bool {
        self.delta.is_zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_scroll_event() {
        let delta = ScrollDelta::new(0.10, -0.05);
        let event = ScrollEvent::new(delta);

        assert_eq!(event.delta, delta);
    }

    #[test]
    fn detects_zero_scroll_event() {
        let event = ScrollEvent::new(ScrollDelta::new(0.0, 0.0));

        assert!(event.is_zero());
    }

    #[test]
    fn detects_non_zero_scroll_event() {
        let event = ScrollEvent::new(ScrollDelta::new(0.10, 0.0));

        assert!(!event.is_zero());
    }
}
