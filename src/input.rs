// SPDX-FileCopyrightText: Copyright (c) 2021-2026 Igal Alkon
// SPDX-License-Identifier: Zlib

use std::io;
use std::time::Duration;

use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[derive(Debug, Clone)]
pub enum Input {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    CameraLeft,
    CameraRight,
    CameraUp,
    CameraDown,
    Confirm,
    Destroy,
    MenuNext,
    MenuPrevious,
    SelectNext,
    SelectPrevious,
    ToggleFollow,
    PinHome,
    GoHome,
    Help,
    LogPageUp,
    LogPageDown,
    Quit,
    Mouse(String),
    Resize { width: u16, height: u16 },
}

impl Input {
    fn allows_repeat(&self) -> bool {
        matches!(
            self,
            Input::MoveLeft
                | Input::MoveRight
                | Input::MoveUp
                | Input::MoveDown
                | Input::CameraLeft
                | Input::CameraRight
                | Input::CameraUp
                | Input::CameraDown
                | Input::MenuNext
                | Input::MenuPrevious
                | Input::SelectNext
                | Input::SelectPrevious
                | Input::LogPageUp
                | Input::LogPageDown
        )
    }
}

pub fn poll_inputs() -> io::Result<Vec<Input>> {
    let mut inputs = Vec::new();

    while poll(Duration::from_millis(0))? {
        match read()? {
            Event::Key(key) => {
                if let Some(input) = map_key(key) {
                    inputs.push(input);
                }
            }
            Event::Mouse(event) => {
                inputs.push(Input::Mouse(format!("{:?}", event)));
            }
            Event::Resize(width, height) => {
                inputs.push(Input::Resize { width, height });
            }
            _ => {}
        }
    }

    Ok(inputs)
}

fn map_key(event: KeyEvent) -> Option<Input> {
    let alt = event.modifiers.contains(KeyModifiers::ALT);
    let shift = event.modifiers.contains(KeyModifiers::SHIFT);

    let input = match event.code {
        KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') if alt => Input::CameraLeft,
        KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') if alt => Input::CameraRight,
        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') if alt => Input::CameraUp,
        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') if alt => Input::CameraDown,
        KeyCode::Left | KeyCode::Char('a') => Input::MoveLeft,
        KeyCode::Right | KeyCode::Char('d') => Input::MoveRight,
        KeyCode::Up | KeyCode::Char('w') => Input::MoveUp,
        KeyCode::Down | KeyCode::Char('s') => Input::MoveDown,
        KeyCode::Enter => Input::Confirm,
        KeyCode::Delete => Input::Destroy,
        KeyCode::Char(';') => Input::MenuPrevious,
        KeyCode::Char('\'') => Input::MenuNext,
        KeyCode::Tab if shift => Input::MenuPrevious,
        KeyCode::BackTab | KeyCode::Char('[') => Input::MenuPrevious,
        KeyCode::Tab | KeyCode::Char(']') => Input::MenuNext,
        KeyCode::PageUp => Input::LogPageUp,
        KeyCode::PageDown => Input::LogPageDown,
        KeyCode::Char('-') | KeyCode::Char(',') => Input::SelectPrevious,
        KeyCode::Char('=') | KeyCode::Char('+') | KeyCode::Char('.') | KeyCode::End => {
            Input::SelectNext
        }
        KeyCode::F(4) => Input::ToggleFollow,
        KeyCode::F(3) => Input::PinHome,
        KeyCode::Home => Input::GoHome,
        KeyCode::Char('?') => Input::Help,
        KeyCode::Esc => Input::Quit,
        _ => return None,
    };

    match event.kind {
        KeyEventKind::Press => Some(input),
        KeyEventKind::Repeat if input.allows_repeat() => Some(input),
        _ => None,
    }
}
