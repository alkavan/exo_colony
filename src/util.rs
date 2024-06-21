use chrono::Local;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub enum GameEvent {
    Update,
    Draw,
    Input,
}

pub struct EventBus {
    rx: mpsc::Receiver<GameEvent>,
    update_handle: thread::JoinHandle<()>,
    draw_handle: thread::JoinHandle<()>,
    input_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub update_rate: Duration,
    pub draw_rate: Duration,
    pub input_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            update_rate: Duration::from_millis(240),
            draw_rate: Duration::from_millis(30),
            input_rate: Duration::from_millis(10),
        }
    }
}

/*
 This is the event bus.
 It supports 3 types of events with different durations:

 - Update: used for updating different game variables.
 - Draw: used to draw elements in the terminal screen
 - Input: use listen to keyboard inputs.
*/
impl EventBus {
    pub fn new() -> EventBus {
        EventBus::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> EventBus {
        let (tx, rx) = mpsc::channel();

        let update_handle = {
            let tx = tx.clone();

            thread::spawn(move || loop {
                if tx.send(GameEvent::Update).is_err() {
                    break;
                }
                thread::sleep(config.update_rate);
            })
        };

        let draw_handle = {
            let tx = tx.clone();

            thread::spawn(move || loop {
                if tx.send(GameEvent::Draw).is_err() {
                    break;
                }
                thread::sleep(config.draw_rate);
            })
        };

        let input_handle = {
            let tx = tx.clone();

            thread::spawn(move || loop {
                if tx.send(GameEvent::Input).is_err() {
                    break;
                }
                thread::sleep(config.input_rate);
            })
        };

        EventBus {
            rx,
            update_handle,
            draw_handle,
            input_handle,
        }
    }

    pub fn next(&self) -> Result<GameEvent, mpsc::RecvError> {
        self.rx.recv()
    }
}

pub fn get_log(message: String) -> String {
    let time = Local::now().format("%H:%M:%S");
    return format!("[{}] {}\n", time, message);
}

pub fn format_welcome_message() -> String {
    let mut message = String::from("Welcome!");
    message.push_str(" Use the arrow (or AWSD) keys to move on map.");
    message.push_str(" Use the ENTER to place structure or action.");
    message.push_str(" Use PageUp/PageDown and Home/End to navigate menus.");
    message.push_str(" Use ESC to exit the game.");
    return get_log(message);
}

pub struct Tick {
    delta: u128,
    duration: Duration,
}

impl Tick {
    pub fn new() -> Tick {
        let delta = 0;
        let duration = Duration::from_millis(0);

        return Tick { delta, duration };
    }

    pub fn update(&mut self, elapsed: &Duration) {
        self.delta = elapsed.as_millis() - self.duration.as_millis();
        self.duration = elapsed.clone();
    }

    pub fn delta(&self) -> u128 {
        return self.delta;
    }
}
