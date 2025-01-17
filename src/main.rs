use std::fs;

use evdev::{Device, InputEventKind, Key};

fn main() {
    let inputs = fs::read_dir("/dev/input").unwrap();

    let keyboard_path = inputs
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| {
            if let Ok(device) = Device::open(path) {
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

    let mut keyboard = Device::open(keyboard_path).unwrap();
    println!(
        "Listening for key events on: {}",
        keyboard.name().unwrap_or("Unknown Device")
    );

    loop {
        for ev in keyboard.fetch_events().unwrap() {
            if let InputEventKind::Key(key) = ev.kind() {
                if ev.value() == 1 {}
            }
        }
    }
}

fn to_letter(key: Key) -> char {
    match key {
        _ => {}
    }
    todo!()
}

struct Log {}
