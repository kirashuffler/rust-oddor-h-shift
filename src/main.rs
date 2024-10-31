use std::thread;

use crossbeam_channel::{bounded, select, tick};
use evdev::{EventType, InputEvent, Key};
use signal_hook::{consts::SIGINT, consts::SIGTERM, iterator::Signals};

mod constants;

mod errors;
use errors::AppError;

mod usb_shifter;
use usb_shifter::UsbShifter;

mod shifter;
use shifter::EventDevice;

fn main() -> Result<(), AppError> {
    let mut signals = Signals::new([SIGINT, SIGTERM])
        .map_err(|e| AppError::from(format!("Internal error: {e}")))?;
    let (signal_sender, signal_receiver) = bounded(100);
    thread::spawn(move || {
        for sig in signals.forever() {
            println!("Received signal {:?}", sig);
            let _ = signal_sender.send(());
        }
    });

    let usb_device = UsbShifter::new()?;
    let endpoint = usb_device.get_readable_endpoint()?;
    let usb_handle = usb_device.open(&endpoint)?;

    let mut evdev_device = EventDevice::new()?;

    println!("Starting to read USB shifter states");
    let mut input_state = usb_handle.read()?;
    let polling_tick = tick(usb_handle.endpoint.polling_interval);

    loop {
        select! {
            recv(polling_tick) -> _ => {
                let new_state = usb_handle.read()?;

                if input_state != new_state {
                    input_state = new_state.clone();

                    let events = [
                        InputEvent::new(EventType::KEY, Key::BTN_MODE.code(), input_state.range.into()),
                        InputEvent::new(EventType::KEY, Key::BTN_GEAR_UP.code(), input_state.splitter.into()),
                        InputEvent::new(EventType::KEY, Key::BTN_EXTRA.code(), input_state.extra.into()),
                    ];
                    evdev_device.emit(&events).map_err(|e| AppError::from(format!("Could not emit a device event: {e}")))?;
                    // TODO: print states only if verbose argv provided
                    println!("state = {:?}", input_state);
                };
            }
            recv(signal_receiver) -> _ => {
                break
            }
        }
    }

    Ok(())
}
