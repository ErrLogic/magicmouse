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

    println!("Magic Mouse touch debugger");
    println!("Device: {device_path}");
    println!("Press Ctrl+C to stop.");
    println!();

    let mut reader = match InputReader::open(&device_path) {
        Ok(reader) => reader,
        Err(error) => {
            eprintln!("Failed to open {device_path}: {error}");
            return ExitCode::FAILURE;
        }
    };

    println!("Name: {}", reader.name().unwrap_or("<unknown>"));
    println!();

    let mut tracker = TouchTracker::new();

    loop {
        match reader.fetch_events() {
            Ok(events) => {
                for event in events {
                    if let Some(frame) = tracker.process_event(event) {
                        println!(
                            "FRAME | touches={} active={}",
                            frame.len(),
                            tracker.active_touch_count()
                        );

                        for touch in frame.touches {
                            println!(
                                "  slot={:<2} tracking={:<6?} x={:<6} y={:<6} state={:?}",
                                touch.slot, touch.tracking_id, touch.x, touch.y, touch.state
                            );
                        }

                        println!();
                    }
                }
            }

            Err(error) => {
                eprintln!("Failed to read input events: {error}");
                return ExitCode::FAILURE;
            }
        }
    }
}
