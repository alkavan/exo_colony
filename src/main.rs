// SPDX-FileCopyrightText: Copyright (c) 2021-2026 Igal Alkon
// SPDX-License-Identifier: Zlib

extern crate itertools;
#[macro_use]
extern crate worldgen;

mod component;
mod game;
mod gui;
mod input;
mod managers;
mod structures;
mod util;
pub(crate) mod terminal;

use std::error::Error;
use std::time::SystemTime;

use tui::layout::Margin;
use tui::widgets::Paragraph;

use worldgen::world::Size;

use crate::game::{Commodity, Manufactured, MapController, Resource, WORLD_HEIGHT, WORLD_WIDTH};
use crate::gui::{
    FactoryCommoditySelect, Menu, MenuSelector, MineResourceSelect, RefineryResourceSelect,
};
use crate::input::{poll_inputs, Input};
use crate::managers::{EnergyManager, ResourceManager};
use crate::structures::{StructureFactory, StructureGroup};

use crate::util::{
    format_help_message, format_welcome_message, parse_args, random_seed, ConsoleLog, EventBus,
    GameEvent, Tick,
};

fn print_usage() {
    eprintln!("Exo Colony 0.3 — a terminal colony sim");
    eprintln!("Usage: exocolony [--seed <string>]");
    eprintln!("  --seed, -s    World generation seed (default: random 8-char)");
    eprintln!("  --help, -h    Show this help");
}

