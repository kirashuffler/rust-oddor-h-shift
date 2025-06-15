use std::{process::exit, thread, time::Duration};

use constants::{MAX_GEAR, PRODUCT_ID, VENDOR_ID};
use crossbeam::{
    channel::{bounded, select, tick, Receiver, Sender},
    thread::{Scope, ScopedJoinHandle},
};
use evdev::{EventType, InputEvent, Key};
use rusb::{Context, Device, HotplugBuilder, Registration, UsbContext};
use signal_hook::{consts::SIGINT, consts::SIGTERM, iterator::Signals};

mod constants;

mod errors;
use errors::AppError;

mod usb_shifter;
use usb_shifter::UsbShifter;

mod shifter;
use shifter::EventDevice;

struct HotPlugHandler {
    sender: Sender<HotplugMessage>,
}
#[derive(Debug)]
enum HotplugMessage {
    Arrived(u8, u8),
    Left(u8, u8),
}

impl<T: UsbContext> rusb::Hotplug<T> for HotPlugHandler {
    fn device_arrived(&mut self, device: Device<T>) {
        self.sender
            .send(HotplugMessage::Arrived(
                device.bus_number(),
                device.address(),
            ))
            .expect("Internal error: Could not send a crossbeam message.");
    }

    fn device_left(&mut self, device: Device<T>) {
        self.sender
            .send(HotplugMessage::Left(device.bus_number(), device.address()))
            .expect("Internal error: Could not send a crossbeam message.");
    }
}

fn hotplug_handler(sender: Sender<HotplugMessage>, receiver: Receiver<()>) -> Result<(), AppError> {
    if !UsbShifter::has_hotplug() {
        return Ok(());
    }

    let context =
        Context::new().map_err(|e| AppError::from(format!("Error creating context: {e}")))?;
    let _handler: Option<Registration<Context>> = Some(
        HotplugBuilder::new()
            .vendor_id(VENDOR_ID)
            .product_id(PRODUCT_ID)
            .enumerate(true)
            .register(
                &context,
                Box::new(HotPlugHandler {
                    sender: sender.clone(),
                }),
            )
            .map_err(|e| AppError::from(format!("Error registering hotplug callbacks: {e}")))?,
    );

    loop {
        select! {
            recv(receiver) -> _ => {
                return Ok(())
            }
            default => {
                context.handle_events(Some(Duration::from_secs(1))).unwrap();
            }
        }
    }
}

fn shifter_reader(receiver: Receiver<()>) -> Result<(), AppError> {
    let usb_device = UsbShifter::new()?;
    let endpoint = usb_device.get_readable_endpoint()?;
    let usb_handle = usb_device.open(&endpoint)?;
    let mut evdev_device = EventDevice::new()?;

    println!("Starting to read USB shifter states.");
    let mut input_state = usb_handle.read()?;
    let polling_tick = tick(usb_handle.endpoint.polling_interval);
    //let polling_tick = tick(Duration::from_millis(50));

    loop {
        select! {
            recv(polling_tick) -> _ => {
                let read_result = usb_handle.read();
                let mut events = Vec::new();
                if read_result.is_err() {
                    if UsbShifter::has_hotplug() {
                        // we politely wait for the message to shut down
                        continue
                    } else {
                        // otherwise, bail out
                        return Err(AppError::from(read_result.unwrap_err().message));
                    }
                }
                let new_state = read_result.as_ref().unwrap();

                if input_state != *new_state {
                    // Release previos button
                    events.push(InputEvent::new(EventType::KEY, Key::BTN_TRIGGER_HAPPY1.code() + input_state, 0));
                    input_state = new_state.clone();
                    if input_state <= MAX_GEAR {
                        events.push(InputEvent::new(EventType::KEY, Key::BTN_TRIGGER_HAPPY1.code() + input_state, 1));
                    }
                    evdev_device.emit(&events).map_err(|e| AppError::from(format!("Could not emit a device event: {e}")))?;
                    // TODO: print states only if verbose argv provided
                    // println!("state = {:?}", input_state);
                };
            }
            recv(receiver) -> _ => {
                break
            }
        }
    }

    println!("Reader thread shutting down.");
    Ok(())
}

fn main_loop(
    s: &Scope,
    signal_receiver: Receiver<()>,
    hotplug_sender: Sender<()>,
    hotplug_receiver: Receiver<HotplugMessage>,
) -> Result<(), AppError> {
    let (main2reader_sender, main2reader_receiver) = bounded(100);
    let mut thread_join_handle: Option<ScopedJoinHandle<()>> = None;

    if !UsbShifter::has_hotplug() {
        println!("Warning: your libusb doesn't support hotplug.");
        let sig = main2reader_receiver.clone();
        thread_join_handle = Some(s.spawn(move |_| {
            let res = shifter_reader(sig);

            if res.is_err() {
                eprintln!("Fatal error: {}", res.unwrap_err());
                exit(1);
            }
        }));
    }

    loop {
        select! {
            recv(hotplug_receiver) -> mess => {
                match mess.as_ref().unwrap() {
                    HotplugMessage::Arrived(bus, address) => {
                        println!("Using device on bus {bus}, address {address}.");
                        let sig = main2reader_receiver.clone();
                        thread_join_handle = Some(s.spawn(move |_| {
                            shifter_reader(sig).unwrap();
                        }));
                    },
                    HotplugMessage::Left(bus, address) => {
                        println!("Device on bus {bus}, address {address} gone.");
                        if let Some(thread_join_handle) = thread_join_handle.take() {
                            main2reader_sender.send(()).unwrap();
                            thread_join_handle.join().unwrap();
                        };
                    },
                }
            }
            recv(signal_receiver) -> _ => {
                if UsbShifter::has_hotplug() {
                    hotplug_sender.send(()).unwrap();
                }
                main2reader_sender.send(()).unwrap();
                break
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), AppError> {
    let mut signals = Signals::new([SIGINT, SIGTERM])
        .map_err(|e| AppError::from(format!("Internal error: {e}")))?;
    let (signal_sender, signal_receiver) = bounded(5);
    thread::spawn(move || {
        for sig in signals.forever() {
            println!("Received signal {:?}, shutting down.", sig);
            let _ = signal_sender.send(());
        }
    });

    let (hotplug2main_sender, hotplug2main_receiver) = bounded(5);
    let (main2hotplug_sender, main2hotplug_receiver) = bounded(5);

    crossbeam::scope(|s| {
        s.spawn(|_| {
            hotplug_handler(hotplug2main_sender.clone(), main2hotplug_receiver.clone()).unwrap();
        });
        s.spawn(|s| {
            main_loop(
                s,
                signal_receiver,
                main2hotplug_sender.clone(),
                hotplug2main_receiver.clone(),
            )
            .unwrap();
        });
    })
    .unwrap();

    Ok(())
}
