use std::env;
use std::process::ExitCode;

use magic_linux::{InputReader, TouchTracker, discover_magic_mice};

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

    println!("Magic Mouse normalized input debugger");
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
    println!("Listening for input...");
    println!();

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
            if let Some(frame) = tracker.process_event(event) {
                let normalized = frame.normalize(&bounds);

                println!("FRAME | touches={}", frame.len());
                println!();

                println!("  #  slot  tracking  state  raw              normalized");

                for (index, (raw, normalized)) in frame
                    .touches
                    .iter()
                    .zip(normalized.touches.iter())
                    .enumerate()
                {
                    let tracking = raw
                        .tracking_id
                        .map_or_else(|| "-".to_string(), |id| id.to_string());

                    println!(
                        "  {index:<2} {slot:<5} {tracking:<9} {state:<6} ({x:>5}, {y:>5})     ({nx:.4}, {ny:.4})",
                        slot = raw.slot,
                        state = format!("{:?}", normalized.state),
                        x = raw.x,
                        y = raw.y,
                        nx = normalized.position.x,
                        ny = normalized.position.y,
                    );
                }

                println!();
            }
        }
    }
}
