use std::{collections::HashMap, fs};

use evdev_rs::{
    Device, DeviceWrapper, ReadFlag,
    enums::{
        EV_KEY::{self, *},
        EventCode,
    },
};

#[derive(Default, Debug)]
struct Log {
    one_grams: HashMap<char, u64>,
    two_grams: HashMap<[char; 2], u64>,
    three_grams: HashMap<[char; 3], u64>,
    current: [char; 2], // the last two characters, [1] being the more recent one
}
impl Log {
    fn push(&mut self, new: char) {
        *self.one_grams.entry(new).or_insert(0) += 1;
        *self.two_grams.entry([self.current[1], new]).or_insert(0) += 1;
        *self
            .three_grams
            .entry([self.current[0], self.current[1], new])
            .or_insert(0) += 1;

        self.current[0] = self.current[1];
        self.current[1] = new;
    }
}

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

    let keyboard = Device::new_from_path(keyboard_path).unwrap();
    println!(
        "Listening for key events on: {}",
        keyboard.name().unwrap_or("Unknown Device")
    );

    let mut log = Log::default();

    loop {
        let event = keyboard
            .next_event(ReadFlag::NORMAL)
            .map(|(_status, event)| event);

        match event {
            Ok(event) => {
                if let EventCode::EV_KEY(key) = event.event_code {
                    // Pressed
                    if event.value == 1 {
                        if let Some(char) = to_char(key) {
                            log.push(char);
                            dbg!(&log);
                        }
                    }
                }
            }

            Err(_) => continue,
        }
    }
}

fn to_char(key: EV_KEY) -> Option<char> {
    match key {
        KEY_MINUS => Some('-'),
        KEY_EQUAL => Some('='),
        KEY_Q => Some('q'),
        KEY_W => Some('w'),
        KEY_E => Some('e'),
        KEY_R => Some('r'),
        KEY_T => Some('t'),
        KEY_Y => Some('y'),
        KEY_U => Some('u'),
        KEY_I => Some('i'),
        KEY_O => Some('o'),
        KEY_P => Some('p'),
        KEY_LEFTBRACE => Some('['),
        KEY_RIGHTBRACE => Some(']'),
        KEY_ENTER => Some('\n'),
        KEY_A => Some('a'),
        KEY_S => Some('s'),
        KEY_D => Some('d'),
        KEY_F => Some('f'),
        KEY_G => Some('g'),
        KEY_H => Some('h'),
        KEY_J => Some('j'),
        KEY_K => Some('k'),
        KEY_L => Some('l'),
        KEY_SEMICOLON => Some(';'),
        KEY_APOSTROPHE => Some('\''),
        KEY_BACKSLASH => Some('\\'),
        KEY_Z => Some('z'),
        KEY_X => Some('x'),
        KEY_C => Some('c'),
        KEY_V => Some('v'),
        KEY_B => Some('b'),
        KEY_N => Some('n'),
        KEY_M => Some('m'),
        KEY_COMMA => Some(','),
        KEY_DOT => Some('.'),
        KEY_SLASH => Some('/'),
        KEY_SPACE => Some(' '),
        other => {
            dbg!(other);
            None
        }
    }
}
