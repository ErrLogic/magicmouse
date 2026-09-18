use std::env;
use std::process::ExitCode;

use magic_linux::{
    ButtonPhase, InputReader, PhysicalButtonTracker, discover_magic_mice, event_code_name,
};

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

    println!("Magic Mouse button input debugger");
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
    println!();
    println!("Listening for physical button input...");
    println!();

    let mut button_tracker = PhysicalButtonTracker::new();

    loop {
        let events = match reader.fetch_events() {
            Ok(events) => events,
            Err(error) => {
                eprintln!("Failed to read input events: {error}");
                return ExitCode::FAILURE;
            }
        };

        for event in events {
            let Some(button_event) = button_tracker.process(event) else {
                continue;
            };

            let phase = match button_event.phase {
                ButtonPhase::Pressed => "PRESSED",
                ButtonPhase::Released => "RELEASED",
            };

            println!(
                "BUTTON | {} | {:?} | {}",
                event_code_name(event.event_type, event.event_code),
                button_event.button,
                phase,
            );
        }
    }
}
