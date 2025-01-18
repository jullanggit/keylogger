#![feature(let_chains)]

use std::fs;

use evdev_rs::{
    Device,
    DeviceWrapper,
    ReadFlag, // enum*, EventCde},
    enums::EventCode,
};
use xkbcommon::xkb::{Context, KeyDirection, Keymap, State};

fn get_keyboard() -> Device {
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

    Device::new_from_path(keyboard_path).unwrap()
}

fn init_xkbcommon() -> State {
    let context = Context::new(0);
    let keymap = Keymap::new_from_names(&context, "", "pc105", "ch", "de", None, 0).unwrap();
    State::new(&keymap)
}

fn main() {
    let keyboard = get_keyboard();
    let mut state = init_xkbcommon();

    println!(
        "Listening for key events on: {}",
        keyboard.name().unwrap_or("Unknown Device")
    );

    loop {
        let event = keyboard
            .next_event(ReadFlag::NORMAL)
            .map(|(_status, event)| event);

        match event {
            Ok(event) => {
                // If it is a keypress and not of type repeat
                if let EventCode::EV_KEY(key) = event.event_code
                    && event.value != 2
                {
                    // Evdev -> xkb keycode
                    let keycode = (key as u32 + 8).into(); // Keycode offset

                    let direction = if event.value == 0 {
                        KeyDirection::Up
                    } else {
                        // Get character
                        dbg!(state.key_get_utf8(keycode));

                        KeyDirection::Down
                    };

                    // Update state
                    state.update_key(keycode.into(), direction);
                }
            }
            Err(e) => continue,
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
