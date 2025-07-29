#![warn(clippy::pedantic)]
use chrono::{DateTime, Local};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Represents a single note with timestamp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    /// The time the note was started.
    pub timestamp: DateTime<Local>,
    /// The contents of the note.
    pub text: String,
}

/// Represents a section with a title but no timestamp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Section {
    /// The section title.
    pub title: String,
}

/// A single entry which may be a note or a section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Prompting for a file path to save entries.
    Saving,
    /// Prompting for a file path to load entries.
    Loading,
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
#[derive(Debug)]
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
    /// Directory used for saving and loading files.
    pub save_dir: PathBuf,
    /// Available files when loading from disk.
    pub load_files: Vec<PathBuf>,
    /// Selected file index when loading.
    pub load_selected: usize,
}

impl Default for App {
    fn default() -> Self {
        let save_dir = Self::default_save_dir();
        let _ = fs::create_dir_all(&save_dir);
        Self {
            entries: Vec::new(),
            notes: Vec::new(),
            input: String::new(),
            mode: InputMode::Normal,
            note_time: None,
            selected: None,
            edit_index: None,
            save_dir,
            load_files: Vec::new(),
            load_selected: 0,
        }
    }
}

impl App {
    /// Returns the directory where files are saved by default.
    #[must_use]
    pub fn default_save_dir() -> PathBuf {
        if let Some(dirs) = ProjectDirs::from("com", "DataForge", "rtnt") {
            dirs.data_local_dir().to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        }
    }
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

    /// Begin entering a file path to save the current entries.
    pub fn start_save(&mut self) {
        self.input = self.save_dir.join("notes.csv").to_string_lossy().into();
        self.mode = InputMode::Saving;
    }

    /// Begin entering a file path to load entries from.
    pub fn start_load(&mut self) {
        self.load_files = fs::read_dir(&self.save_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| {
                let p = e.ok()?.path();
                if p.extension().and_then(|s| s.to_str()) == Some("csv") {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();
        self.load_selected = 0;
        self.mode = InputMode::Loading;
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

    /// Saves all entries to a CSV file.
    ///
    /// # Errors
    /// Returns any I/O or serialization errors encountered.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(false)
            .from_path(path)?;
        for e in &self.entries {
            match e {
                Entry::Note(n) => {
                    wtr.serialize(("note", n.timestamp.to_rfc3339(), &n.text))?;
                }
                Entry::Section(s) => {
                    wtr.serialize(("section", "", &s.title))?;
                }
            }
        }
        wtr.flush()?;
        Ok(())
    }

    /// Loads entries from a CSV file, replacing existing ones.
    ///
    /// # Errors
    /// Returns any I/O or deserialization errors encountered.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)?;
        let mut app = Self::default();
        for result in rdr.records() {
            let record = result?;
            match record.get(0) {
                Some("note") => {
                    if let (Some(ts), Some(text)) = (record.get(1), record.get(2)) {
                        let ts = DateTime::parse_from_rfc3339(ts)
                            .map_err(io::Error::other)?
                            .with_timezone(&Local);
                        let note = Note { timestamp: ts, text: text.to_string() };
                        app.notes.push(note.clone());
                        app.entries.push(Entry::Note(note));
                    }
                }
                Some("section") => {
                    if let Some(title) = record.get(2) {
                        app.entries.push(Entry::Section(Section {
                            title: title.to_string(),
                        }));
                    }
                }
                _ => {}
            }
        }
        Ok(app)
    }

    /// Loads entries from a file, replacing the current state.
    ///
    /// # Errors
    /// Returns any I/O or deserialization errors encountered.
    pub fn load_from_file_in_place<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let new_app = Self::load_from_file(path)?;
        *self = new_app;
        Ok(())
    }

    /// Processes a key event in normal mode.
    fn handle_normal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.select_previous(),
            KeyCode::Down => self.select_next(),
            KeyCode::Char('e') => self.edit_selected(),
            KeyCode::Enter => self.start_note(),
            KeyCode::Char('s') => self.start_section(),
            KeyCode::Char('w') => self.start_save(),
            KeyCode::Char('l') => self.start_load(),
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
                InputMode::Saving => {
                    let path: String = self.input.drain(..).collect();
                    if !path.is_empty() {
                        self.save_to_file(path).ok();
                    }
                    self.mode = InputMode::Normal;
                }
                InputMode::Normal | InputMode::Loading => {}
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

    fn handle_loading_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.load_selected > 0 {
                    self.load_selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.load_selected + 1 < self.load_files.len() {
                    self.load_selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(path) = self.load_files.get(self.load_selected).cloned() {
                    self.load_from_file_in_place(path).ok();
                }
                self.mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                self.mode = InputMode::Normal;
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
                InputMode::Loading => self.handle_loading_key(key),
                InputMode::EditingNote
                | InputMode::EditingSection
                | InputMode::EditingExistingNote
                | InputMode::EditingExistingSection
                | InputMode::Saving => self.handle_editing_key(key),
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

    #[test]
    fn save_and_load() {
        let mut app = App::new();
        app.start_note();
        app.input.push_str("hello");
        app.finalize_note();
        app.start_section();
        app.input.push_str("section");
        app.finalize_section();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notes.csv");
        app.save_to_file(&path).unwrap();

        let loaded = App::load_from_file(&path).unwrap();
        assert_eq!(loaded.entries, app.entries);
        assert_eq!(loaded.notes, app.notes);
    }
}