fn main() -> Result<(), Box<dyn Error>> {
    let (seed_arg, show_help) = parse_args(std::env::args());
    if show_help {
        print_usage();
        return Ok(());
    }
    let seed = seed_arg.unwrap_or_else(random_seed);

    let mut terminal = crate::terminal::setup()?;

    let now = SystemTime::now();
    let events = EventBus::new();

    let mut console = ConsoleLog::new();
    console.push(format_welcome_message(&seed));
    let mut console_view_height: u16 = 8;

    // For keeping game update interval
    let mut update_tick = Tick::new();

    // For keeping game draw interval
    let mut draw_tick = Tick::new();

    let mut energy_manager = EnergyManager::new();

    // An object for player score keeping and updating.
    let storage_resources = vec![
        Resource::Iron,
        Resource::Aluminum,
        Resource::Carbon,
        Resource::Silica,
        Resource::Uranium,
        Resource::Water,
    ];

    let storage_manufactured = vec![
        Manufactured::Silicon,
        Manufactured::Food,
        Manufactured::Steel,
        Manufactured::BioPlastic,
        Manufactured::Oxygen,
        Manufactured::Gravel,
        Manufactured::Hydrogen,
        Manufactured::FuelPellet,
    ];

    let storage_commodities = vec![
        Commodity::Concrete,
        Commodity::Semiconductor,
        Commodity::Fuel,
        Commodity::Glass,
        Commodity::FuelRod,
    ];

    let mut resource_manager =
        ResourceManager::new(storage_resources, storage_manufactured, storage_commodities);

    let mut menu = Menu::new(vec![
        StructureGroup::Base,
        StructureGroup::Power,
        StructureGroup::Mine,
        StructureGroup::Refinery,
        StructureGroup::Factory,
        StructureGroup::Storage,
    ]);

    let mut mine_select = MineResourceSelect::new(vec![
        Resource::Iron,
        Resource::Aluminum,
        Resource::Carbon,
        Resource::Silica,
        Resource::Uranium,
        Resource::Water,
    ]);

    let mut refinery_select = RefineryResourceSelect::new(vec![
        vec![Manufactured::Silicon],
        vec![Manufactured::Food],
        vec![Manufactured::Steel],
        vec![Manufactured::BioPlastic],
        vec![Manufactured::Hydrogen, Manufactured::Oxygen],
        vec![Manufactured::FuelPellet],
    ]);

    let mut factory_select = FactoryCommoditySelect::new(vec![
        Commodity::Concrete,
        Commodity::Semiconductor,
        Commodity::Fuel,
        Commodity::Glass,
        Commodity::FuelRod,
    ]);

    // The game controller works with the Map object.
    let mut controller = MapController::new(Size::of(WORLD_WIDTH as i64, WORLD_HEIGHT as i64), &seed);

    // Default margin used when drawing interfaces.
    let margin_1 = Margin {
        vertical: 1,
        horizontal: 1,
    };

    let mut map_widget: Option<Paragraph> = None;

    controller.generate_deposits();

    'game: loop {
        let elapsed = now.elapsed()?;
        let game_event = events.next()?;

        terminal.draw(|frame| {
            let main_layout = gui::build_main_layout(frame.size());
            let left_layout = gui::build_left_layout(main_layout[0]);
            let right_layout = gui::build_right_layout(main_layout[2]);
            let menu_layout = gui::build_menu_layout(right_layout[0]);
            let colony_layout = gui::build_colony_layout(left_layout[0]);

            let stats_widget_left = gui::draw_stats_widget_left(
                &resource_manager,
                &energy_manager,
                elapsed,
                update_tick.delta(),
                draw_tick.delta(),
            );

            let stats_widget_right = gui::draw_stats_widget_right(&resource_manager);

            frame.render_widget(stats_widget_left, colony_layout[0]);
            frame.render_widget(stats_widget_right, colony_layout[1]);

            console_view_height = left_layout[1].height.saturating_sub(2).max(1);
            let console_text = console.text();
            let console_widget = gui::draw_console_widget(
                &console_text,
                console.title(),
                console.paragraph_scroll(console_view_height),
            );
            frame.render_widget(console_widget, left_layout[1]);

            let build_menu = gui::draw_structure_menu_widget(&menu);
            frame.render_widget(build_menu, menu_layout[0]);

            match menu.selected() {
                StructureGroup::Base => {}
                StructureGroup::Power => {}
                StructureGroup::Mine => {
                    let resource_select_widget = gui::draw_mine_select_widget(&mine_select);
                    frame.render_widget(resource_select_widget, menu_layout[1]);
                }
                StructureGroup::Factory => {
                    let commodity_select_widget = gui::draw_factory_select_widget(&factory_select);
                    frame.render_widget(commodity_select_widget, menu_layout[1]);
                }
                StructureGroup::Refinery => {
                    let commodity_select_widget =
                        gui::draw_refinery_select_widget(&refinery_select);
                    frame.render_widget(commodity_select_widget, menu_layout[1]);
                }
                StructureGroup::Storage => {}
            }

            let info_panel = gui::draw_info_widget(
                controller.position(),
                controller.tile(),
                controller.object(),
            );
            frame.render_widget(info_panel, right_layout[1]);

            let map_viewport = main_layout[1].inner(&margin_1);
            controller.set_viewport(map_viewport.width, map_viewport.height);

            let map_title = gui::format_map_title(
                controller.follow(),
                controller.camera(),
                controller.position(),
                controller.map().width(),
                controller.map().height(),
                controller.seed(),
            );
            let map_block = gui::draw_map_block(map_title);
            frame.render_widget(map_block, main_layout[1]);

            // If the widget was drawn by the draw event, render it, otherwise do not.
            if map_widget.is_some() {
                frame.render_widget(map_widget.clone().unwrap(), map_viewport);
            }
        })?;

        match game_event {
            // When we get the draw event, we'll update the game map.
            // Map will not be drawn every loop iteration.
            GameEvent::Update => {
                energy_manager.zero();
                controller.clear_activity();
                controller.tick_construction();

                energy_manager.collect(controller.objects_mut().list_mut());

                let objects = controller.objects_mut().list_mut();
                resource_manager.collect(objects, &mut energy_manager);

                // if we discharged energy from storage, discharge batteries.
                if energy_manager.discharged() > 0 {
                    energy_manager.discharge(controller.objects_mut().list_mut());
                }

                // if we have available energy output, use it to charge batteries.
                if energy_manager.output() > 0 {
                    energy_manager.charge(controller.objects_mut().list_mut());
                }

                update_tick.update(&elapsed);
            }
            GameEvent::Draw => {
                let (view_w, view_h) = controller.view_size();
                let map_text = gui::render_map(
                    controller.map(),
                    controller.objects(),
                    controller.position(),
                    controller.camera(),
                    view_w,
                    view_h,
                );
                map_widget = Option::from(gui::draw_map_widget(&map_text));
                draw_tick.update(&elapsed);
            }
            GameEvent::Input => {
                for input in poll_inputs()? {
                    match input {
                        Input::MoveLeft => controller.left(),
                        Input::MoveRight => controller.right(),
                        Input::MoveUp => controller.up(),
                        Input::MoveDown => controller.down(),
                        Input::CameraLeft => controller.pan_left(),
                        Input::CameraRight => controller.pan_right(),
                        Input::CameraUp => controller.pan_up(),
                        Input::CameraDown => controller.pan_down(),
                        Input::ToggleFollow => {
                            controller.toggle_follow();
                            let state = if controller.follow() { "on" } else { "off" };
                            console.push_log(format!("Camera follow {}", state));
                        }
                        Input::PinHome => {
                            controller.pin_home();
                            console.push_log(format!(
                                "Home pinned at {}",
                                controller.position()
                            ));
                        }
                        Input::GoHome => {
                            if controller.go_home() {
                                console.push_log(format!(
                                    "Returned home at {}",
                                    controller.position()
                                ));
                            } else {
                                console.push_log(
                                    "No home pinned. Press F3 to pin this tile.".to_string(),
                                );
                            }
                        }
                        Input::Help => {
                            console.push(format_help_message());
                        }
                        Input::Confirm => {
                            let structure_group = menu.selected();
                            let tile = controller.tile();
                            let object = controller.object();
                            let existing = object.and_then(|o| o.structure.as_ref());

                            if let Some(structure) = existing {
                                let message = format!(
                                    "Cannot build here: {} already exists at {}",
                                    structure,
                                    controller.position()
                                );
                                console.push_log(message);
                            } else if StructureFactory::allowed(&structure_group, tile) {
                                let structure = StructureFactory::new(
                                    &structure_group,
                                    object,
                                    &resource_manager,
                                    &refinery_select,
                                    &factory_select,
                                );

                                if structure.is_some() {
                                    controller.add_structure(structure.unwrap());
                                    console.push_log(format!(
                                        "Construction started at {}",
                                        controller.position()
                                    ));
                                }
                            }
                        }
                        Input::SelectPrevious => match menu.selected() {
                            StructureGroup::Base => {}
                            StructureGroup::Power => {}
                            StructureGroup::Mine => {
                                mine_select.previous();
                            }
                            StructureGroup::Refinery => {
                                refinery_select.previous();
                            }
                            StructureGroup::Factory => {
                                factory_select.previous();
                            }
                            StructureGroup::Storage => {}
                        },
                        Input::SelectNext => match menu.selected() {
                            StructureGroup::Base => {}
                            StructureGroup::Power => {}
                            StructureGroup::Mine => {
                                mine_select.next();
                            }
                            StructureGroup::Refinery => {
                                refinery_select.next();
                            }
                            StructureGroup::Factory => {
                                factory_select.next();
                            }
                            StructureGroup::Storage => {}
                        },
                        Input::MenuPrevious => menu.previous(),
                        Input::MenuNext => menu.next(),
                        Input::LogPageUp => {
                            console.page_up(console_view_height as usize);
                        }
                        Input::LogPageDown => {
                            console.page_down(console_view_height as usize);
                        }
                        Input::Destroy => {
                            if controller.destroy_structure() {
                                console.push_log(format!(
                                    "Structure removed at {}",
                                    controller.position()
                                ));
                            }
                        }
                        Input::Quit => {
                            terminal::restore(&mut terminal)?;
                            break 'game;
                        }
                        Input::Mouse(message) => {
                            console.push_log(message);
                        }
                        Input::Resize { width, height } => {
                            let message = format!("Screen Resize ({}x{})", width, height);
                            console.push_log(message);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
