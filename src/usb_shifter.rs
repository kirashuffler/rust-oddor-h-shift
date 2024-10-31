use std::time::Duration;

use rusb::{Context, Device, UsbContext, DeviceDescriptor, DeviceHandle, Direction};

use crate::constants::{VENDOR_ID, PRODUCT_ID};
use crate::errors::AppError;


#[derive(Debug)]
pub struct UsbShifter {
    context: Context,
    pub device: Device<Context>,
    pub descriptor: DeviceDescriptor,
}

#[derive(Debug)]
pub struct UsbShifterHandle {
    raw: DeviceHandle<Context>,
    pub endpoint: Endpoint,
}


#[derive(Clone, Debug)]
pub struct Endpoint {
    pub address: u8,
    pub config: u8,
    pub interface: u8,
    pub polling_interval: Duration,
    pub setting: u8,
}


#[derive(Clone, Debug)]
pub struct UsbShifterState {
    pub range: bool,
    pub splitter: bool,
    pub extra: bool,
}

impl PartialEq for UsbShifterState {
    fn eq(&self, other: &Self) -> bool {
        self.range == other.range && self.splitter == other.splitter && self.extra == other.extra
    }
}
impl Eq for UsbShifterState {}


impl UsbShifter {
    pub fn new() -> Result<UsbShifter, AppError> {
        let libusb_context = match Context::new() {
            Ok(ctx) => ctx,
            Err(_) => return Err(AppError::from("Could not initialise libusb context.")),
        };

        let devices = match libusb_context.devices() {
            Ok(dev) => dev,
            Err(_) => return Err(AppError::from("Could not list USB devices.")),
        };

        for device in devices.iter() {
            let device_descriptor = match device.device_descriptor() {
                Ok(desc) => desc,
                Err(_) => continue,
            };

            if device_descriptor.vendor_id() != VENDOR_ID || device_descriptor.product_id() != PRODUCT_ID {
                continue;
            };

            println!(
                "Found a matching device on bus = {}, address = {}, port number = {}",
                device.bus_number(),
                device.address(),
                device.port_number(),
            );

            return Ok(UsbShifter {
                context: libusb_context,
                device,
                descriptor: device_descriptor,
            })
        }

        Err(AppError::from("No matching USB devices found."))
    }

    pub fn open(&self, endpoint: &Endpoint) -> Result<UsbShifterHandle, AppError> {
        match self.device.open() {
            Ok(handle) => Ok(
                UsbShifterHandle::new(handle, endpoint.clone())?
            ),
            Err(e) => Err(AppError::from(format!("Could not open USB device: {e}")))
        }
    }

    /// There should be only one IN endpoint on the shifter.
    pub fn get_readable_endpoint(&self) -> Result<Endpoint, AppError> {
        for n in 0..self.descriptor.num_configurations() {
            let config_desc = match self.device.config_descriptor(n) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for interface in config_desc.interfaces() {
                for interface_desc in interface.descriptors() {
                    for endpoint_desc in interface_desc.endpoint_descriptors() {
                        if endpoint_desc.direction() == Direction::In {
                            return Ok(Endpoint {
                                address: endpoint_desc.address(),
                                config: config_desc.number(),
                                interface: interface_desc.interface_number(),
                                polling_interval: Duration::from_millis(u64::from(endpoint_desc.interval())),
                                setting: interface_desc.setting_number(),
                            });
                        }
                    }
                }
            }
        }

        Err(AppError::from("Could not find a readable endpoint on the USB device"))
    }

    pub fn has_hotplug(&self) -> bool {
        rusb::has_hotplug()
    }
}


impl UsbShifterHandle {
    pub fn new(raw: DeviceHandle<Context>, endpoint: Endpoint) -> Result<UsbShifterHandle, AppError> {
        let handle = UsbShifterHandle { raw, endpoint };
        handle.configure()?;
        Ok(handle)
    }

    fn configure(&self) -> Result<(), AppError> {
        self.raw.reset().map_err(|e| AppError::from(format!("Error resetting device: {e}")))?;
        self.raw.set_active_configuration(self.endpoint.config).map_err(|e| AppError::from(format!("Error setting configuration: {e}")))?;
        self.raw.claim_interface(self.endpoint.interface).map_err(|e| AppError::from(format!("Error claiming interface: {e}")))?;
        self.raw.set_alternate_setting(self.endpoint.interface, self.endpoint.setting).map_err(|e| AppError::from(format!("Error setting alternate setting: {e}")))?;
        Ok(())
    }

    pub fn read(&self) -> Result<UsbShifterState, AppError> {
        let mut buffer: [u8; 64] = [0; 64];
        let timeout = Duration::from_millis(500);

        let raw_data: &[u8] = match self.raw.read_interrupt(self.endpoint.address, &mut buffer, timeout) {
            Ok(len) => &buffer[..len],
            Err(e) => return Err(AppError::from(format!("Warning: could not read from device endpoint: {e}"))),
        };

        // looks like this reads 5 bytes, but only the first one is actually used
        let data = raw_data[0];

        Ok(UsbShifterState {
            range: (data & 0x1) != 0,
            splitter: (data & 0x2) != 0,
            extra: (data & 0x4) != 0,
        })
    }
}
