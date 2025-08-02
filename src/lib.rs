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
//!     let _app = run(app)?;
//!     Ok(())
//! }
//! ```

mod app;
mod key_utils;
mod theme;
mod ui;

pub use app::{Action, App, AppError, Entry, InputMode, KeyBindings, Note, Section};
pub use key_utils::{key_to_string, string_to_key};
use std::io;
pub use theme::{Theme, ThemeName};

/// Runs the real-time note taking application until the user exits.
///
/// # Arguments
/// * `app` - Initial [`App`] state to run.
///
/// # Returns
/// The final [`App`] state when the UI terminates.
///
/// # Examples
/// ```no_run
/// use real_time_note_taker::{run, App};
///
/// fn main() -> std::io::Result<()> {
///     let app = App::new();
///     let finished = run(app)?;
///     println!("{} notes", finished.notes.len());
///     Ok(())
/// }
/// ```
///
/// # Errors
/// Propagates any terminal initialization or rendering errors.
///
/// # See also
/// [`App::new`], [`App::save_to_file`], [`ui::init_terminal`], [`ui::run_ui`],
/// [`ui::restore_terminal`]
pub fn run(app: App) -> io::Result<App> {
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
