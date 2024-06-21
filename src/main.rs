#[macro_use]
extern crate itertools;
#[macro_use]
extern crate worldgen;

mod game;
mod gui;
mod util;

use std::error::Error;
use std::io;
use std::time::{Duration, SystemTime};

use tui::backend::CrosstermBackend;
use tui::layout::Margin;
use tui::widgets::Paragraph;
use tui::Terminal;

use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use worldgen::world::Size;

use crate::game::{MapController, ResourceGroup, ResourceStorage};
use crate::gui::Menu;
use crate::util::format_welcome_message;
use crate::util::{EventBus, GameEvent, Tick};

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    enable_raw_mode()?;

    let now = SystemTime::now();
    let events = EventBus::new();

    let mut log_buffer = String::default();
    log_buffer.push_str(&format_welcome_message());

    // For keeping game update interval
    let mut update_tick = Tick::new();

    // For keeping game draw interval
    let mut draw_tick = Tick::new();

    // An object for player score keeping and updating.
    let mut resource_storage = ResourceStorage::new(vec![
        ResourceGroup::Metal,
        ResourceGroup::Mineral,
        ResourceGroup::Gas,
        ResourceGroup::Carbon,
    ]);

    let mut menu = Menu::new(vec![
        "Base".to_string(),
        "Power Plant".to_string(),
        "Mine".to_string(),
        "Storage".to_string(),
        "Factory".to_string(),
    ]);

    // The game controller, work with the Map object.
    let mut controller = MapController::new(Size::of(90, 40));

    // Default margin used when drawing interfaces.
    let margin_1 = Margin {
        vertical: 1,
        horizontal: 1,
    };

    let mut map_widget: Option<Paragraph> = None;

    loop {
        let elapsed = now.elapsed()?;
        let game_event = events.next()?;

        terminal.draw(|frame| {
            let main_layout = gui::build_main_layout(frame.size());
            let left_layout = gui::build_left_layout(main_layout[0]);
            let right_layout = gui::build_right_layout(main_layout[2]);

            let stats_widget = gui::draw_stats_widget(
                &resource_storage,
                controller.position(),
                controller.tile(),
                elapsed,
                update_tick.delta(),
                draw_tick.delta(),
            );

            frame.render_widget(stats_widget, left_layout[0]);

            let console_widget = gui::draw_console_widget(&log_buffer);
            frame.render_widget(console_widget, left_layout[1]);

            let build_menu = gui::draw_build_menu_widget(&menu);
            frame.render_widget(build_menu, right_layout[0]);

            let info_panel = gui::draw_info_widget(controller.position(), controller.tile());
            frame.render_widget(info_panel, right_layout[1]);

            let map_block = gui::draw_map_block();
            frame.render_widget(map_block, main_layout[1]);

            let map_viewport = main_layout[1].inner(&margin_1);

            // If the widget was drawn by the draw event, render it, otherwise do not.
            if map_widget.is_some() {
                frame.render_widget(map_widget.clone().unwrap(), map_viewport);
            }
        })?;

        match game_event {
            // When we get the draw event, we'll update the game map.
            // Map will not be drawn every loop iteration.
            GameEvent::Draw => {
                let map_text = gui::render_map(controller.map(), controller.position());
                map_widget = Option::from(gui::draw_map_widget(&map_text));
                draw_tick.update(&elapsed);
            }
            GameEvent::Update => {
                update_tick.update(&elapsed);
            }
            GameEvent::Input => {
                if poll(Duration::from_millis(0))? {
                    // It's guaranteed that the `read()` won't block when the `poll()`
                    // function returns `true`
                    match read()? {
                        Event::Key(event) => {
                            // let log = util::get_log(format!("{:?}", event));
                            // log_buffer.push_str(&log);
                            match event.code {
                                KeyCode::Backspace => {}
                                KeyCode::Enter => {}
                                KeyCode::Left => {
                                    controller.left();
                                }
                                KeyCode::Right => {
                                    controller.right();
                                }
                                KeyCode::Up => {
                                    controller.up();
                                }
                                KeyCode::Down => {
                                    controller.down();
                                }
                                KeyCode::Home => {}
                                KeyCode::End => {}
                                KeyCode::PageUp => {
                                    menu.previous();
                                }
                                KeyCode::PageDown => {
                                    menu.next();
                                }
                                KeyCode::Tab => {}
                                KeyCode::BackTab => {}
                                KeyCode::Delete => {}
                                KeyCode::Insert => {}
                                KeyCode::F(_) => {}
                                KeyCode::Char(c) => {
                                    // If 'q' key is pressed, clear screen and exit.
                                    if c == 'q' {
                                        disable_raw_mode()?;
                                        terminal.clear()?;
                                        break;
                                    }
                                }
                                KeyCode::Null => {}
                                KeyCode::Esc => {}
                            }
                        }
                        Event::Mouse(event) => {
                            let log = util::get_log(format!("{:?}", event));
                            log_buffer.push_str(&log);
                        }
                        Event::Resize(width, height) => {
                            let message = format!("Screen Resize ({}x{})", width, height);
                            let log = util::get_log(message);
                            log_buffer.push_str(&log);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
