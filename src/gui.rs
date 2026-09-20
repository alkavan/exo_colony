// SPDX-FileCopyrightText: Copyright (c) 2021-2026 Igal Alkon
// SPDX-License-Identifier: Zlib

use std::ops::Neg;
use std::time::Duration;

use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap};

use crate::game::{
    Commodity, Flora, GameMap, Manufactured, MapObject, MapTile, ObjectManager, Position, Resource,
};

use crate::managers::{EnergyManager, ResourceManager};
use crate::structures::{
    BatteryTrait, EnergyTrait, ResourceStorageTrait, Structure, StructureBlueprint, StructureGroup,
};
use itertools::Itertools;

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
            BlockType::Resource => '·',
        }
    }
}

pub trait MenuSelector<T> {
    fn selected(&self) -> T;
    fn items(&self) -> Vec<ListItem<'static>>;
    fn next(&mut self);
    fn previous(&mut self);
    fn style(&self, name: String, index: usize) -> Span<'_>;
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
        self.items[self.selected].clone()
    }

    fn items(&self) -> Vec<ListItem<'static>> {
        let list = self
            .items
            .iter()
            .enumerate()
            .map(|(index, structure_group)| {
                let style = if index == self.selected {
                    self.selected_style
                } else {
                    self.default_style
                };
                let content = Span::styled(structure_group.to_string(), style);
                ListItem::new(content)
            })
            .collect();

        list
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

    fn style(&self, name: String, index: usize) -> Span<'_> {
        let style = if index == self.selected {
            self.selected_style
        } else {
            self.default_style
        };

        Span::styled(name, style)
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

        MineResourceSelect {
            selected,
            items,
            selected_style,
            default_style,
        }
    }
}

impl MenuSelector<Resource> for MineResourceSelect {
    fn selected(&self) -> Resource {
        self.items[self.selected].clone()
    }

    fn items(&self) -> Vec<ListItem<'static>> {
        let list = self
            .items
            .iter()
            .enumerate()
            .map(|(index, structure_group)| {
                let style = if index == self.selected {
                    self.selected_style
                } else {
                    self.default_style
                };
                let content = Span::styled(structure_group.to_string(), style);
                ListItem::new(content)
            })
            .collect();

        list
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

    fn style(&self, name: String, index: usize) -> Span<'_> {
        let style = if index == self.selected {
            self.selected_style
        } else {
            self.default_style
        };

        Span::styled(name, style)
    }
}

pub struct RefineryResourceSelect {
    selected: usize,
    items: Vec<Vec<Manufactured>>,
    selected_style: Style,
    default_style: Style,
}

impl RefineryResourceSelect {
    pub fn new(items: Vec<Vec<Manufactured>>) -> RefineryResourceSelect {
        let selected = 0;

        let selected_style = Style::default().bg(Color::Blue).fg(Color::White);
        let default_style = Style::default().bg(Color::Gray).fg(Color::Black);

        RefineryResourceSelect {
            selected,
            items,
            selected_style,
            default_style,
        }
    }
}

impl MenuSelector<Vec<Manufactured>> for RefineryResourceSelect {
    fn selected(&self) -> Vec<Manufactured> {
        self.items[self.selected].clone()
    }

    fn items(&self) -> Vec<ListItem<'static>> {
        let list = self
            .items
            .iter()
            .enumerate()
            .map(|(index, structure_group)| {
                let style = if index == self.selected {
                    self.selected_style
                } else {
                    self.default_style
                };
                let content = Span::styled(structure_group.iter().join(", "), style);
                ListItem::new(content)
            })
            .collect();

        list
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

    fn style(&self, name: String, index: usize) -> Span<'_> {
        let style = if index == self.selected {
            self.selected_style
        } else {
            self.default_style
        };

        Span::styled(name, style)
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

        FactoryCommoditySelect {
            selected,
            items,
            selected_style,
            default_style,
        }
    }
}

