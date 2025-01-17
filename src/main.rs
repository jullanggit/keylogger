use std::fs;

use evdev_rs::{
    Device, DeviceWrapper, InputEvent, ReadFlag,
    enums::{EV_KEY::*, EventCode},
};

fn main() {
    let inputs = fs::read_dir("/dev/input").unwrap();

    let keyboard_path = inputs
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| {
            if let Ok(device) = Device::new_from_path(path) {
                if let Some(name) = device.name() {
                    name.contains("evremap")
                } else {
                    false
                }
            } else {
                false
            }
        })
        .expect("Failed to find keyboard input device");

    let mut keyboard = Device::new_from_path(keyboard_path).unwrap();
    println!(
        "Listening for key events on: {}",
        keyboard.name().unwrap_or("Unknown Device")
    );

    loop {
        let event = keyboard
            .next_event(ReadFlag::NORMAL)
            .map(|(_status, event)| event);

        match event {
            Ok(event) => if let EventCode::EV_KEY(key) = event.event_code {},

            Err(e) => eprintln!("{e}"),
        }
    }
}

// fn to_letter(key: Key) -> char {
//     match key {
//         _ => {}
//     }
//     todo!()
// }

struct Log {}
