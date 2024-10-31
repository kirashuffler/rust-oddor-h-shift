use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, BusType, InputEvent, InputId, Key, PropType};
use input::{Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use std::fs::{File, OpenOptions};
use std::os::fd::OwnedFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::constants::{PRODUCT_ID, VENDOR_ID};
use crate::errors::AppError;

pub struct EventDevice {
    pub raw: VirtualDevice,
    #[allow(dead_code)]
    libinput_device: Libinput, // we just need this to exist so the event device is visible in libinput
}

struct UdevInterface;

impl LibinputInterface for UdevInterface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: OwnedFd) {
        let _ = File::from(fd);
    }
}

impl EventDevice {
    pub fn new() -> Result<EventDevice, AppError> {
        let mut device = match Self::build() {
            Ok(device) => device,
            Err(e) => {
                return Err(AppError::from(format!(
                    "Could not create a new evdev device: {e}"
                )))
            }
        };

        let path = match device.enumerate_dev_nodes_blocking() {
            Ok(mut path_list) => path_list.nth(0).unwrap().unwrap(),
            Err(e) => return Err(AppError::from(format!("Could not get device path: {e}"))),
        };

        // this seems to be necessary to allow udev enough time to set the correct permissions,
        // otherwise adding the device to libinput fails silently
        thread::sleep(Duration::from_millis(100));

        let mut libinput_device = Libinput::new_from_path(UdevInterface);
        libinput_device.path_add_device(&path.to_str().unwrap());
        println!("Virtual device available at {:?}", path);

        Ok(EventDevice {
            raw: device,
            libinput_device,
        })
    }

    fn build() -> std::io::Result<VirtualDevice> {
        let input_id = InputId::new(BusType::BUS_USB, VENDOR_ID, PRODUCT_ID, 1);
        let mut keys = AttributeSet::<Key>::new();
        let mut props = AttributeSet::<PropType>::new();

        props.insert(PropType::BUTTONPAD);
        keys.insert(Key::BTN_MODE);
        keys.insert(Key::BTN_GEAR_UP);
        keys.insert(Key::BTN_EXTRA);

        let device = VirtualDeviceBuilder::new()?
            .name("Labtec ODDOR-TRUCKSHIFT")
            .input_id(input_id)
            .with_properties(&props)?
            .with_keys(&keys)?
            .build()
            .unwrap();

        Ok(device)
    }

    pub fn emit(&mut self, messages: &[InputEvent]) -> Result<(), std::io::Error> {
        self.raw.emit(messages)
    }
}
