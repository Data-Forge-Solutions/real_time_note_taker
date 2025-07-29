#![warn(clippy::pedantic)]
//! # `real_time_note_taker`
//!
//! A reusable library providing a terminal user interface to take timestamped
//! notes in real time.
//!
//! ## Example
//!
//! ```no_run
//! use real_time_note_taker::{run, App};
//!
//! fn main() -> std::io::Result<()> {
//!     let app = App::new();
//!     run(app)
//! }
//! ```

mod app;
mod ui;

pub use app::{App, AppError, Entry, InputMode, Note, Section};
use std::io;

/// Runs the real-time note taking application.
///
/// # Arguments
/// * `app` - Initial application state.
///
/// # Errors
/// Propagates any terminal initialization or rendering errors.
///
/// # See also
/// [`App`] for manipulating the application state directly.
pub fn run(app: App) -> io::Result<()> {
    let mut terminal = ui::init_terminal()?;
    let res = ui::run_ui(&mut terminal, app);
    ui::restore_terminal(&mut terminal)?;
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_app() {
        let app = App::new();
        assert!(app.notes.is_empty());
    }
}
