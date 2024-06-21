use std::time::Duration;

use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap};

use crate::game::{Flora, GameMap, MapTile, Position, ResourceStorage};
use crate::structures::{Structure, StructureGroup};

#[derive(Clone, Copy)]
pub enum BlockType {
    Full,
    Dark,
    Medium,
    Light,
}

impl From<BlockType> for char {
    fn from(block: BlockType) -> Self {
        match block {
            BlockType::Full => '█',
            BlockType::Dark => '▓',
            BlockType::Medium => '▓',
            BlockType::Light => '░',
        }
    }
}

pub struct Menu {
    items: Vec<StructureGroup>,
    selected: usize,
    selected_style: Style,
    default_style: Style,
}

impl Menu {
    pub fn new(items: Vec<StructureGroup>) -> Menu {
        let selected_style = Style::default().bg(Color::Red).fg(Color::White);
        let default_style = Style::default().bg(Color::Gray).fg(Color::Black);
        let selected = 0;

        Menu {
            items,
            selected,
            selected_style,
            default_style,
        }
    }

    fn get_item_span(&self, name: String, index: usize) -> Span {
        let style = if index == self.selected {
            self.selected_style
        } else {
            self.default_style
        };

        return Span::styled(name, style);
    }

    pub fn selected(&self) -> StructureGroup {
        return self.items[self.selected].clone();
    }

    pub fn items(&self) -> Vec<ListItem> {
        let list = self
            .items
            .iter()
            .enumerate()
            .map(|(index, structure_group)| {
                let content = self.get_item_span(structure_group.to_string(), index);
                ListItem::new(content)
            })
            .collect();

        return list;
    }

    pub fn next(&mut self) {
        if self.items.len() == 0 {
            return;
        }

        if self.selected == self.items.len() - 1 {
            self.selected = 0;
            return;
        }

        self.selected += 1;
    }

    pub fn previous(&mut self) {
        if self.items.len() == 0 {
            return;
        }

        if self.selected == 0 {
            self.selected = self.items.len() - 1;
            return;
        }

        self.selected -= 1;
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
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(area);

    return layout;
}

pub fn build_right_layout(area: Rect) -> Vec<Rect> {
    let layout = Layout::default()
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
    storage: &ResourceStorage,
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

    // Spacing
    items.push(ListItem::new("------------------"));

    for (resource, amount) in storage.list().iter() {
        let content = format!("{}: {}", resource, amount);
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

pub fn draw_build_menu_widget(menu: &Menu) -> List {
    let block = build_container_block("Build Menu".to_string());

    let list = List::new(menu.items())
        .block(block)
        .style(Style::default().fg(Color::White));

    return list;
}

pub fn draw_info_widget(position: Position, tile: MapTile) -> List<'static> {
    let block = build_container_block("Info".to_string());

    let tile_content = format!("Tile({}, {}):", position.x, position.y);
    let flora_content = format!("Flora: {}", tile.flora);

    let mut items = vec![ListItem::new(tile_content), ListItem::new(flora_content)];

    if tile.structure.is_some() {
        let structure_content = format!("Structure: {}", tile.structure.unwrap());
        items.push(ListItem::new(structure_content));
    }

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(Color::White));

    return list;
}

pub fn render_map(map: &GameMap, position: Position) -> Vec<Spans<'static>> {
    let y = position.y as usize;
    let x = position.x as usize;

    let water_style = Style::default().bg(Color::Rgb(32, 178, 170));
    let sand_style = Style::default().bg(Color::Yellow);
    let dirt_style = Style::default().bg(Color::Rgb(139, 69, 19));
    let grass_style = Style::default().bg(Color::Rgb(0, 128, 0));
    let rock_style = Style::default().bg(Color::Black);

    let selected_style = Style::default().bg(Color::White);

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

                    let style = match tile.flora {
                        Flora::Water => {
                            if selected == true {
                                selected_style
                            } else {
                                water_style
                            }
                        }
                        Flora::Sand => {
                            if selected == true {
                                selected_style
                            } else {
                                sand_style
                            }
                        }
                        Flora::Dirt => {
                            if selected == true {
                                selected_style
                            } else {
                                dirt_style
                            }
                        }
                        Flora::Grass => {
                            if selected == true {
                                selected_style
                            } else {
                                grass_style
                            }
                        }
                        Flora::Rock => {
                            if selected == true {
                                selected_style
                            } else {
                                rock_style
                            }
                        }
                    };

                    let block_symbol = match tile.flora {
                        Flora::Water => BlockType::Light,
                        Flora::Sand => BlockType::Light,
                        Flora::Dirt => BlockType::Light,
                        Flora::Grass => BlockType::Light,
                        Flora::Rock => BlockType::Light,
                    };

                    if tile.structure.is_some() {
                        let structure_symbol = match tile.structure.as_ref().unwrap() {
                            Structure::PowerPlant { .. } => 'P',
                            Structure::Mine { .. } => 'M',
                        };

                        return Span::styled(char::from(structure_symbol).to_string(), style);
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
