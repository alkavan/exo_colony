use std::ops::Neg;
use std::time::Duration;

use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap};

use crate::game::{Flora, GameMap, MapObject, MapTile, ObjectManager, Position, Resource};

use crate::managers::{EnergyManager, ResourceManager};
use crate::structures::{
    BatteryTrait, Commodity, EnergyTrait, ResourceStorageTrait, Structure, StructureBlueprint,
    StructureGroup,
};

#[derive(Clone, Copy)]
pub enum BlockType {
    Full,
    Dark,
    Medium,
    Light,
    Selected,
    Resource,
}

impl From<BlockType> for char {
    fn from(block: BlockType) -> Self {
        match block {
            BlockType::Full => '█',
            BlockType::Dark => '▓',
            BlockType::Medium => '▓',
            BlockType::Light => '░',
            BlockType::Selected => '◆',
            BlockType::Resource => 'R',
        }
    }
}

pub trait MenuSelector<T> {
    fn selected(&self) -> T;
    fn items(&self) -> Vec<ListItem>;
    fn next(&mut self);
    fn previous(&mut self);
    fn style(&self, name: String, index: usize) -> Span;
}

pub struct Menu {
    items: Vec<StructureGroup>,
    selected: usize,
    selected_style: Style,
    default_style: Style,
}

impl Menu {
    pub fn new(items: Vec<StructureGroup>) -> Menu {
        let selected = 0;

        let selected_style = Style::default().bg(Color::Red).fg(Color::White);
        let default_style = Style::default().bg(Color::Gray).fg(Color::Black);

        Menu {
            items,
            selected,
            selected_style,
            default_style,
        }
    }
}

impl MenuSelector<StructureGroup> for Menu {
    fn selected(&self) -> StructureGroup {
        return self.items[self.selected].clone();
    }

    fn items(&self) -> Vec<ListItem> {
        let list = self
            .items
            .iter()
            .enumerate()
            .map(|(index, structure_group)| {
                let content = self.style(structure_group.to_string(), index);
                ListItem::new(content)
            })
            .collect();

        return list;
    }

    fn next(&mut self) {
        if self.items.len() == 0 {
            return;
        }

        if self.selected == self.items.len() - 1 {
            self.selected = 0;
            return;
        }

        self.selected += 1;
    }

    fn previous(&mut self) {
        if self.items.len() == 0 {
            return;
        }

        if self.selected == 0 {
            self.selected = self.items.len() - 1;
            return;
        }

        self.selected -= 1;
    }

    fn style(&self, name: String, index: usize) -> Span {
        let style = if index == self.selected {
            self.selected_style
        } else {
            self.default_style
        };

        return Span::styled(name, style);
    }
}

pub struct MineResourceSelect {
    selected: usize,
    items: Vec<Resource>,
    selected_style: Style,
    default_style: Style,
}

impl MineResourceSelect {
    pub fn new(items: Vec<Resource>) -> MineResourceSelect {
        let selected = 0;

        let selected_style = Style::default().bg(Color::Blue).fg(Color::White);
        let default_style = Style::default().bg(Color::Gray).fg(Color::Black);

        return MineResourceSelect {
            selected,
            items,
            selected_style,
            default_style,
        };
    }
}

impl MenuSelector<Resource> for MineResourceSelect {
    fn selected(&self) -> Resource {
        return self.items[self.selected].clone();
    }

    fn items(&self) -> Vec<ListItem> {
        let list = self
            .items
            .iter()
            .enumerate()
            .map(|(index, structure_group)| {
                let content = self.style(structure_group.to_string(), index);
                ListItem::new(content)
            })
            .collect();

        return list;
    }

    fn next(&mut self) {
        if self.items.len() == 0 {
            return;
        }

        if self.selected == self.items.len() - 1 {
            self.selected = 0;
            return;
        }

        self.selected += 1;
    }

    fn previous(&mut self) {
        if self.items.len() == 0 {
            return;
        }

        if self.selected == 0 {
            self.selected = self.items.len() - 1;
            return;
        }

        self.selected -= 1;
    }

    fn style(&self, name: String, index: usize) -> Span {
        let style = if index == self.selected {
            self.selected_style
        } else {
            self.default_style
        };

        return Span::styled(name, style);
    }
}

pub struct FactoryCommoditySelect {
    selected: usize,
    items: Vec<Commodity>,
    selected_style: Style,
    default_style: Style,
}

impl FactoryCommoditySelect {
    pub fn new(items: Vec<Commodity>) -> FactoryCommoditySelect {
        let selected = 0;

        let selected_style = Style::default().bg(Color::Blue).fg(Color::White);
        let default_style = Style::default().bg(Color::Gray).fg(Color::Black);

        return FactoryCommoditySelect {
            selected,
            items,
            selected_style,
            default_style,
        };
    }
}

impl MenuSelector<Commodity> for FactoryCommoditySelect {
    fn selected(&self) -> Commodity {
        return self.items[self.selected].clone();
    }

