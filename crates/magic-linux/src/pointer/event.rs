use super::PointerDelta;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerEvent {
    pub delta: PointerDelta,
}

impl PointerEvent {
    pub fn new(delta: PointerDelta) -> Self {
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
    fn creates_pointer_event() {
        let delta = PointerDelta::new(0.10, -0.05);
        let event = PointerEvent::new(delta);

        assert_eq!(event.delta, delta);
    }

    #[test]
    fn detects_zero_pointer_event() {
        let event = PointerEvent::new(PointerDelta::new(0.0, 0.0));

        assert!(event.is_zero());
    }

    #[test]
    fn detects_non_zero_pointer_event() {
        let event = PointerEvent::new(PointerDelta::new(0.10, 0.0));

        assert!(!event.is_zero());
    }
}
