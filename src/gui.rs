use std::time::Duration;

use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap};

use crate::game::{Flora, GameMap, MapTile, Position, ResourceStorage};

pub(crate) fn build_main_layout(area: Rect) -> Vec<Rect> {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(area);

    return layout;
}

pub(crate) fn build_left_layout(area: Rect) -> Vec<Rect> {
    let layout = Layout::default()
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(area);

    return layout;
}

pub(crate) fn build_container_block(title: String) -> Block<'static> {
    let block = Block::default()
        .title(format!(" [ {} ] ", title))
        .borders(Borders::ALL);

    return block;
}

pub(crate) fn draw_stats_widget(
    storage: &ResourceStorage,
    position: Position,
    tile: MapTile,
    elapsed: Duration,
    update_delta: u128,
    draw_delta: u128,
) -> List {
    let time = format!(
        "Time: {:.1} (update: {}) (draw: {})",
        elapsed.as_secs_f32(),
        update_delta,
        draw_delta
    );

    let mut items = vec![ListItem::new(time.clone())];

    let position_content = format!("Position: ({}, {})", position.x, position.y);
    items.push(ListItem::new(position_content));

    let title_content = format!("Tile: [Flora: {}]", tile.flora);
    items.push(ListItem::new(title_content));

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

pub(crate) fn draw_console_widget(buffer: &String) -> Paragraph {
    let block = build_container_block("Console".to_string());

    let style = Style::default().fg(Color::Yellow);

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

pub(crate) fn draw_board_block() -> Block<'static> {
    let block = Block::default()
        .title(" [ Map ] ")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .style(Style::default().fg(Color::White));

    return block;
}

pub(crate) fn render_map(map: &GameMap, position: Position) -> Vec<Spans<'static>> {
    let y = position.y as usize;
    let x = position.x as usize;

    let water_style = Style::default().bg(Color::Blue);
    let sand_style = Style::default().bg(Color::Yellow);
    let dirt_style = Style::default().bg(Color::Red);
    let grass_style = Style::default().bg(Color::Green);
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

                    let symbol = match tile.flora {
                        Flora::Water => char::from('▒'),
                        Flora::Sand => char::from('▒'),
                        Flora::Dirt => char::from('▒'),
                        Flora::Grass => char::from('▒'),
                        Flora::Rock => char::from('▒'),
                        // ░ , ▓ , ▒
                    };

                    return Span::styled(String::from(symbol), style);
                })
                .collect();
            return Spans::from(spans);
        })
        .collect();

    return text;
}

pub(crate) fn draw_map_widget(text: &Vec<Spans<'static>>) -> Paragraph<'static> {
    let p = Paragraph::new(text.clone())
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    return p;
}
