#![feature(let_chains)]

use std::{
    array,
    collections::HashMap,
    fs,
    sync::{Arc, Mutex},
    time::Duration,
};

use evdev::{Device, InputEventKind};
use tokio::time;
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
                .map(|(key, value)| (key.replace('\\', "\\\\").replace('\r', "\\n"), value))
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
            let path = format!("/home/julius/ngrams/{}-grams.txt", n + 1);
            fs::write(path, serialized_ngram).unwrap();
        }
    }
    fn deserialize() -> Self {
        let ngrams = array::from_fn(|n| {
            let path = format!("/home/julius/ngrams/{}-grams.txt", n + 1);
            let serialized_ngram = fs::read_to_string(path).unwrap();

            serialized_ngram
                .lines()
                // Split into key & value
                .map(|line| {
                    let mut split = line.split(' ');
                    let value = split.next().unwrap();
                    let key = split.next().unwrap();
                    (key, value)
                })
                // Parse value
                .map(|(key, value)| (key, value.parse().unwrap()))
                // Process special characters
                .map(|(key, value)| (key.replace("\\n", "\r").replace("\\\\", "\\"), value))
                .collect()
        });

        Self {
            ngrams,
            ..Default::default()
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

    let log = Arc::new(Mutex::new(Log::deserialize()));

    // Background task for serialization
    let log_clone = Arc::clone(&log);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;
            log_clone.lock().unwrap().serialize();
        }
    });

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
                    log.lock().unwrap().push(char);
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
