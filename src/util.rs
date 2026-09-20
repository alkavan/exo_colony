// SPDX-FileCopyrightText: Copyright (c) 2021-2026 Igal Alkon
// SPDX-License-Identifier: Zlib

use chrono::Local;
use std::collections::VecDeque;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Kept log lines (oldest dropped first). A few hundred is enough for a session
/// without growing the draw string without bound.
pub const CONSOLE_CAPACITY: usize = 256;

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
    format!("[{}] {}", time, message)
}

/// Chronological console: newest line is last. `scroll == 0` pins the view to
/// the tail so new messages keep coming into view; Page Up moves the window
/// toward older lines and holds it there until Page Down reaches the tail.
pub struct ConsoleLog {
    lines: VecDeque<String>,
    scroll: usize,
}

impl ConsoleLog {
    pub fn new() -> ConsoleLog {
        ConsoleLog {
            lines: VecDeque::new(),
            scroll: 0,
        }
    }

    pub fn push(&mut self, message: String) {
        for raw in message.lines() {
            let line = raw.trim_end().to_string();
            if line.is_empty() {
                continue;
            }
            self.lines.push_back(line);
        }
        while self.lines.len() > CONSOLE_CAPACITY {
            self.lines.pop_front();
        }
        let max_scroll = self.max_scroll(1);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }
    }

    pub fn push_log(&mut self, message: String) {
        self.push(get_log(message));
    }

    fn max_scroll(&self, view_h: usize) -> usize {
        self.lines.len().saturating_sub(view_h.max(1))
    }

    /// Distance from the tail. Zero means follow the latest line.
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    pub fn following(&self) -> bool {
        self.scroll == 0
    }

    pub fn page_up(&mut self, view_h: usize) {
        let page = view_h.max(1);
        let max_scroll = self.max_scroll(page);
        self.scroll = (self.scroll + page).min(max_scroll);
    }

    pub fn page_down(&mut self, view_h: usize) {
        self.scroll = self.scroll.saturating_sub(view_h.max(1));
    }

    pub fn text(&self) -> String {
        let mut out = String::new();
        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(line);
        }
        out
    }

    /// Vertical offset for `Paragraph::scroll`: show the tail when `scroll == 0`.
    pub fn paragraph_scroll(&self, view_h: u16) -> u16 {
        let view_h = view_h.max(1) as usize;
        let max_from_top = self.lines.len().saturating_sub(view_h);
        let from_top = max_from_top.saturating_sub(self.scroll);
        from_top as u16
    }

    pub fn title(&self) -> String {
        if self.following() {
            format!("Console [{}] follow", self.lines.len())
        } else {
            format!("Console [{}] +{}", self.lines.len(), self.scroll)
        }
    }
}

pub fn format_welcome_message(seed: &str) -> String {
    let mut message = String::from("Welcome to Exo Colony 0.3!");
    message.push_str(&format!(" World seed: {}.", seed));
    message.push_str(" Arrows/WASD move the cursor. Alt+Arrows/WASD pan the camera.");
    message.push_str(" F4 toggles camera-follow. F3 pins home, Home returns to it.");
    message.push_str(" ; ' cycle the build menu. - = (or , .) cycle variants.");
    message.push_str(" PageUp/PageDown scroll the console (pinned to latest at the bottom).");
    message.push_str(" Enter places a structure. Delete removes one. ? reprints help. Esc quits.");
    get_log(message)
}

pub fn format_help_message() -> String {
    get_log(
        "Controls: cursor Arrows/WASD | camera Alt+Arrows | F4 follow | F3 pin home | Home go home | ;/' menu | -/= variant | PgUp/PgDn console | Enter build | Del destroy | Esc quit."
            .to_string(),
    )
}

pub fn parse_args(args: impl IntoIterator<Item = String>) -> (Option<String>, bool) {
    let mut seed = None;
    let mut help = false;
    let mut iter = args.into_iter();
    let _bin = iter.next();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--seed" | "-s" => {
                seed = iter.next();
            }
            "--help" | "-h" => {
                help = true;
            }
            other if other.starts_with("--seed=") => {
                seed = Some(other.trim_start_matches("--seed=").to_string());
            }
            _ => {}
        }
    }
    (seed, help)
}

pub fn random_seed() -> String {
    use rand::RngExt;
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..8)
        .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
        .collect()
}

pub struct Tick {
    delta: u128,
    duration: Duration,
}

impl Tick {
    pub fn new() -> Tick {
        let delta = 0;
        let duration = Duration::from_millis(0);

        Tick { delta, duration }
    }

    pub fn update(&mut self, elapsed: &Duration) {
        self.delta = elapsed.as_millis() - self.duration.as_millis();
        self.duration = elapsed.clone();
    }

    pub fn delta(&self) -> u128 {
        self.delta
    }
}
