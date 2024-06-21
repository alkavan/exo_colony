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

use crate::game::{EnergyManager, MapController, ResourceGroup, ResourceManager};
use crate::gui::{FactoryCommoditySelect, Menu, MenuSelector, MineResourceSelect};
use crate::structures::{
    Base, CommodityGroup, CommodityOutputTrait, EnergyTrait, Factory, ResourceOutputTrait,
    ResourceRequire, ResourceStorageTrait, Storage,
};
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

    let mut energy_manager = EnergyManager::new();

    // An object for player score keeping and updating.
    let storage_resources = vec![
        ResourceGroup::Metal,
        ResourceGroup::Mineral,
        ResourceGroup::Gas,
        ResourceGroup::Carbon,
    ];

    let storage_commodities = vec![
        CommodityGroup::MetalPipe,
        CommodityGroup::MetalPlate,
        CommodityGroup::Gravel,
        CommodityGroup::Fuel,
    ];

    let mut resource_manager = ResourceManager::new(storage_resources, storage_commodities);

    let mut menu = Menu::new(vec![
        StructureGroup::Base,
        StructureGroup::Energy,
        StructureGroup::Mine,
        StructureGroup::Storage,
        StructureGroup::Factory,
    ]);

    let mut resource_select = MineResourceSelect::new(vec![
        ResourceGroup::Metal,
        ResourceGroup::Mineral,
        ResourceGroup::Carbon,
        ResourceGroup::Gas,
    ]);

    let mut commodity_select = FactoryCommoditySelect::new(vec![
        CommodityGroup::MetalPipe,
        CommodityGroup::MetalPlate,
        CommodityGroup::Gravel,
        CommodityGroup::Fuel,
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
            let menu_layout = gui::build_menu_layout(right_layout[0]);

            let stats_widget = gui::draw_stats_widget(
                &resource_manager,
                &energy_manager,
                controller.position(),
                elapsed,
                update_tick.delta(),
                draw_tick.delta(),
            );

            frame.render_widget(stats_widget, left_layout[0]);

            let console_widget = gui::draw_console_widget(&log_buffer);
            frame.render_widget(console_widget, left_layout[1]);

            let build_menu = gui::draw_structure_menu_widget(&menu);
            frame.render_widget(build_menu, menu_layout[0]);

            match menu.selected() {
                StructureGroup::Base => {}
                StructureGroup::Energy => {}
                StructureGroup::Mine => {
                    let resource_select_widget = gui::draw_resource_select_widget(&resource_select);
                    frame.render_widget(resource_select_widget, menu_layout[1]);
                }
                StructureGroup::Storage => {}
                StructureGroup::Factory => {
                    let commodity_select_widget =
                        gui::draw_commodity_select_widget(&commodity_select);
                    frame.render_widget(commodity_select_widget, menu_layout[1]);
                }
            }

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
                energy_manager.zero();

                energy_manager.collect(controller.objects_mut().list());

                let objects = controller.objects_mut().list_mut();
                for (position, object) in objects {
                    // let time_factor: f64 = update_tick.delta() as f64 / 2000.0;
                    let structure = object.structure.as_mut().unwrap();

                    match structure {
                        Structure::PowerPlant { structure } => {}
                        Structure::Mine { structure } => {
                            let energy_required = structure.blueprint().energy_in();
                            let energy_available = energy_manager.withdraw(energy_required);

                            if energy_available >= energy_required {
                                resource_manager.deposit_resource(
                                    structure.resource(),
                                    structure.blueprint().resource_out(),
                                );
                            } else {
                                let deficit = energy_required - energy_available;
                                energy_manager.deposit_deficit(deficit);
                            }
                        }
                        Structure::Base { structure } => {}
                        Structure::Storage { structure } => {
                            for (resource, amount) in resource_manager.list_resources_mut() {
                                if *amount > 0 {
                                    let amount_stored = structure
                                        .blueprint_mut()
                                        .resource_add(resource, amount.clone());

                                    amount.sub_assign(amount_stored);
                                }
                            }
                        }
                        Structure::Factory { structure } => {
                            let requires = structure.blueprint().requires();

                            if requires.is_some() {
                                let has_resources = requires.unwrap().iter().all(
                                    |(required_resource, required_amount)| {
                                        if *required_resource == ResourceGroup::Energy {
                                            return energy_manager
                                                .has_energy(required_amount.clone());
                                        }

                                        resource_manager.has_resource(
                                            required_resource,
                                            required_amount.clone(),
                                        )
                                    },
                                );

                                if has_resources {
                                    for (required_resource, required_amount) in
                                        requires.unwrap().iter()
                                    {
                                        if *required_resource == ResourceGroup::Energy {
                                            energy_manager.withdraw(required_amount.clone());
                                            continue;
                                        }

                                        resource_manager.withdraw_resource(
                                            required_resource,
                                            required_amount.clone(),
                                        );
                                    }

                                    let commodity_out = structure.blueprint().commodity_out();
                                    resource_manager
                                        .deposit_commodity(structure.commodity(), commodity_out);
                                }
                            }
                        }
                    }
                }

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
                                KeyCode::Left => {
                                    controller.left();
                                }
                                KeyCode::Enter => {
                                    let structure = match menu.selected() {
                                        StructureGroup::Base => Structure::Base {
                                            structure: Base::new(),
                                        },
                                        StructureGroup::Energy => Structure::PowerPlant {
                                            structure: PowerPlant::new(),
                                        },
                                        StructureGroup::Mine => Structure::Mine {
                                            structure: Mine::new(resource_select.selected()),
                                        },
                                        StructureGroup::Storage => Structure::Storage {
                                            structure: Storage::new(),
                                        },
                                        StructureGroup::Factory => Structure::Factory {
                                            structure: {
                                                Factory::new(commodity_select.selected())
                                            },
                                        },
                                    };

                                    controller.add_structure(structure);
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
                                KeyCode::Home => match menu.selected() {
                                    StructureGroup::Base => {}
                                    StructureGroup::Energy => {}
                                    StructureGroup::Mine => {
                                        resource_select.previous();
                                    }
                                    StructureGroup::Storage => {}
                                    StructureGroup::Factory => {
                                        commodity_select.previous();
                                    }
                                },
                                KeyCode::End => match menu.selected() {
                                    StructureGroup::Base => {}
                                    StructureGroup::Energy => {}
                                    StructureGroup::Mine => {
                                        resource_select.next();
                                    }
                                    StructureGroup::Storage => {}
                                    StructureGroup::Factory => {
                                        commodity_select.next();
                                    }
                                },
                                KeyCode::PageUp => {
                                    menu.previous();
                                }
                                KeyCode::PageDown => {
                                    menu.next();
                                }
                                KeyCode::Tab => {}
                                KeyCode::BackTab => {}
                                KeyCode::Delete => {
                                    controller.destroy_structure();
                                }
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
