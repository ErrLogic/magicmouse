use std::path::PathBuf;

use udev::{Device, Enumerator};

const APPLE_VENDOR_ID: u16 = 0x004c;
const MAGIC_MOUSE_PRODUCT_ID: u16 = 0x0323;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputDeviceInfo {
    pub name: Option<String>,
    pub path: PathBuf,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub bus: Option<String>,
    pub driver: Option<String>,
    pub phys: Option<String>,
    pub uniq: Option<String>,
}

impl InputDeviceInfo {
    pub fn is_magic_mouse(&self) -> bool {
        self.vendor_id == Some(APPLE_VENDOR_ID) && self.product_id == Some(MAGIC_MOUSE_PRODUCT_ID)
    }
}

#[derive(Debug)]
pub enum DiscoveryError {
    Io(std::io::Error),
}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
        }
    }
}

impl std::error::Error for DiscoveryError {}

impl From<std::io::Error> for DiscoveryError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn discover_input_devices() -> Result<Vec<InputDeviceInfo>, DiscoveryError> {
    let mut enumerator = Enumerator::new()?;

    enumerator.match_subsystem("input")?;

    let mut devices = Vec::new();

    for device in enumerator.scan_devices()? {
        let Some(devnode) = device.devnode() else {
            continue;
        };

        let name = read_value(&device, "name");
        let phys = read_value(&device, "phys");
        let uniq = read_value(&device, "uniq");

        let hid_device = find_hid_parent(&device);

        let (vendor_id, product_id, bus, driver) = match hid_device {
            Some(hid) => (
                parse_hid_vendor(&hid),
                parse_hid_product(&hid),
                parse_hid_bus(&hid),
                read_driver_name(&hid),
            ),
            None => (None, None, None, None),
        };

        devices.push(InputDeviceInfo {
            name,
            path: devnode.to_path_buf(),
            vendor_id,
            product_id,
            bus,
            driver,
            phys,
            uniq,
        });
    }

    Ok(devices)
}

pub fn discover_magic_mice() -> Result<Vec<InputDeviceInfo>, DiscoveryError> {
    Ok(discover_input_devices()?
        .into_iter()
        .filter(|device| {
            device.vendor_id == Some(0x004c)
                && device.product_id == Some(0x0323)
                && is_event_device(&device.path)
        })
        .collect())
}

fn is_event_device(path: &std::path::Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("event"))
}

fn read_value(device: &Device, attribute: &str) -> Option<String> {
    device
        .attribute_value(attribute)
        .map(|value| value.to_string_lossy().into_owned())
}

fn find_hid_parent(device: &Device) -> Option<Device> {
    let mut current = device.parent()?;

    loop {
        if current.subsystem().is_some_and(|s| s == "hid") {
            return Some(current);
        }

        current = current.parent()?;
    }
}

fn read_driver_name(device: &Device) -> Option<String> {
    device
        .driver()
        .map(|driver| driver.to_string_lossy().into_owned())
}

fn parse_hid_vendor(device: &Device) -> Option<u16> {
    let modalias = read_value(device, "modalias")?;

    parse_modalias_field(&modalias, 'v')
}

fn parse_hid_product(device: &Device) -> Option<u16> {
    let modalias = read_value(device, "modalias")?;

    parse_modalias_field(&modalias, 'p')
}

fn parse_hid_bus(device: &Device) -> Option<String> {
    let modalias = read_value(device, "modalias")?;

    let bus = modalias.strip_prefix("hid:b")?.get(..4)?;

    Some(bus.to_string())
}

fn parse_modalias_field(modalias: &str, field: char) -> Option<u16> {
    let marker = format!("{field}");

    let start = modalias.find(&marker)? + marker.len();

    let hex = modalias
        .get(start..)?
        .chars()
        .take_while(|character| character.is_ascii_hexdigit())
        .collect::<String>();

    if hex.is_empty() {
        return None;
    }

    u16::from_str_radix(&hex, 16).ok()
}