    fn items(&self) -> Vec<ListItem> {
        let list = self
            .items
            .iter()
            .enumerate()
            .map(|(index, structure_group)| {
                let content = self.style(structure_group.to_string(), index);
                ListItem::new(content)
            })
            .collect();

        return list;
    }

    fn next(&mut self) {
        if self.items.len() == 0 {
            return;
        }

        if self.selected == self.items.len() - 1 {
            self.selected = 0;
            return;
        }

        self.selected += 1;
    }

    fn previous(&mut self) {
        if self.items.len() == 0 {
            return;
        }

        if self.selected == 0 {
            self.selected = self.items.len() - 1;
            return;
        }

        self.selected -= 1;
    }

    fn style(&self, name: String, index: usize) -> Span {
        let style = if index == self.selected {
            self.selected_style
        } else {
            self.default_style
        };

        return Span::styled(name, style);
    }
}

pub fn build_main_layout(area: Rect) -> Vec<Rect> {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(50),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(area);

    return layout;
}

pub fn build_left_layout(area: Rect) -> Vec<Rect> {
    let layout = Layout::default()
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(area);

    return layout;
}

pub fn build_right_layout(area: Rect) -> Vec<Rect> {
    let layout = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    return layout;
}

pub fn build_menu_layout(area: Rect) -> Vec<Rect> {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    return layout;
}

pub fn build_container_block(title: String) -> Block<'static> {
    let style = Style::default().fg(Color::White);

    let block = Block::default()
        .title(format!(" [ {} ] ", title))
        .borders(Borders::ALL)
        .style(style);

    return block;
}

pub fn draw_stats_widget(
    storage: &ResourceManager,
    energy: &EnergyManager,
    position: Position,
    elapsed: Duration,
    update_delta: u128,
    draw_delta: u128,
) -> List<'static> {
    let time = format!(
        "Time: {:.1} (update: {} ms) (draw: {} ms)",
        elapsed.as_secs_f32(),
        update_delta,
        draw_delta
    );

    let mut items = vec![ListItem::new(time.clone())];

    let position_content = format!("Position: ({}, {})", position.x, position.y);
    items.push(ListItem::new(position_content));

    // Energy list
    items.push(ListItem::new("-[ Energy ]-"));
    items.push(ListItem::new(format!(
        "(output: {}) (stored: {}) (deficit: {})",
        energy.output().to_string(),
        energy.stored().to_string(),
        (energy.deficit() as i64).neg().to_string()
    )));

    // Resource list
    items.push(ListItem::new(format!("{:-^30}", "[ Resources ]")));
    for (resource, amount) in storage.resources() {
        let deficit = storage.get_resource_deficit(resource) as i64;
        let content = format!(
            "{:>12}: {:>9} ({})",
            resource.to_string(),
            amount,
            deficit.neg().to_string()
        );
        items.push(ListItem::new(content));
    }

    // Commodity list
    items.push(ListItem::new(format!("{:-^30}", "[ Commodities ]")));
    for (commodity, amount) in storage.commodities() {
        let deficit = storage.get_commodity_deficit(commodity) as i64;
        let content = format!(
            "{:>12}: {:>9} ({})",
            commodity.to_string(),
            amount,
            deficit.neg().to_string()
        );
        items.push(ListItem::new(content));
    }

    let block = build_container_block("Colony".to_string());

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(Color::White));

    return list;
}

pub fn draw_console_widget(buffer: &String) -> Paragraph {
    let block = build_container_block("Console".to_string());

    let style = Style::default();

    let mut log = String::new();
    for l in buffer.lines().rev() {
        log.push_str(format!("{}\n", l).as_str())
    }

    let paragraph = Paragraph::new(log)
        .block(block)
        .style(style)
        .wrap(Wrap { trim: true });

    return paragraph;
}

pub fn draw_map_block() -> Block<'static> {
    let block = build_container_block("Map".to_string()).border_type(BorderType::Thick);

    return block;
}

pub fn draw_structure_menu_widget(menu: &Menu) -> List {
    let block = build_container_block("Build Menu".to_string());

    let list = List::new(menu.items())
        .block(block)
        .style(Style::default().fg(Color::White));

    return list;
}

pub fn draw_resource_select_widget(menu: &MineResourceSelect) -> List {
    let block = build_container_block("Resource Select".to_string());

    let list = List::new(menu.items())
        .block(block)
        .style(Style::default().fg(Color::White));

    return list;
}

pub fn draw_commodity_select_widget(menu: &FactoryCommoditySelect) -> List {
    let block = build_container_block("Commodity Select".to_string());

    let list = List::new(menu.items())
        .block(block)
        .style(Style::default().fg(Color::White));

    return list;
}

pub fn format_resource(resource_group: &Resource) -> String {
    return format!(
        "{:<10}: {}",
        "Resource:".to_string(),
        resource_group.to_string()
    );
}

