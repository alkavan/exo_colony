#[macro_use]
extern crate itertools;
#[macro_use]
extern crate worldgen;

mod game;
mod gui;
mod structures;
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

use crate::game::{MapController, ResourceGroup, ResourceManager};
use crate::gui::Menu;
use crate::structures::{Base, EnergyTrait, ResourceTrait, Storage, StorageTrait};
use crate::structures::{Mine, PowerPlant, Structure, StructureGroup};

use crate::util::format_welcome_message;
use crate::util::{EventBus, GameEvent, Tick};
use std::ops::SubAssign;

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
    let mut resource_manager = ResourceManager::new(vec![
        ResourceGroup::Energy,
        ResourceGroup::Metal,
        ResourceGroup::Mineral,
        ResourceGroup::Gas,
        ResourceGroup::Carbon,
    ]);

    let mut menu = Menu::new(vec![
        StructureGroup::Base,
        StructureGroup::Energy,
        StructureGroup::Mine,
        StructureGroup::Storage,
        StructureGroup::Factory,
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
                &resource_manager,
                controller.position(),
                elapsed,
                update_tick.delta(),
                draw_tick.delta(),
            );

            frame.render_widget(stats_widget, left_layout[0]);

            let console_widget = gui::draw_console_widget(&log_buffer);
            frame.render_widget(console_widget, left_layout[1]);

            let build_menu = gui::draw_build_menu_widget(&menu);
            frame.render_widget(build_menu, right_layout[0]);

            let info_panel = gui::draw_info_widget(
                controller.position(),
                controller.tile(),
                controller.object(),
            );
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
            GameEvent::Update => {
                let objects = controller.objects_mut().list_mut();

                for (position, object) in objects {
                    // let time_factor: f64 = update_tick.delta() as f64 / 2000.0;

                    let structure = object.structure.as_mut().unwrap();

                    match structure {
                        Structure::PowerPlant { structure } => resource_manager
                            .deposit(&ResourceGroup::Energy, structure.blueprint().energy_out()),
                        Structure::Mine { structure } => resource_manager.deposit(
                            &ResourceGroup::Mineral,
                            structure.blueprint().resource_out(),
                        ),
                        Structure::Base { structure } => resource_manager
                            .deposit(&ResourceGroup::Energy, structure.blueprint().energy_out()),
                        Structure::Storage { ref mut structure } => {
                            for (resource, amount) in resource_manager.list_mut() {
                                if *amount > 0 {
                                    let amount_stored = structure
                                        .blueprint_mut()
                                        .resource_add(resource, amount.clone());

                                    amount.sub_assign(amount_stored);
                                }
                            }
                        }
                    }
                }

                update_tick.update(&elapsed);
            }
            GameEvent::Draw => {
                let map_text = gui::render_map(
                    controller.map(),
                    controller.objects(),
                    controller.position(),
                );
                map_widget = Option::from(gui::draw_map_widget(&map_text));
                draw_tick.update(&elapsed);
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
                                KeyCode::Enter => {
                                    let structure = match menu.selected() {
                                        StructureGroup::Base => Structure::Base {
                                            structure: Base::new(),
                                        },
                                        StructureGroup::Energy => Structure::PowerPlant {
                                            structure: PowerPlant::new(),
                                        },
                                        StructureGroup::Mine => Structure::Mine {
                                            structure: Mine::new(),
                                        },
                                        StructureGroup::Storage => Structure::Storage {
                                            structure: Storage::new(),
                                        },
                                        StructureGroup::Factory => Structure::PowerPlant {
                                            structure: PowerPlant::new(),
                                        },
                                    };

                                    controller.add_structure(structure);
                                }
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
                                KeyCode::Char(c) => match c {
                                    'a' => {
                                        controller.left();
                                    }
                                    'd' => {
                                        controller.right();
                                    }
                                    'w' => {
                                        controller.up();
                                    }
                                    's' => {
                                        controller.down();
                                    }
                                    _ => {}
                                },
                                KeyCode::Null => {}
                                KeyCode::Esc => {
                                    // Quit
                                    disable_raw_mode()?;
                                    terminal.clear()?;
                                    break;
                                }
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