impl MenuSelector<Commodity> for FactoryCommoditySelect {
    fn selected(&self) -> Commodity {
        self.items[self.selected].clone()
    }

    fn items(&self) -> Vec<ListItem<'static>> {
        let list = self
            .items
            .iter()
            .enumerate()
            .map(|(index, structure_group)| {
                let style = if index == self.selected {
                    self.selected_style
                } else {
                    self.default_style
                };
                let content = Span::styled(structure_group.to_string(), style);
                ListItem::new(content)
            })
            .collect();

        list
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

    fn style(&self, name: String, index: usize) -> Span<'_> {
        let style = if index == self.selected {
            self.selected_style
        } else {
            self.default_style
        };

        Span::styled(name, style)
    }
}

pub fn build_main_layout(area: Rect) -> Vec<Rect> {
    // Very conservative size checks
    if area.width < 50 || area.height < 15 {
        // For very small terminals, just use the full area for all sections
        return vec![area, area, area];
    }

    // Use fixed widths instead of percentages to avoid overflow
    let left_width = (area.width / 3).max(10);
    let right_width = (area.width / 5).max(15);
    let map_width = area.width.saturating_sub(left_width + right_width + 2); // 2 for margins

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([
            Constraint::Length(left_width),
            Constraint::Length(map_width),
            Constraint::Length(right_width),
        ])
        .split(area);

    layout
}


pub fn build_left_layout(area: Rect) -> Vec<Rect> {
    let layout = Layout::default()
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(area);

    layout
}

pub fn build_right_layout(area: Rect) -> Vec<Rect> {
    let layout = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    layout
}

pub fn build_menu_layout(area: Rect) -> Vec<Rect> {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    layout
}

pub fn build_colony_layout(area: Rect) -> Vec<Rect> {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    layout
}

pub fn build_container_block(title: String) -> Block<'static> {
    let style = Style::default().fg(Color::White);

    let block = Block::default()
        .title(format!(" [ {} ] ", title))
        .borders(Borders::ALL)
        .style(style);

    block
}

pub fn draw_stats_widget_left(
    storage: &ResourceManager,
    energy: &EnergyManager,
    elapsed: Duration,
    update_delta: u128,
    draw_delta: u128,
) -> List<'static> {
    // Time
    let mut items = vec![
        ListItem::new(format!("Time: {:.1} (seconds)", elapsed.as_secs_f32())),
        ListItem::new(format!("Draw: {} (ms)", update_delta)),
        ListItem::new(format!("Update: {} (ms)", draw_delta)),
    ];

    // Energy list
    items.push(ListItem::new(format!("{:-^30}", "[ Energy ]")));
    items.push(ListItem::new(format!(
        "{:>9}: {:>9}",
        "Output".to_string(),
        energy.output().to_string()
    )));
    items.push(ListItem::new(format!(
        "{:>9}: {:>9}",
        "Stored".to_string(),
        energy.stored().to_string()
    )));
    items.push(ListItem::new(format!(
        "{:>9}: {:>9}",
        "Deficit".to_string(),
        (energy.deficit() as i64).neg().to_string()
    )));

    // Resource list
    items.push(ListItem::new(format!("{:-^30}", "[ Resources ]")));
    for (resource, amount) in storage.resources() {
        let deficit = storage.get_resource_deficit(resource) as i64;
        let content = format!(
            "{:>9}: {:>9} ({})",
            resource.to_string(),
            amount,
            deficit.neg().to_string()
        );
        items.push(ListItem::new(content));
    }

    let block = build_container_block("Colony Information".to_string());

    List::new(items)
        .block(block)
        .style(Style::default().fg(Color::White))
}

pub fn draw_stats_widget_right(storage: &ResourceManager) -> List<'static> {
    let mut items = vec![];

    // Manufactured list
    items.push(ListItem::new(format!("{:-^30}", "[ Manufactured ]")));
    for (manufactured, amount) in storage.manufactured() {
        let deficit = storage.get_manufactured_deficit(manufactured) as i64;
        let content = format!(
            "{:>14}: {:>9} ({})",
            manufactured.to_string(),
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
            "{:>14}: {:>9} ({})",
            commodity.to_string(),
            amount,
            deficit.neg().to_string()
        );
        items.push(ListItem::new(content));
    }

    let block = build_container_block("Colony Information".to_string());

    List::new(items)
        .block(block)
        .style(Style::default().fg(Color::White))
}

