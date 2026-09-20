//! Terminal backend selection for `tui`.
//!
//! Availability is feature-gated so the crate compiles with either backend:
//! - `crossterm` (default) — Windows, macOS, Linux
//! - `termion` — Unix-like terminals
//!
//! If both features are enabled, crossterm wins. `setup` / `restore` stay
//! public, so this module compiles even when `main` does not call it yet.

use std::io;
use tui::Terminal;

#[cfg(feature = "crossterm")]
mod backend {
    use super::*;
    use std::io::Stdout;

    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
    use tui::backend::CrosstermBackend;

    pub type Backend = CrosstermBackend<Stdout>;

    pub(crate) fn setup() -> io::Result<Terminal<Backend>> {
        enable_raw_mode()?;
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(terminal)
    }

    pub(crate) fn restore(terminal: &mut Terminal<Backend>) -> io::Result<()> {
        terminal.clear()?;
        disable_raw_mode()?;
        Ok(())
    }
}

#[cfg(all(feature = "termion", not(feature = "crossterm")))]
mod backend {
    use super::*;
    use std::io::Stdout;

    use termion::raw::{IntoRawMode, RawTerminal};
    use tui::backend::TermionBackend;

    pub type Backend = TermionBackend<RawTerminal<Stdout>>;

    pub(crate) fn setup() -> io::Result<Terminal<Backend>> {
        let stdout = io::stdout().into_raw_mode()?;
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(terminal)
    }

    pub(crate) fn restore(terminal: &mut Terminal<Backend>) -> io::Result<()> {
        terminal.clear()?;
        Ok(())
    }
}

#[cfg(not(any(feature = "crossterm", feature = "termion")))]
compile_error!("Enable a terminal backend: feature `crossterm` (default) or `termion`.");

#[cfg(any(feature = "crossterm", feature = "termion"))]
pub(crate) use backend::{restore, setup, Backend};
