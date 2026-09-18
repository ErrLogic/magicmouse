use std::env;
use std::process::ExitCode;

use magic_linux::{InputReader, TapRecognizer, TouchTracker, discover_magic_mice};

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

    println!("Magic Mouse tap input debugger");
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
    println!("Listening for tap input...");
    println!();

    let mut touch_tracker = TouchTracker::new();
    let mut tap_recognizer = TapRecognizer::new();

    loop {
        let events = match reader.fetch_events() {
            Ok(events) => events,
            Err(error) => {
                eprintln!("Failed to read input events: {error}");
                return ExitCode::FAILURE;
            }
        };

        for event in events {
            let Some(frame) = touch_tracker.process_event(event) else {
                continue;
            };

            let normalized = frame.normalize(&bounds);

            if let Some(tap) = tap_recognizer.process(&normalized) {
                println!(
                    "TAP | {} finger{}",
                    tap.fingers,
                    if tap.fingers == 1 { "" } else { "s" }
                );
            }
        }
    }
}
