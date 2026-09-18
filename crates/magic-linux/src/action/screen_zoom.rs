use crate::modifier::ModifierScrollEvent;

use super::Action;

pub struct ScreenZoomActionMapper;

impl ScreenZoomActionMapper {
    pub fn map(event: ModifierScrollEvent) -> Option<Action> {
        if !event.ctrl_pressed() {
            return None;
        }

        let delta = event.scroll.delta.dy;

        if delta == 0.0 {
            return None;
        }

        Some(Action::ScreenZoom { delta })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modifier::ModifierState;
    use crate::scroll::{ScrollDelta, ScrollEvent};

    fn modifier_scroll(dy: f32, ctrl: bool) -> ModifierScrollEvent {
        let scroll = ScrollEvent::new(ScrollDelta::new(0.0, dy));

        let mut modifiers = ModifierState::new();
        modifiers.set_ctrl(ctrl);

        ModifierScrollEvent::new(scroll, modifiers)
    }

    #[test]
    fn maps_ctrl_scroll_to_screen_zoom() {
        let event = modifier_scroll(0.25, true);

        assert_eq!(
            ScreenZoomActionMapper::map(event),
            Some(Action::ScreenZoom { delta: 0.25 })
        );
    }

    #[test]
    fn preserves_negative_zoom_delta() {
        let event = modifier_scroll(-0.50, true);

        assert_eq!(
            ScreenZoomActionMapper::map(event),
            Some(Action::ScreenZoom { delta: -0.50 })
        );
    }

    #[test]
    fn does_not_map_scroll_without_ctrl() {
        let event = modifier_scroll(0.25, false);

        assert_eq!(ScreenZoomActionMapper::map(event), None);
    }

    #[test]
    fn does_not_map_zero_scroll() {
        let event = modifier_scroll(0.0, true);

        assert_eq!(ScreenZoomActionMapper::map(event), None);
    }
}
