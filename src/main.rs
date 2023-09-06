use clap::Parser;
use debug_print::debug_println;
use lazy_static::lazy_static;
use rdev::{grab, Event, EventType};
use screenshots::Screen;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use threadpool::ThreadPool;

lazy_static! {
    static ref SCREEN: Screen = Screen::all().unwrap()[0];
    static ref POOL: ThreadPool = ThreadPool::new(num_cpus::get());
    static ref DIR_PATH: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    static ref LAST_MOUSE_EVENT: Mutex<Event> = Mutex::new(Event {
        time: SystemTime::now(),
        name: None,
        event_type: EventType::MouseMove { x: 0.0, y: 0.0 }
    });
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
        .save(format!("{}.png", path))
        .expect("Failed to image to file");
}

fn event_handler(event: Event) -> Option<Event> {
    debug_println!("Event {:?}", event);
    match event.event_type {
        EventType::MouseMove { x: _, y: _ } => {
            let mut mouse_event = LAST_MOUSE_EVENT.lock().unwrap();
            *mouse_event = event.clone();
            Some(event)
        }

        EventType::ButtonPress(_) => {
            let last_mouse = (*LAST_MOUSE_EVENT.lock().unwrap()).clone();

            let data = vec![last_mouse, event.clone()];
            let json = serde_json::to_string(&data).unwrap();

            let timestamp = event
                .time
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();

            POOL.execute(move || save_state(json, timestamp));
            Some(event)
        }

        EventType::KeyPress(_)
        | EventType::Wheel {
            delta_x: _,
            delta_y: _,
        } => {
            let json = serde_json::to_string(&event).unwrap();

            let timestamp = event
                .time
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();

            POOL.execute(move || save_state(json, timestamp));
            Some(event)
        }
        _ => Some(event),
    }
}
