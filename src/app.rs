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
    /// Editing an existing note.
    EditingExistingNote,
    /// Editing an existing section title.
    EditingExistingSection,
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
    /// Selected entry index when navigating.
    selected: Option<usize>,
    /// Index of the entry currently being edited if editing an existing entry.
    edit_index: Option<usize>,
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

    /// Returns the currently selected entry index if any.
    #[must_use]
    pub const fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Move selection to the previous entry if any.
    pub fn select_previous(&mut self) {
        match self.selected {
            Some(0) | None => {}
            Some(i) => self.selected = Some(i - 1),
        }
    }

    /// Move selection to the next entry if any.
    pub fn select_next(&mut self) {
        match self.selected {
            Some(i) if i + 1 < self.entries.len() => self.selected = Some(i + 1),
            None if !self.entries.is_empty() => self.selected = Some(0),
            _ => {}
        }
    }

    /// Begin editing the selected entry if any.
    pub fn edit_selected(&mut self) {
        if let Some(idx) = self.selected {
            match &self.entries[idx] {
                Entry::Note(n) => {
                    self.input = n.text.clone();
                    self.note_time = Some(n.timestamp);
                    self.edit_index = Some(idx);
                    self.mode = InputMode::EditingExistingNote;
                }
                Entry::Section(s) => {
                    self.input = s.title.clone();
                    self.note_time = None;
                    self.edit_index = Some(idx);
                    self.mode = InputMode::EditingExistingSection;
                }
            }
        }
    }

    /// Starts a new note capturing the current timestamp.
    pub fn start_note(&mut self) {
        self.note_time = Some(Local::now());
        self.input.clear();
        self.edit_index = None;
        self.mode = InputMode::EditingNote;
    }

    /// Starts a new section entry.
    pub fn start_section(&mut self) {
        self.note_time = None;
        self.input.clear();
        self.edit_index = None;
        self.mode = InputMode::EditingSection;
    }

    /// Finalizes the note if editing, pushing it into the note list.
    pub fn finalize_note(&mut self) {
        if let Some(idx) = self.edit_index.take() {
            if let Entry::Note(ref mut n) = self.entries[idx] {
                n.text = self.input.drain(..).collect();
                if let Some(orig) = self
                    .notes
                    .iter_mut()
                    .find(|orig| orig.timestamp == n.timestamp)
                {
                    orig.text.clone_from(&n.text);
                }
            }
            self.mode = InputMode::Normal;
            self.note_time = None;
        } else if let Some(time) = self.note_time.take() {
            let note = Note {
                timestamp: time,
                text: self.input.drain(..).collect(),
            };
            self.notes.push(note.clone());
            self.entries.push(Entry::Note(note));
            self.selected = Some(self.entries.len() - 1);
            self.mode = InputMode::Normal;
        }
    }

    /// Finalizes a section entry.
    pub fn finalize_section(&mut self) {
        let title = self.input.drain(..).collect::<String>();
        if let Some(idx) = self.edit_index.take() {
            if let Entry::Section(ref mut s) = self.entries[idx] {
                s.title = title;
            }
            self.note_time = None;
        } else if !title.is_empty() {
            let section = Section { title };
            self.entries.push(Entry::Section(section));
            self.selected = Some(self.entries.len() - 1);
        }
        self.mode = InputMode::Normal;
    }

    /// Cancels the current entry editing.
    pub fn cancel_entry(&mut self) {
        self.input.clear();
        self.note_time = None;
        self.edit_index = None;
        self.mode = InputMode::Normal;
    }

    /// Processes a key event in normal mode.
    fn handle_normal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.select_previous(),
            KeyCode::Down => self.select_next(),
            KeyCode::Char('e') => self.edit_selected(),
            KeyCode::Enter => self.start_note(),
            KeyCode::Char('s') => self.start_section(),
            _ => {}
        }
    }

    /// Processes a key event in any editing mode.
    fn handle_editing_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => match self.mode {
                InputMode::EditingNote | InputMode::EditingExistingNote => {
                    self.finalize_note();
                }
                InputMode::EditingSection | InputMode::EditingExistingSection => {
                    self.finalize_section();
                }
                InputMode::Normal => {}
            },
            KeyCode::Esc => self.cancel_entry(),
            KeyCode::Char(c)
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                self.input.push(c);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            _ => {}
        }
    }

    /// Handles a terminal event.
    ///
    /// # Errors
    /// Propagates any I/O errors from the terminal event system.
    pub fn handle_event(&mut self, event: &Event) -> Result<(), AppError> {
        if let Event::Key(key) = *event {
            match self.mode {
                InputMode::Normal => self.handle_normal_key(key),
                InputMode::EditingNote
                | InputMode::EditingSection
                | InputMode::EditingExistingNote
                | InputMode::EditingExistingSection => self.handle_editing_key(key),
            }
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