pub fn draw_console_widget(
    text: &str,
    title: String,
    scroll_y: u16,
) -> Paragraph<'_> {
    let block = build_container_block(title);

    Paragraph::new(text)
        .block(block)
        .style(Style::default())
        .wrap(Wrap { trim: true })
        .scroll((scroll_y, 0))
}

pub fn draw_map_block(title: String) -> Block<'static> {
    build_container_block(title).border_type(BorderType::Thick)
}

pub fn format_map_title(
    follow: bool,
    camera: Position,
    cursor: Position,
    world_w: u16,
    world_h: u16,
    seed: &str,
) -> String {
    let mode = if follow { "FOLLOW" } else { "FREE" };
    format!(
        "Map [{}] cam({},{}) cur({},{}) {}x{} seed:{}",
        mode, camera.x, camera.y, cursor.x, cursor.y, world_w, world_h, seed
    )
}

pub fn draw_structure_menu_widget(menu: &Menu) -> List<'static> {
    let block = build_container_block("Build Menu".to_string());

    List::new(menu.items())
        .block(block)
        .style(Style::default().fg(Color::White))
}

pub fn draw_mine_select_widget(menu: &MineResourceSelect) -> List<'static> {
    let block = build_container_block("Mine Select".to_string());

    List::new(menu.items())
        .block(block)
        .style(Style::default().fg(Color::White))
}

pub fn draw_factory_select_widget(menu: &FactoryCommoditySelect) -> List<'static> {
    let block = build_container_block("Factory Select".to_string());

    List::new(menu.items())
        .block(block)
        .style(Style::default().fg(Color::White))
}

pub fn draw_refinery_select_widget(menu: &RefineryResourceSelect) -> List<'static> {
    let block = build_container_block("Refinery Select".to_string());

    List::new(menu.items())
        .block(block)
        .style(Style::default().fg(Color::White))
}

pub fn format_mine_resource(resource_group: &Resource) -> String {
    format!("Resource: {}", resource_group.to_string())
}

pub fn format_resource_capacity(
    blueprint: &StructureBlueprint,
    resource_group: &Resource,
) -> String {
    let capacity = ResourceStorageTrait::capacity(blueprint, resource_group);
    let resource = ResourceStorageTrait::resource(blueprint, resource_group);

    format!(
        "{:<10} ({:>8} / {:<8})",
        resource_group.to_string(),
        resource,
        capacity
    )
}

pub fn format_energy_io(blueprint: &StructureBlueprint) -> String {
    format!(
        "{:<10} ({:>8} / {:<8})",
        "Energy I/O".to_string(),
        blueprint.energy_in().to_string(),
        blueprint.energy_out().to_string(),
    )
}

pub fn format_battery(blueprint: &StructureBlueprint) -> String {
    let stored = BatteryTrait::stored(blueprint);
    let capacity = BatteryTrait::capacity(blueprint);
    format!(
        "{:<10} ({:>8} / {:<8})",
        "Battery".to_string(),
        stored,
        capacity,
    )
}

