use crate::axis::{AxisBounds, TouchBounds};
use evdev::Device;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawInputEvent {
    pub event_type: u16,
    pub event_code: u16,
    pub value: i32,
}

pub struct InputReader {
    device: Device,
}

impl InputReader {
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let device = Device::open(path)?;

        Ok(Self { device })
    }

    pub fn name(&self) -> Option<&str> {
        self.device.name()
    }

    pub fn fetch_events(&mut self) -> std::io::Result<Vec<RawInputEvent>> {
        let events = self.device.fetch_events()?;

        Ok(events
            .map(|event| RawInputEvent {
                event_type: event.event_type().0,
                event_code: event.code(),
                value: event.value(),
            })
            .collect())
    }

    pub fn abs_axis_bounds(&self, code: u16) -> Option<AxisBounds> {
        self.device
            .get_absinfo()
            .ok()?
            .find(|(axis, _)| axis.0 == code)
            .map(|(_, info)| AxisBounds::new(info.minimum(), info.maximum()))
    }

    pub fn touch_bounds(&self) -> Option<TouchBounds> {
        let x = self.abs_axis_bounds(53)?;
        let y = self.abs_axis_bounds(54)?;

        Some(TouchBounds::new(x, y))
    }
}
