//! magic-linux
pub mod action;
pub mod axis;
pub mod codes;
pub mod discovery;
pub mod gesture;
pub mod input;
pub mod normalize;
pub mod pointer;
pub mod scroll;
pub mod touch;

pub use touch::{Touch, TouchFrame, TouchState, TouchTracker};

pub use codes::{event_code_name, event_type_name};

pub use discovery::{DiscoveryError, InputDeviceInfo, discover_input_devices, discover_magic_mice};

pub use input::{InputReader, RawInputEvent};

pub use normalize::{NormalizedPoint, NormalizedTouch, NormalizedTouchFrame};

pub use axis::{AxisBounds, TouchBounds};

pub use pointer::{PointerEvent, PointerSensitivity, PointerTracker};

pub use scroll::{ScrollEvent, ScrollSensitivity, ScrollTracker};