pub fn draw_info_widget(
    position: Position,
    tile: &MapTile,
    object: Option<&MapObject>,
) -> List<'static> {
    let block = build_container_block("Info".to_string());

    let mut items = vec![
        ListItem::new(format!("Position: ({}, {})", position.x, position.y)),
        ListItem::new(format!("Flora: {}", tile.flora.to_string())),
    ];

    if tile.is_resource {
        if let Some(object) = object {
            if let Some(deposit) = object.deposit {
                items.push(ListItem::new(format!(
                    "Deposit: {} ({}/{})",
                    deposit.resource, deposit.available, deposit.amount
                )));
            }
        }
    }

    if object.is_some() {
        let map_object = object.unwrap();
        let structure = map_object.structure.as_ref();

        if structure.is_some() {
            let structure = structure.unwrap();
            let status = if map_object.construction_left > 0 {
                format!(
                    "[ {} ] building {}/{}",
                    structure.to_string(),
                    crate::game::CONSTRUCTION_TICKS - map_object.construction_left,
                    crate::game::CONSTRUCTION_TICKS
                )
            } else if map_object.active {
                format!("[ {} ] active", structure.to_string())
            } else {
                format!("[ {} ] idle", structure.to_string())
            };
            items.push(ListItem::new(status));

            match structure {
                Structure::Base { structure } => {
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
                Structure::Mine { structure } => {
                    items.push(ListItem::new(format_mine_resource(structure.resource())));
                }
                Structure::Refinery { .. } => {}
                Structure::Factory { .. } => {}
                Structure::Storage { structure } => {
                    for resource in structure.blueprint().resources() {
                        items.push(ListItem::new(format_resource_capacity(
                            structure.blueprint(),
                            resource,
                        )));
                    }
                }
            }
        }
    }

    List::new(items)
        .block(block)
        .style(Style::default().fg(Color::White))
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
        Structure::Factory { .. } => 'F',
        Structure::Refinery { .. } => 'R',
        Structure::Storage { .. } => 'S',
    }
}

pub fn render_map(
    map: &GameMap,
    objects: &ObjectManager,
    cursor: Position,
    camera: Position,
    view_width: u16,
    view_height: u16,
) -> Vec<Spans<'static>> {
    let cache = map.cache();
    let world_h = map.height() as i16;
    let world_w = map.width() as i16;
    let view_w = view_width as i16;
    let view_h = view_height as i16;

    let mut rows = Vec::with_capacity(view_height as usize);

    for row_i in 0..view_h {
        let world_y = camera.y + row_i;
        let mut spans: Vec<Span> = Vec::with_capacity(view_width as usize);

        if world_y < 0 || world_y >= world_h {
            for _ in 0..view_w {
                spans.push(Span::raw(" "));
            }
            rows.push(Spans::from(spans));
            continue;
        }

        let tile_row = &cache[world_y as usize];

        for col_i in 0..view_w {
            let world_x = camera.x + col_i;
            if world_x < 0 || world_x >= world_w {
                spans.push(Span::raw(" "));
                continue;
            }

            let tile = &tile_row[world_x as usize];
            let selected = cursor.y == world_y && cursor.x == world_x;
            let mut style = get_flora_style(&tile.flora);
            let position = Position::new(world_x, world_y);
            let object = objects.get(&position);

            if let Some(object) = object {
                if let Some(structure) = object.structure.as_ref() {
                    let mut symbol = get_structure_symbol(structure);
                    if object.construction_left > 0 {
                        symbol = symbol.to_ascii_lowercase();
                        style = style.fg(Color::DarkGray);
                    } else if object.active {
                        style = style.fg(Color::White);
                    } else {
                        style = style.fg(Color::Black);
                    }
                    if selected {
                        style = style.fg(Color::Red);
                    }
                    spans.push(Span::styled(symbol.to_string(), style));
                    continue;
                } else if object.deposit.is_some() {
                    if selected {
                        style = style.fg(Color::Red);
                    }
                    spans.push(Span::styled(
                        char::from(BlockType::Resource).to_string(),
                        style,
                    ));
                    continue;
                }
            }

            let block_symbol = if selected {
                BlockType::Selected
            } else {
                BlockType::Light
            };
            spans.push(Span::styled(char::from(block_symbol).to_string(), style));
        }

        rows.push(Spans::from(spans));
    }

    rows
}

pub fn draw_map_widget(text: &Vec<Spans<'static>>) -> Paragraph<'static> {
    Paragraph::new(text.clone())
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().bg(Color::Rgb(0, 0, 0)))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
}
