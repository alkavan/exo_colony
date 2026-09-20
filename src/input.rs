use std::io;
use std::time::Duration;

use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind};

#[derive(Debug, Clone)]
pub enum Input {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Confirm,
    Destroy,
    MenuNext,
    MenuPrevious,
    SelectNext,
    SelectPrevious,
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
                | Input::MenuNext
                | Input::MenuPrevious
                | Input::SelectNext
                | Input::SelectPrevious
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
    let input = match event.code {
        KeyCode::Left | KeyCode::Char('a') => Input::MoveLeft,
        KeyCode::Right | KeyCode::Char('d') => Input::MoveRight,
        KeyCode::Up | KeyCode::Char('w') => Input::MoveUp,
        KeyCode::Down | KeyCode::Char('s') => Input::MoveDown,
        KeyCode::Enter => Input::Confirm,
        KeyCode::Delete => Input::Destroy,
        KeyCode::PageUp => Input::MenuPrevious,
        KeyCode::PageDown => Input::MenuNext,
        KeyCode::Home => Input::SelectPrevious,
        KeyCode::End => Input::SelectNext,
        KeyCode::Esc => Input::Quit,
        _ => return None,
    };

    match event.kind {
        KeyEventKind::Press => Some(input),
        KeyEventKind::Repeat if input.allows_repeat() => Some(input),
        _ => None,
    }
}
