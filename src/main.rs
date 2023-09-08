use clap::Parser;
use debug_print::debug_println;
use lazy_static::lazy_static;
use rdev::{grab, Event, EventType, Key};
use screenshots::Screen;
use std::collections::HashSet;
use std::process::id;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use threadpool::ThreadPool;

lazy_static! {
    static ref SCREEN: Screen = Screen::all().unwrap()[0];
    static ref POOL: ThreadPool = ThreadPool::new(num_cpus::get());
    static ref KEY_BUFFER: Arc<Mutex<HashSet<Key>>> = Arc::new(Mutex::new(HashSet::new()));
    static ref IDLE_FLAG: AtomicBool = AtomicBool::new(true);
    static ref DIR_PATH: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    static ref MOUSE_MOVE_BUFFER: Arc<Mutex<Vec<Event>>> = Arc::new(Mutex::new(Vec::new()));
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Data directory path
    #[clap(default_value_t=String::from(""),short, long)]
    directory_path: String,
}

fn main() {
    let args = Args::parse();
    {
        let mut dir_path = DIR_PATH.lock().unwrap();
        *dir_path = args.directory_path;
    }

    if let Err(error) = grab(event_handler) {
        println!("Error: {:?}", error)
    }
}

fn save_state(json: String, timestamp: u128) {
    let dir_path = DIR_PATH.lock().unwrap();
    let path = std::format!("{}/{}", dir_path, timestamp);

    std::fs::write(std::format!("{}.json", path), json).expect("Failed to write to file");

    let image = SCREEN.capture().unwrap();
    image
        .save(format!("{}.jpg", path))
        .expect("Failed to image to file");
}

fn check_idle() -> bool {
    let buffer = KEY_BUFFER.lock().unwrap();

    debug_println!("Ctrl: {}", buffer.contains(&Key::ControlLeft));
    debug_println!("Alt: {}", buffer.contains(&Key::Alt));
    debug_println!("KeyP: {}", buffer.contains(&Key::KeyP));

    if buffer.contains(&Key::ControlLeft)
        && buffer.contains(&Key::Alt)
        && buffer.contains(&Key::KeyP)
    {
        IDLE_FLAG.store(!IDLE_FLAG.load(Ordering::Relaxed), Ordering::Relaxed);
    }
    IDLE_FLAG.load(Ordering::Relaxed)
}
fn event_handler(event: Event) -> Option<Event> {
    debug_println!("Event {:?}", event);

    let idle = check_idle();
    debug_println!("Idle: {}", idle);
    match event.event_type {
        EventType::MouseMove { x: _, y: _ } => {
            if !idle {
                let ref mut event_buffer = *MOUSE_MOVE_BUFFER.lock().unwrap();
                event_buffer.push(event.clone());
                debug_println!("buffer size: {}", event_buffer.len());
            }
            Some(event)
        }

        EventType::ButtonPress(_) => {
            let ref mut event_buffer = *MOUSE_MOVE_BUFFER.lock().unwrap();
            event_buffer.push(event.clone());
            let json = serde_json::to_string(event_buffer).unwrap();
            event_buffer.clear();
            let timestamp = event
                .time
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();

            if !idle {
                POOL.execute(move || save_state(json, timestamp));
            }
            Some(event)
        }

        EventType::ButtonRelease(_) => {
            let ref mut event_buffer = *MOUSE_MOVE_BUFFER.lock().unwrap();
            event_buffer.push(event.clone());
            let json = serde_json::to_string(event_buffer).unwrap();
            event_buffer.clear();

            let timestamp = event
                .time
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();

            if !idle {
                POOL.execute(move || save_state(json, timestamp));
            }

            Some(event)
        }

        EventType::KeyPress(key) => {
            let mut buffer = KEY_BUFFER.lock().unwrap();
            buffer.insert(key);
            Some(event)
        }
        EventType::KeyRelease(key) => {
            let mut buffer = KEY_BUFFER.lock().unwrap();
            buffer.remove(&key);
            Some(event)
        }
        EventType::Wheel {
            delta_x: _,
            delta_y: _,
        } => {
            let json = serde_json::to_string(&event).unwrap();
            let timestamp = event
                .time
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();

            if !idle {
                POOL.execute(move || save_state(json, timestamp));
            }
            Some(event)
        }
    }
}
