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

fn menu_styles(selected_bg: Color) -> (Style, Style) {
    (
        Style::default().bg(selected_bg).fg(Color::White),
        Style::default().bg(Color::Gray).fg(Color::Black),
    )
}

fn cycle_next(selected: usize, len: usize) -> usize {
    if len == 0 {
        selected
    } else if selected + 1 == len {
        0
    } else {
        selected + 1
    }
}

fn cycle_previous(selected: usize, len: usize) -> usize {
    if len == 0 {
        selected
    } else if selected == 0 {
        len - 1
    } else {
        selected - 1
    }
}

fn menu_item_style(
    index: usize,
    selected: usize,
    selected_style: Style,
    default_style: Style,
) -> Style {
    if index == selected {
        selected_style
    } else {
        default_style
    }
}

fn styled_menu_span(
    name: String,
    index: usize,
    selected: usize,
    selected_style: Style,
    default_style: Style,
) -> Span<'static> {
    Span::styled(
        name,
        menu_item_style(index, selected, selected_style, default_style),
    )
}

fn menu_list_items<T, F>(
    items: &[T],
    selected: usize,
    selected_style: Style,
    default_style: Style,
    mut label: F,
) -> Vec<ListItem<'static>>
where
    F: FnMut(&T) -> String,
{
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            ListItem::new(styled_menu_span(
                label(item),
                index,
                selected,
                selected_style,
                default_style,
            ))
        })
        .collect()
}

pub struct Menu {
    items: Vec<StructureGroup>,
    selected: usize,
    selected_style: Style,
    default_style: Style,
}

impl Menu {
    pub fn new(items: Vec<StructureGroup>) -> Menu {
        let (selected_style, default_style) = menu_styles(Color::Red);

        Menu {
            items,
            selected: 0,
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
        menu_list_items(
            &self.items,
            self.selected,
            self.selected_style,
            self.default_style,
            ToString::to_string,
        )
    }

    fn next(&mut self) {
        self.selected = cycle_next(self.selected, self.items.len());
    }

    fn previous(&mut self) {
        self.selected = cycle_previous(self.selected, self.items.len());
    }

    fn style(&self, name: String, index: usize) -> Span<'_> {
        styled_menu_span(
            name,
            index,
            self.selected,
            self.selected_style,
            self.default_style,
        )
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
        let (selected_style, default_style) = menu_styles(Color::Blue);

        MineResourceSelect {
            selected: 0,
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
        menu_list_items(
            &self.items,
            self.selected,
            self.selected_style,
            self.default_style,
            ToString::to_string,
        )
    }

    fn next(&mut self) {
        self.selected = cycle_next(self.selected, self.items.len());
    }

    fn previous(&mut self) {
        self.selected = cycle_previous(self.selected, self.items.len());
    }

    fn style(&self, name: String, index: usize) -> Span<'_> {
        styled_menu_span(
            name,
            index,
            self.selected,
            self.selected_style,
            self.default_style,
        )
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
        let (selected_style, default_style) = menu_styles(Color::Blue);

        RefineryResourceSelect {
            selected: 0,
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
        menu_list_items(
            &self.items,
            self.selected,
            self.selected_style,
            self.default_style,
            |group| group.iter().join(", "),
        )
    }

    fn next(&mut self) {
        self.selected = cycle_next(self.selected, self.items.len());
    }

    fn previous(&mut self) {
        self.selected = cycle_previous(self.selected, self.items.len());
    }

    fn style(&self, name: String, index: usize) -> Span<'_> {
        styled_menu_span(
            name,
            index,
            self.selected,
            self.selected_style,
            self.default_style,
        )
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
        let (selected_style, default_style) = menu_styles(Color::Blue);

        FactoryCommoditySelect {
            selected: 0,
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
        menu_list_items(
            &self.items,
            self.selected,
            self.selected_style,
            self.default_style,
            ToString::to_string,
        )
    }

    fn next(&mut self) {
        self.selected = cycle_next(self.selected, self.items.len());
    }

    fn previous(&mut self) {
        self.selected = cycle_previous(self.selected, self.items.len());
    }

    fn style(&self, name: String, index: usize) -> Span<'_> {
        styled_menu_span(
            name,
            index,
            self.selected,
            self.selected_style,
            self.default_style,
        )
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

    Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([
            Constraint::Length(left_width),
            Constraint::Length(map_width),
            Constraint::Length(right_width),
        ])
        .split(area)
}

fn split_layout(
    area: Rect,
    direction: Direction,
    constraints: impl Into<Vec<Constraint>>,
) -> Vec<Rect> {
    Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(area)
}

pub fn build_left_layout(area: Rect) -> Vec<Rect> {
    split_layout(
        area,
        Direction::Vertical,
        [Constraint::Percentage(40), Constraint::Percentage(60)],
    )
}

pub fn build_right_layout(area: Rect) -> Vec<Rect> {
    split_layout(
        area,
        Direction::Vertical,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
}

pub fn build_menu_layout(area: Rect) -> Vec<Rect> {
    split_layout(
        area,
        Direction::Horizontal,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
}

pub fn build_colony_layout(area: Rect) -> Vec<Rect> {
    split_layout(
        area,
        Direction::Horizontal,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
}

pub fn build_container_block(title: String) -> Block<'static> {
    Block::default()
        .title(format!(" [ {} ] ", title))
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
}

fn draw_list_widget(title: impl Into<String>, items: Vec<ListItem<'static>>) -> List<'static> {
    List::new(items)
        .block(build_container_block(title.into()))
        .style(Style::default().fg(Color::White))
}

fn section_header(title: &str) -> ListItem<'static> {
    ListItem::new(format!("{:-^30}", format!("[ {} ]", title)))
}

fn labeled_row(label: &str, value: impl ToString, label_width: usize) -> ListItem<'static> {
    ListItem::new(format!(
        "{:>label_width$}: {:>9}",
        label,
        value.to_string(),
        label_width = label_width
    ))
}

fn labeled_deficit_row(
    label: &str,
    amount: impl ToString,
    deficit: i64,
    label_width: usize,
) -> ListItem<'static> {
    ListItem::new(format!(
        "{:>label_width$}: {:>9} ({})",
        label,
        amount.to_string(),
        deficit.neg(),
        label_width = label_width
    ))
}

pub fn draw_stats_widget_left(
    storage: &ResourceManager,
    energy: &EnergyManager,
    elapsed: Duration,
    update_delta: u128,
    draw_delta: u128,
) -> List<'static> {
    let mut items = vec![
        ListItem::new(format!("Time: {:.1} (seconds)", elapsed.as_secs_f32())),
        ListItem::new(format!("Draw: {} (ms)", update_delta)),
        ListItem::new(format!("Update: {} (ms)", draw_delta)),
        section_header("Energy"),
        labeled_row("Output", energy.output(), 9),
        labeled_row("Stored", energy.stored(), 9),
        labeled_row("Deficit", (energy.deficit() as i64).neg(), 9),
        section_header("Resources"),
    ];

