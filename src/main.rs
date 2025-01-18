#![feature(let_chains)]

use std::{collections::HashMap, fs, io::BufWriter};

use evdev::{Device, EventType, InputEventKind};
use xkbcommon::xkb::{Context, KeyDirection, Keymap, State};

#[derive(Default, Debug)]
struct Log {
    ngrams: [HashMap<String, u64>; 3],
    current: [char; 2], // the last two characters, [1] being the more recent one
}
impl Log {
    fn push(&mut self, new: char) {
        *self.ngrams[0].entry(new.into()).or_insert(0) += 1;
        *self.ngrams[1]
            .entry([self.current[1], new].iter().collect())
            .or_insert(0) += 1;
        *self.ngrams[2]
            .entry([self.current[0], self.current[1], new].iter().collect())
            .or_insert(0) += 1;

        self.current[0] = self.current[1];
        self.current[1] = new;
    }
    fn serialize(&self) {
        let serialized_ngrams = self.ngrams.iter().map(|gram| {
            gram.iter()
                // Process special characters
                .map(|(key, value)| (key.replace('\n', "\\n").replace('\\', "\\\\"), value))
                // Format key & value
                .map(|(key, value)| format!("{value} {key}"))
                // .join('\n)
                .fold(String::new(), |mut acc, item| {
                    acc.push('\n');
                    acc.push_str(&item);
                    acc
                })
        });

        for (n, serialized_ngram) in serialized_ngrams.enumerate() {
            let path = format!("~/ngrams/{}-grams.txt", n + 1);
            fs::write(path, serialized_ngram).unwrap();
        }
    }
}

fn get_keyboard() -> Device {
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

    Device::open(keyboard_path).unwrap()
}

fn init_xkbcommon() -> State {
    let context = Context::new(0);
    let keymap = Keymap::new_from_names(&context, "", "pc105", "ch", "de", None, 0).unwrap();
    State::new(&keymap)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keyboard = get_keyboard();
    let mut state = init_xkbcommon();

    println!("Listening for key events on: {}", keyboard.name().unwrap());

    let mut log = Log::default();

    let mut events = keyboard.into_event_stream()?;
    loop {
        let event = events.next_event().await?;

        // If it is a keypress and not of type repeat
        if let InputEventKind::Key(key) = event.kind()
            && event.value() != 2
        {
            // Evdev -> xkb keycode
            let keycode = (key.code() as u32 + 8).into(); // Keycode offset

            // Pressed
            if event.value() == 1 {
                let string = state.key_get_utf8(keycode);
                if let Some(char) = string.chars().next() {
                    log.push(char);
                    dbg!(&log);
                }
            }

            let direction = if event.value() == 0 {
                KeyDirection::Up
            } else {
                KeyDirection::Down
            };

            // Update state
            state.update_key(keycode.into(), direction);
        }
    }
}
