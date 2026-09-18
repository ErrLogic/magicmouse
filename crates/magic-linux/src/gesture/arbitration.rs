use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::gesture::{
    DoubleTapEvent, DoubleTapRecognizer, GestureEvent, GestureRecognizer, GestureState,
    MovementThreshold, OneFingerSwipeEvent, OneFingerSwipeRecognizer, TapEvent, TapRecognizer,
    TrackedTouch, TwoFingerSwipeEvent, TwoFingerSwipeRecognizer,
};
use crate::normalize::NormalizedTouchFrame;
use crate::touch::TouchState;

const DEFAULT_DOUBLE_TAP_INTERVAL: Duration = Duration::from_millis(300);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecognizedGesture {
    Tap(TapEvent),
    DoubleTap(DoubleTapEvent),
    OneFingerSwipe(OneFingerSwipeEvent),
    TwoFingerSwipe(TwoFingerSwipeEvent),
    ThreeFingerSwipe(GestureEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingTap {
    event: TapEvent,
    completed_at: Instant,
}

#[derive(Debug)]
pub struct GestureArbitrator {
    tap_recognizer: TapRecognizer,
    double_tap_recognizer: DoubleTapRecognizer,
    one_finger_swipe_recognizer: OneFingerSwipeRecognizer,
    two_finger_swipe_recognizer: TwoFingerSwipeRecognizer,
    three_finger_gesture_recognizer: GestureRecognizer,

    gesture_state: GestureState,

    pending_tap: Option<PendingTap>,
    pending_events: VecDeque<RecognizedGesture>,

    double_tap_interval: Duration,
}

impl GestureArbitrator {
    pub fn new() -> Self {
        Self {
            tap_recognizer: TapRecognizer::new(),
            double_tap_recognizer: DoubleTapRecognizer::new(),
            one_finger_swipe_recognizer: OneFingerSwipeRecognizer::new(),
            two_finger_swipe_recognizer: TwoFingerSwipeRecognizer::new(),
            three_finger_gesture_recognizer: GestureRecognizer::new(MovementThreshold::new(0.15)),

            gesture_state: GestureState::new(),

            pending_tap: None,
            pending_events: VecDeque::new(),

            double_tap_interval: DEFAULT_DOUBLE_TAP_INTERVAL,
        }
    }

    pub fn process(
        &mut self,
        frame: &NormalizedTouchFrame,
        timestamp: Instant,
    ) -> Option<RecognizedGesture> {
        self.update_gesture_state(frame);

        if let Some(event) = self
            .three_finger_gesture_recognizer
            .process(&self.gesture_state)
        {
            self.pending_tap = None;

            self.pending_events
                .push_back(RecognizedGesture::ThreeFingerSwipe(event));
        }

        if let Some(event) = self.one_finger_swipe_recognizer.process(frame) {
            self.pending_tap = None;

            self.pending_events
                .push_back(RecognizedGesture::OneFingerSwipe(event));
        }

        if let Some(event) = self.two_finger_swipe_recognizer.process(frame) {
            self.pending_tap = None;

            self.pending_events
                .push_back(RecognizedGesture::TwoFingerSwipe(event));
        }

        /*
         * Double tap must be evaluated before flushing a pending
         * single tap.
         */
        if let Some(event) = self.double_tap_recognizer.process(frame, timestamp) {
            self.pending_tap = None;

            self.pending_events
                .push_back(RecognizedGesture::DoubleTap(event));
        }

        /*
         * Tap recognition is deliberately deferred.
         *
         * If this completes a tap, it becomes pending instead of
         * being emitted immediately.
         */
        if let Some(event) = self.tap_recognizer.process(frame) {
            self.pending_tap = Some(PendingTap {
                event,
                completed_at: timestamp,
            });
        }

        /*
         * If the previous tap has expired, emit it now.
         *
         * The current frame has already been processed above, so
         * a new tap can simultaneously become pending.
         */
        self.flush_expired_tap(timestamp);

        self.pending_events.pop_front()
    }

    pub fn flush(&mut self, timestamp: Instant) -> Option<RecognizedGesture> {
        self.flush_expired_tap(timestamp);

        self.pending_events.pop_front()
    }

    fn flush_expired_tap(&mut self, timestamp: Instant) {
        let Some(pending) = self.pending_tap else {
            return;
        };

        if timestamp.saturating_duration_since(pending.completed_at) < self.double_tap_interval {
            return;
        }

        self.pending_tap = None;

        self.pending_events
            .push_back(RecognizedGesture::Tap(pending.event));
    }

    fn update_gesture_state(&mut self, frame: &NormalizedTouchFrame) {
        for touch in &frame.touches {
            match touch.state {
                TouchState::Down => {
                    let Some(tracking_id) = touch.tracking_id else {
                        continue;
                    };

                    self.gesture_state.add_touch(TrackedTouch::with_slot(
                        touch.slot,
                        tracking_id,
                        touch.position,
                    ));
                }

                TouchState::Move => {
                    let Some(tracking_id) = touch.tracking_id else {
                        continue;
                    };

                    self.gesture_state.update_touch(tracking_id, touch.position);
                }

                TouchState::Up => {
                    self.gesture_state.remove_touch_by_slot(touch.slot);
                }
            }
        }
    }
}

impl Default for GestureArbitrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::gesture::{Direction, Gesture, GesturePhase};
    use crate::normalize::{NormalizedPoint, NormalizedTouch};
    use std::time::Duration;

    fn touch(
        slot: i32,
        tracking_id: Option<i32>,
        x: f32,
        y: f32,
        state: TouchState,
    ) -> NormalizedTouch {
        NormalizedTouch {
            slot,
            tracking_id,
            position: NormalizedPoint { x, y },
            state,
        }
    }

    fn frame(touches: Vec<NormalizedTouch>) -> NormalizedTouchFrame {
        NormalizedTouchFrame { touches }
    }

    fn down(slot: i32, x: f32, y: f32) -> NormalizedTouch {
        touch(slot, Some(slot), x, y, TouchState::Down)
    }

    fn move_touch(slot: i32, x: f32, y: f32) -> NormalizedTouch {
        touch(slot, Some(slot), x, y, TouchState::Move)
    }

    fn up(slot: i32, x: f32, y: f32) -> NormalizedTouch {
        touch(slot, None, x, y, TouchState::Up)
    }

    #[test]
    fn single_tap_is_deferred() {
        let mut arbitrator = GestureArbitrator::new();
        let start = Instant::now();

        assert_eq!(
            arbitrator.process(&frame(vec![down(1, 0.5, 0.5)]), start,),
            None
        );

        assert_eq!(
            arbitrator.process(&frame(vec![up(1, 0.5, 0.5)]), start,),
            None
        );

        assert_eq!(arbitrator.flush(start + Duration::from_millis(299)), None);

        assert_eq!(
            arbitrator.flush(start + Duration::from_millis(300)),
            Some(RecognizedGesture::Tap(TapEvent { fingers: 1 }))
        );
    }

    #[test]
    fn single_tap_is_not_emitted_before_timeout() {
        let mut arbitrator = GestureArbitrator::new();
        let start = Instant::now();

        arbitrator.process(&frame(vec![down(1, 0.5, 0.5)]), start);

        arbitrator.process(&frame(vec![up(1, 0.5, 0.5)]), start);

        assert_eq!(arbitrator.flush(start + Duration::from_millis(299)), None);
    }

    #[test]
    fn double_tap_replaces_pending_single_tap() {
        let mut arbitrator = GestureArbitrator::new();
        let start = Instant::now();

        arbitrator.process(&frame(vec![down(1, 0.5, 0.5)]), start);

        assert_eq!(
            arbitrator.process(&frame(vec![up(1, 0.5, 0.5)]), start,),
            None
        );

        let second_tap = start + Duration::from_millis(100);

        arbitrator.process(&frame(vec![down(2, 0.5, 0.5)]), second_tap);

        assert_eq!(
            arbitrator.process(&frame(vec![up(2, 0.5, 0.5)]), second_tap,),
            Some(RecognizedGesture::DoubleTap(DoubleTapEvent { fingers: 1 }))
        );

        assert_eq!(arbitrator.flush(start + Duration::from_millis(300)), None);
    }

    #[test]
    fn double_tap_is_recognized_at_exact_boundary() {
        let mut arbitrator = GestureArbitrator::new();
        let start = Instant::now();

        arbitrator.process(&frame(vec![down(1, 0.5, 0.5)]), start);

        arbitrator.process(&frame(vec![up(1, 0.5, 0.5)]), start);

        let second_tap = start + Duration::from_millis(300);

        arbitrator.process(&frame(vec![down(2, 0.5, 0.5)]), second_tap);

        assert_eq!(
            arbitrator.process(&frame(vec![up(2, 0.5, 0.5)]), second_tap,),
            Some(RecognizedGesture::DoubleTap(DoubleTapEvent { fingers: 1 }))
        );
    }

    #[test]
    fn tap_after_timeout_becomes_new_pending_tap() {
        let mut arbitrator = GestureArbitrator::new();
        let start = Instant::now();

        arbitrator.process(&frame(vec![down(1, 0.5, 0.5)]), start);

        arbitrator.process(&frame(vec![up(1, 0.5, 0.5)]), start);

        let after_timeout = start + Duration::from_millis(301);

        assert_eq!(
            arbitrator.process(&frame(vec![down(2, 0.5, 0.5)]), after_timeout,),
            Some(RecognizedGesture::Tap(TapEvent { fingers: 1 }))
        );
    }

    #[test]
    fn second_tap_after_timeout_does_not_become_double_tap() {
        let mut arbitrator = GestureArbitrator::new();
        let start = Instant::now();

        arbitrator.process(&frame(vec![down(1, 0.5, 0.5)]), start);

        arbitrator.process(&frame(vec![up(1, 0.5, 0.5)]), start);

        let second = start + Duration::from_millis(301);

        arbitrator.process(&frame(vec![down(2, 0.5, 0.5)]), second);

        assert_eq!(
            arbitrator.process(&frame(vec![up(2, 0.5, 0.5)]), second,),
            None
        );

        assert_eq!(
            arbitrator.flush(second + Duration::from_millis(300)),
            Some(RecognizedGesture::Tap(TapEvent { fingers: 1 }))
        );
    }

    #[test]
    fn two_finger_single_tap_is_deferred() {
        let mut arbitrator = GestureArbitrator::new();
        let start = Instant::now();

        arbitrator.process(&frame(vec![down(1, 0.4, 0.5), down(2, 0.6, 0.5)]), start);

        assert_eq!(
            arbitrator.process(&frame(vec![up(1, 0.4, 0.5), up(2, 0.6, 0.5),]), start,),
            None
        );

        assert_eq!(
            arbitrator.flush(start + Duration::from_millis(300)),
            Some(RecognizedGesture::Tap(TapEvent { fingers: 2 }))
        );
    }

    #[test]
    fn two_finger_double_tap_replaces_pending_tap() {
        let mut arbitrator = GestureArbitrator::new();
        let start = Instant::now();

        arbitrator.process(&frame(vec![down(1, 0.4, 0.5), down(2, 0.6, 0.5)]), start);

        arbitrator.process(&frame(vec![up(1, 0.4, 0.5), up(2, 0.6, 0.5)]), start);

        let second = start + Duration::from_millis(100);

        arbitrator.process(&frame(vec![down(3, 0.4, 0.5), down(4, 0.6, 0.5)]), second);

        assert_eq!(
            arbitrator.process(&frame(vec![up(3, 0.4, 0.5), up(4, 0.6, 0.5),]), second,),
            Some(RecognizedGesture::DoubleTap(DoubleTapEvent { fingers: 2 }))
        );
    }

    #[test]
    fn swipe_clears_pending_single_tap() {
        let mut arbitrator = GestureArbitrator::new();
        let start = Instant::now();

        arbitrator.process(&frame(vec![down(1, 0.3, 0.5)]), start);

        arbitrator.process(&frame(vec![up(1, 0.3, 0.5)]), start);

        let swipe_start = start + Duration::from_millis(50);

        arbitrator.process(&frame(vec![down(2, 0.3, 0.5)]), swipe_start);

        assert_eq!(
            arbitrator.process(&frame(vec![move_touch(2, 0.5, 0.5)]), swipe_start,),
            Some(RecognizedGesture::OneFingerSwipe(OneFingerSwipeEvent {
                direction: Direction::Right,
            }))
        );

        assert_eq!(arbitrator.flush(start + Duration::from_millis(300)), None);
    }

    #[test]
    fn three_finger_swipe_clears_pending_single_tap() {
        let mut arbitrator = GestureArbitrator::new();
        let start = Instant::now();

        arbitrator.process(&frame(vec![down(1, 0.5, 0.5)]), start);

        arbitrator.process(&frame(vec![up(1, 0.5, 0.5)]), start);

        let gesture_start = start + Duration::from_millis(50);

        // Three-finger gesture starts with DOWN.
        assert_eq!(
            arbitrator.process(
                &frame(vec![
                    down(2, 0.10, 0.30),
                    down(3, 0.10, 0.50),
                    down(4, 0.10, 0.70),
                ]),
                gesture_start,
            ),
            None
        );

        // Movement is evaluated against the DOWN positions.
        assert_eq!(
            arbitrator.process(
                &frame(vec![
                    move_touch(2, 0.40, 0.30),
                    move_touch(3, 0.45, 0.50),
                    move_touch(4, 0.50, 0.70),
                ]),
                gesture_start,
            ),
            Some(RecognizedGesture::ThreeFingerSwipe(GestureEvent {
                gesture: Gesture::Swipe {
                    fingers: 3,
                    direction: Direction::Right,
                },
                phase: GesturePhase::Started,
            }))
        );

        assert_eq!(arbitrator.flush(start + Duration::from_millis(300)), None);
    }

    #[test]
    fn flush_is_idempotent_after_pending_tap_is_emitted() {
        let mut arbitrator = GestureArbitrator::new();
        let start = Instant::now();

        arbitrator.process(&frame(vec![down(1, 0.5, 0.5)]), start);

        arbitrator.process(&frame(vec![up(1, 0.5, 0.5)]), start);

        let timeout = start + Duration::from_millis(300);

        assert_eq!(
            arbitrator.flush(timeout),
            Some(RecognizedGesture::Tap(TapEvent { fingers: 1 }))
        );

        assert_eq!(arbitrator.flush(timeout), None);
    }
}