pub fn format_resource_capacity(
    blueprint: &StructureBlueprint,
    resource_group: &Resource,
) -> String {
    let capacity = ResourceStorageTrait::capacity(blueprint, resource_group);
    let resource = ResourceStorageTrait::resource(blueprint, resource_group);

    return format!(
        "{:<10} ({:>8} / {:<8})",
        resource_group.to_string(),
        resource,
        capacity
    );
}

pub fn format_energy_io(blueprint: &StructureBlueprint) -> String {
    return format!(
        "{:<10} ({:>8} / {:<8})",
        "Energy I/O".to_string(),
        blueprint.energy_in().to_string(),
        blueprint.energy_out().to_string(),
    );
}

pub fn format_battery(blueprint: &StructureBlueprint) -> String {
    let stored = BatteryTrait::stored(blueprint);
    let capacity = BatteryTrait::capacity(blueprint);
    return format!(
        "{:<10} ({:>8} / {:<8})",
        "Battery".to_string(),
        stored,
        capacity,
    );
}

pub fn draw_info_widget(
    position: Position,
    tile: &MapTile,
    object: Option<&MapObject>,
) -> List<'static> {
    let block = build_container_block("Info".to_string());

    let mut items = vec![
        ListItem::new(format!("Tile({}, {}):", position.x, position.y)),
        ListItem::new(format!("Flora: {}", tile.flora.to_string())),
    ];

    if tile.resource.is_some() {
        items.push(ListItem::new(format!(
            "Resource: {}",
            tile.resource.unwrap().to_string()
        )));
    }

    if object.is_some() {
        let structure = object.unwrap().structure.as_ref();

        if structure.is_some() {
            let structure = structure.unwrap();
            let structure_content = format!("Structure: {}", structure.to_string());
            items.push(ListItem::new(structure_content));

            match structure {
                Structure::Base { ref structure } => {
                    items.push(ListItem::new(format_energy_io(structure.blueprint())));
                    items.push(ListItem::new(format_battery(structure.blueprint())));

                    for resource in structure.blueprint().resources() {
                        items.push(ListItem::new(format_resource_capacity(
                            structure.blueprint(),
                            resource,
                        )));
                    }
                }
                Structure::PowerPlant { .. } => {}
                Structure::Mine { ref structure } => {
                    items.push(ListItem::new(format_resource(structure.resource())));
                }
                Structure::Storage { ref structure } => {
                    for resource in structure.blueprint().resources() {
                        items.push(ListItem::new(format_resource_capacity(
                            structure.blueprint(),
                            resource,
                        )));
                    }
                }
                Structure::Factory { .. } => {}
            }
        }
    }

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(Color::White));

    return list;
}

fn get_flora_style(flora: &Flora) -> Style {
    match flora {
        Flora::Water => Style::default().bg(Color::Rgb(32, 178, 170)),
        Flora::Sand => Style::default().bg(Color::Yellow),
        Flora::Dirt => Style::default().bg(Color::Rgb(139, 69, 19)),
        Flora::Grass => Style::default().bg(Color::Rgb(0, 128, 0)),
        Flora::Rock => Style::default().bg(Color::Rgb(0, 0, 0)),
    }
}

fn get_structure_symbol(structure: &Structure) -> char {
    match structure {
        Structure::Base { .. } => 'B',
        Structure::PowerPlant { .. } => 'P',
        Structure::Mine { .. } => 'M',
        Structure::Storage { .. } => 'S',
        Structure::Factory { .. } => 'F',
    }
}

pub fn render_map(
    map: &GameMap,
    objects: &ObjectManager,
    position: Position,
) -> Vec<Spans<'static>> {
    let y = position.y as usize;
    let x = position.x as usize;

    let map_render = map.cache();

    let text = map_render
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let spans: Vec<Span> = row
                .iter()
                .enumerate()
                .map(|(j, tile)| {
                    let selected = y == i && x == j;

                    let mut style = get_flora_style(&tile.flora);

                    let block_symbol = if selected {
                        BlockType::Selected
                    } else {
                        BlockType::Light
                    };

                    let position = Position::new(j as i16, i as i16);
                    let object = objects.get(&position);

                    if object.is_some() {
                        if object.unwrap().structure.is_some() {
                            let structure = object.unwrap().structure.as_ref();
                            if structure.is_some() {
                                let structure_symbol = get_structure_symbol(structure.unwrap());

                                if selected {
                                    style = style.fg(Color::Red);
                                }

                                return Span::styled(
                                    char::from(structure_symbol).to_string(),
                                    style,
                                );
                            }
                        } else if object.unwrap().deposit.is_some() {
                            if selected {
                                style = style.fg(Color::Red);
                            }

                            return Span::styled(
                                char::from(BlockType::Resource).to_string(),
                                style,
                            );
                        }
                    }

                    return Span::styled(char::from(block_symbol).to_string(), style);
                })
                .collect();
            return Spans::from(spans);
        })
        .collect();

    return text;
}

pub fn draw_map_widget(text: &Vec<Spans<'static>>) -> Paragraph<'static> {
    let p = Paragraph::new(text.clone())
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().bg(Color::Rgb(0, 0, 0)))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    return p;
}
