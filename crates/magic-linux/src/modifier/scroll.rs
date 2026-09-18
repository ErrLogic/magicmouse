use crate::modifier::ModifierState;
use crate::scroll::ScrollEvent;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModifierScrollEvent {
    pub scroll: ScrollEvent,
    pub modifiers: ModifierState,
}

impl ModifierScrollEvent {
    pub fn new(scroll: ScrollEvent, modifiers: ModifierState) -> Self {
        Self { scroll, modifiers }
    }

    pub fn ctrl_pressed(&self) -> bool {
        self.modifiers.ctrl_pressed()
    }
}

#[derive(Debug, Default)]
pub struct ModifierScrollMapper;

impl ModifierScrollMapper {
    pub fn new() -> Self {
        Self
    }

    pub fn map(&self, scroll: ScrollEvent, modifiers: ModifierState) -> ModifierScrollEvent {
        ModifierScrollEvent::new(scroll, modifiers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modifier::ModifierState;
    use crate::scroll::ScrollDelta;

    fn scroll_event(dx: f32, dy: f32) -> ScrollEvent {
        ScrollEvent::new(ScrollDelta::new(dx, dy))
    }

    #[test]
    fn preserves_scroll_event() {
        let scroll = scroll_event(1.0, -2.0);
        let modifiers = ModifierState::new();

        let event = ModifierScrollEvent::new(scroll, modifiers);

        assert_eq!(event.scroll, scroll);
    }

    #[test]
    fn reports_ctrl_as_not_pressed_by_default() {
        let scroll = scroll_event(0.0, 1.0);
        let modifiers = ModifierState::new();

        let event = ModifierScrollEvent::new(scroll, modifiers);

        assert!(!event.ctrl_pressed());
    }

    #[test]
    fn reports_ctrl_as_pressed() {
        let scroll = scroll_event(0.0, 1.0);

        let mut modifiers = ModifierState::new();
        modifiers.set_ctrl(true);

        let event = ModifierScrollEvent::new(scroll, modifiers);

        assert!(event.ctrl_pressed());
    }

    #[test]
    fn preserves_scroll_direction_with_ctrl() {
        let scroll = scroll_event(-3.0, 2.0);

        let mut modifiers = ModifierState::new();
        modifiers.set_ctrl(true);

        let event = ModifierScrollEvent::new(scroll, modifiers);

        assert_eq!(event.scroll.delta.dx, -3.0);
        assert_eq!(event.scroll.delta.dy, 2.0);
        assert!(event.ctrl_pressed());
    }

    #[test]
    fn maps_normal_scroll_without_ctrl() {
        let mapper = ModifierScrollMapper::new();
        let scroll = scroll_event(1.0, 2.0);
        let modifiers = ModifierState::new();

        let event = mapper.map(scroll, modifiers);

        assert_eq!(event.scroll, scroll);
        assert!(!event.ctrl_pressed());
    }

    #[test]
    fn maps_scroll_with_ctrl() {
        let mapper = ModifierScrollMapper::new();
        let scroll = scroll_event(1.0, 2.0);

        let mut modifiers = ModifierState::new();
        modifiers.set_ctrl(true);

        let event = mapper.map(scroll, modifiers);

        assert_eq!(event.scroll, scroll);
        assert!(event.ctrl_pressed());
    }
}
