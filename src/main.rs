use std::fs;

use evdev_rs::{
    Device,
    DeviceWrapper,
    ReadFlag, // enum*, EventCde},
    enums::EventCode,
};
use xkbcommon_rs::{Context, Keymap, State, xkb_keymap::RuleNames, xkb_state::KeyDirection};

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
    let context = Context::new(0).unwrap();
    let keymap = Keymap::new_from_names(
        context,
        Some(RuleNames {
            rules: None,
            model: Some("pc105".into()),
            layout: Some("ch".into()),
            variant: Some("de".into()),
            options: None,
        }),
        0,
    )
    .unwrap();
    State::new(keymap)
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
                if let EventCode::EV_KEY(key) = event.event_code {
                    state.update_key(
                        key as u32 + 8,
                        if event.value == 0 {
                            KeyDirection::Up
                        } else {
                            KeyDirection::Down
                        },
                    );
                    if let Some(chars) = state.key_get_utf8(key as u32) {
                        println!("Char: {}", chars[0] as char);
                    } else {
                        println!("No Char Mapping");
                    }
                }
            }

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
