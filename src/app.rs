#![warn(clippy::pedantic)]
use chrono::{DateTime, Local};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::io;
use thiserror::Error;

/// Represents a single note with timestamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    /// The time the note was entered.
    pub timestamp: DateTime<Local>,
    /// The contents of the note.
    pub text: String,
}

/// Input mode for the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Normal mode where key events control the application.
    Normal,
    /// Editing mode for entering note text.
    Editing,
}

impl Default for InputMode {
    fn default() -> Self {
        Self::Normal
    }
}

/// Errors that can occur within the application.
#[derive(Error, Debug)]
pub enum AppError {
    /// I/O error.
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Main application state.
#[derive(Debug, Default)]
pub struct App {
    /// Collected notes.
    pub notes: Vec<Note>,
    /// Current input buffer when editing.
    input: String,
    /// Current mode.
    mode: InputMode,
    /// Timestamp captured when note editing started.
    note_time: Option<DateTime<Local>>,
}

impl App {
    /// Creates a new [`App`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns current input mode.
    #[must_use]
    pub const fn mode(&self) -> InputMode {
        self.mode
    }

    /// Returns current input buffer.
    #[must_use]
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Starts a new note capturing the current timestamp.
    pub fn start_note(&mut self) {
        self.note_time = Some(Local::now());
        self.input.clear();
        self.mode = InputMode::Editing;
    }

    /// Finalizes the note if editing, pushing it into the note list.
    pub fn finalize_note(&mut self) {
        if self.note_time.take().is_some() {
            self.notes.push(Note {
                timestamp: Local::now(),
                text: self.input.drain(..).collect(),
            });
            self.mode = InputMode::Normal;
        }
    }

    /// Cancels the current note editing.
    pub fn cancel_note(&mut self) {
        self.input.clear();
        self.note_time = None;
        self.mode = InputMode::Normal;
    }

    /// Handles a terminal event.
    ///
    /// # Errors
    /// Propagates any I/O errors from the terminal event system.
    pub fn handle_event(&mut self, event: Event) -> Result<(), AppError> {
        match (self.mode, event) {
            (
                InputMode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }),
            ) => {
                self.start_note();
            }
            (
                InputMode::Editing,
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }),
            ) => {
                self.finalize_note();
            }
            (
                InputMode::Editing,
                Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }),
            ) => {
                self.cancel_note();
            }
            (
                InputMode::Editing,
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    ..
                }),
            ) => {
                self.input.push(c);
            }
            (
                InputMode::Editing,
                Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                }),
            ) => {
                self.input.pop();
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_and_finalize_note() {
        let mut app = App::new();
        assert!(app.notes.is_empty());
        app.start_note();
        assert!(matches!(app.mode(), InputMode::Editing));
        assert!(app.note_time.is_some());
        app.input.push_str("test");
        app.finalize_note();
        assert!(app.note_time.is_none());
        assert!(matches!(app.mode(), InputMode::Normal));
        assert_eq!(app.notes.len(), 1);
        assert_eq!(app.notes[0].text, "test");
    }

    #[test]
    fn cancel_note() {
        let mut app = App::new();
        app.start_note();
        app.input.push_str("discard");
        app.cancel_note();
        assert!(app.notes.is_empty());
        assert!(app.input.is_empty());
        assert!(app.note_time.is_none());
        assert!(matches!(app.mode(), InputMode::Normal));
    }

    #[test]
    fn note_timestamp_on_finalize() {
        use std::time::Duration;

        let mut app = App::new();
        app.start_note();
        let start_time = app.note_time.unwrap();
        std::thread::sleep(Duration::from_millis(10));
        app.input.push_str("entry");
        app.finalize_note();
        let finalized = app.notes[0].timestamp;
        assert!(finalized >= start_time);
        assert_ne!(finalized, start_time);
    }
}
