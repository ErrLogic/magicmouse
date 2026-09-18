use std::env;
use std::process::ExitCode;

use magic_linux::{
    InputReader, TouchTracker, discover_magic_mice,
    gesture::{GestureEvent, GestureRecognizer, GestureState, MovementThreshold, TrackedTouch},
    normalize::NormalizedTouchFrame,
    touch::TouchState,
};

const GESTURE_THRESHOLD: f32 = 0.15;

fn main() -> ExitCode {
    let device_path = match env::args().nth(1) {
        Some(path) => path,

        None => match discover_magic_mice() {
            Ok(devices) if devices.len() == 1 => devices[0].path.to_string_lossy().into_owned(),

            Ok(devices) if devices.is_empty() => {
                eprintln!("No Magic Mouse devices found.");
                return ExitCode::FAILURE;
            }

            Ok(devices) => {
                eprintln!(
                    "Multiple Magic Mouse devices found. Specify the device path explicitly:"
                );

                for device in devices {
                    eprintln!("  {}", device.path.display());
                }

                return ExitCode::FAILURE;
            }

            Err(error) => {
                eprintln!("Failed to discover Magic Mouse: {error}");
                return ExitCode::FAILURE;
            }
        },
    };

    println!("Magic Mouse gesture debugger");
    println!("Device: {device_path}");
    println!("Press Ctrl+C to stop.");
    println!();

    let mut reader = match InputReader::open(&device_path) {
        Ok(reader) => reader,

        Err(error) => {
            eprintln!("Failed to open {device_path}: {error}");
            eprintln!();
            eprintln!("The current user may need access to the 'input' group.");

            return ExitCode::FAILURE;
        }
    };

    println!("Name: {}", reader.name().unwrap_or("<unknown>"));

    let bounds = match reader.touch_bounds() {
        Some(bounds) => bounds,

        None => {
            eprintln!("Failed to read Magic Mouse touch bounds.");
            return ExitCode::FAILURE;
        }
    };

    println!("X bounds: {} .. {}", bounds.x.min, bounds.x.max);
    println!("Y bounds: {} .. {}", bounds.y.min, bounds.y.max);

    println!();
    println!("Touch bounds loaded successfully.");
    println!("Gesture threshold: {GESTURE_THRESHOLD:.4}");
    println!("Listening for gestures...");
    println!();

    let threshold = MovementThreshold::new(GESTURE_THRESHOLD);

    let mut recognizer = GestureRecognizer::new(threshold);
    let mut gesture_state = GestureState::new();
    let mut tracker = TouchTracker::new();

    loop {
        let events = match reader.fetch_events() {
            Ok(events) => events,

            Err(error) => {
                eprintln!("Failed to read input events: {error}");
                return ExitCode::FAILURE;
            }
        };

        for event in events {
            let Some(frame) = tracker.process_event(event) else {
                continue;
            };

            let normalized = frame.normalize(&bounds);

            print_frame(&normalized);

            update_gesture_state(&mut gesture_state, &normalized);

            println!(
                "STATE  | fingers={} | empty={}",
                gesture_state.finger_count(),
                gesture_state.is_empty()
            );

            for touch in gesture_state.touches() {
                let movement = touch.movement();

                println!(
                    "TRACK  | slot={:?} | id={} | start=({:.4}, {:.4}) | current=({:.4}, {:.4}) | movement=({:.4}, {:.4}) | magnitude={:.4} | direction={:?}",
                    touch.slot,
                    touch.tracking_id,
                    touch.start.x,
                    touch.start.y,
                    touch.current.x,
                    touch.current.y,
                    movement.dx,
                    movement.dy,
                    movement.magnitude(),
                    movement.direction(),
                );
            }

            if let Some(event) = recognizer.process(&gesture_state) {
                print_gesture_event(event);
            }

            println!();
        }
    }
}

fn update_gesture_state(state: &mut GestureState, frame: &NormalizedTouchFrame) {
    for touch in &frame.touches {
        match touch.state {
            TouchState::Down => {
                let Some(tracking_id) = touch.tracking_id else {
                    continue;
                };

                state.add_touch(TrackedTouch::with_slot(
                    touch.slot,
                    tracking_id,
                    touch.position,
                ));
            }

            TouchState::Move => {
                let Some(tracking_id) = touch.tracking_id else {
                    continue;
                };

                state.update_touch(tracking_id, touch.position);
            }

            TouchState::Up => {
                state.remove_touch_by_slot(touch.slot);
            }
        }
    }
}

fn print_frame(frame: &NormalizedTouchFrame) {
    println!("FRAME  | touches={}", frame.len());

    for touch in &frame.touches {
        println!(
            "TOUCH  | slot={} | tracking={:?} | state={:?} | position=({:.4}, {:.4})",
            touch.slot, touch.tracking_id, touch.state, touch.position.x, touch.position.y,
        );
    }
}

fn print_gesture_event(event: GestureEvent) {
    println!(
        "GESTURE | phase={:<9} | {:?}",
        format!("{:?}", event.phase),
        event.gesture
    );
}