    for (resource, amount) in storage.resources() {
        items.push(labeled_deficit_row(
            &resource.to_string(),
            amount,
            storage.get_resource_deficit(resource) as i64,
            9,
        ));
    }

    draw_list_widget("Colony Information", items)
}

pub fn draw_stats_widget_right(storage: &ResourceManager) -> List<'static> {
    let mut items = vec![section_header("Manufactured")];

    for (manufactured, amount) in storage.manufactured() {
        items.push(labeled_deficit_row(
            &manufactured.to_string(),
            amount,
            storage.get_manufactured_deficit(manufactured) as i64,
            14,
        ));
    }

    items.push(section_header("Commodities"));
    for (commodity, amount) in storage.commodities() {
        items.push(labeled_deficit_row(
            &commodity.to_string(),
            amount,
            storage.get_commodity_deficit(commodity) as i64,
            14,
        ));
    }

    draw_list_widget("Colony Information", items)
}

pub fn draw_console_widget(
    text: &str,
    title: String,
    scroll_y: u16,
) -> Paragraph<'_> {
    Paragraph::new(text)
        .block(build_container_block(title))
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
    draw_list_widget("Build Menu", menu.items())
}

pub fn draw_mine_select_widget(menu: &MineResourceSelect) -> List<'static> {
    draw_list_widget("Mine Select", menu.items())
}

pub fn draw_factory_select_widget(menu: &FactoryCommoditySelect) -> List<'static> {
    draw_list_widget("Factory Select", menu.items())
}

pub fn draw_refinery_select_widget(menu: &RefineryResourceSelect) -> List<'static> {
    draw_list_widget("Refinery Select", menu.items())
}

pub fn format_mine_resource(resource_group: &Resource) -> String {
    format!("Resource: {}", resource_group.to_string())
}

fn format_capacity_row(label: &str, current: impl ToString, maximum: impl ToString) -> String {
    format!(
        "{:<10} ({:>8} / {:<8})",
        label,
        current.to_string(),
        maximum.to_string(),
    )
}

pub fn format_resource_capacity(
    blueprint: &StructureBlueprint,
    resource_group: &Resource,
) -> String {
    format_capacity_row(
        &resource_group.to_string(),
        ResourceStorageTrait::resource(blueprint, resource_group),
        ResourceStorageTrait::capacity(blueprint, resource_group),
    )
}

pub fn format_energy_io(blueprint: &StructureBlueprint) -> String {
    format_capacity_row("Energy I/O", blueprint.energy_in(), blueprint.energy_out())
}

pub fn format_battery(blueprint: &StructureBlueprint) -> String {
    format_capacity_row(
        "Battery",
        BatteryTrait::stored(blueprint),
        BatteryTrait::capacity(blueprint),
    )
}

pub fn draw_info_widget(
    position: Position,
    tile: &MapTile,
    object: Option<&MapObject>,
) -> List<'static> {
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

    draw_list_widget("Info", items)
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
