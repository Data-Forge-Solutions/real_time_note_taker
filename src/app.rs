#![warn(clippy::pedantic)]
use chrono::{DateTime, Local};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::io;
use thiserror::Error;

/// Represents a single note with timestamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    /// The time the note was started.
    pub timestamp: DateTime<Local>,
    /// The contents of the note.
    pub text: String,
}

/// Represents a section with a title but no timestamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// The section title.
    pub title: String,
}

/// A single entry which may be a note or a section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// A timestamped note.
    Note(Note),
    /// A section title.
    Section(Section),
}

/// Input mode for the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Normal mode where key events control the application.
    Normal,
    /// Editing mode for entering a note.
    EditingNote,
    /// Editing mode for entering a section title.
    EditingSection,
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
    /// Collected entries in the order they were created.
    pub entries: Vec<Entry>,
    /// Collected notes for convenience.
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

    /// Returns the timestamp of the note being edited if present.
    #[must_use]
    pub fn note_time(&self) -> Option<DateTime<Local>> {
        self.note_time
    }

    /// Starts a new note capturing the current timestamp.
    pub fn start_note(&mut self) {
        self.note_time = Some(Local::now());
        self.input.clear();
        self.mode = InputMode::EditingNote;
    }

    /// Starts a new section entry.
    pub fn start_section(&mut self) {
        self.note_time = None;
        self.input.clear();
        self.mode = InputMode::EditingSection;
    }

    /// Finalizes the note if editing, pushing it into the note list.
    pub fn finalize_note(&mut self) {
        if let Some(time) = self.note_time.take() {
            let note = Note {
                timestamp: time,
                text: self.input.drain(..).collect(),
            };
            self.notes.push(note.clone());
            self.entries.push(Entry::Note(note));
            self.mode = InputMode::Normal;
        }
    }

    /// Finalizes a section entry.
    pub fn finalize_section(&mut self) {
        let title = self.input.drain(..).collect::<String>();
        if !title.is_empty() {
            let section = Section { title };
            self.entries.push(Entry::Section(section));
        }
        self.mode = InputMode::Normal;
    }

    /// Cancels the current entry editing.
    pub fn cancel_entry(&mut self) {
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
                InputMode::Normal,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('s'),
                    ..
                }),
            ) => {
                self.start_section();
            }
            (
                InputMode::EditingNote,
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }),
            ) => {
                self.finalize_note();
            }
            (
                InputMode::EditingSection,
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }),
            ) => {
                self.finalize_section();
            }
            (
                InputMode::EditingNote | InputMode::EditingSection,
                Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }),
            ) => {
                self.cancel_entry();
            }
            (
                InputMode::EditingNote | InputMode::EditingSection,
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    ..
                }),
            ) => {
                self.input.push(c);
            }
            (
                InputMode::EditingNote | InputMode::EditingSection,
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
        assert!(matches!(app.mode(), InputMode::EditingNote));
        assert!(app.note_time.is_some());
        app.input.push_str("test");
        app.finalize_note();
        assert!(app.note_time.is_none());
        assert!(matches!(app.mode(), InputMode::Normal));
        assert_eq!(app.notes.len(), 1);
        assert_eq!(app.notes[0].text, "test");
    }

    #[test]
    fn cancel_entry() {
        let mut app = App::new();
        app.start_note();
        app.input.push_str("discard");
        app.cancel_entry();
        assert!(app.notes.is_empty());
        assert!(app.input.is_empty());
        assert!(app.note_time.is_none());
        assert!(matches!(app.mode(), InputMode::Normal));
    }
}
