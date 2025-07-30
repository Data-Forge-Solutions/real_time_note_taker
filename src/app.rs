#![warn(clippy::pedantic)]
use chrono::{DateTime, Duration, Local, NaiveTime, TimeZone};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;
use thiserror::Error;

use crate::ThemeName;

fn key_to_string(key: KeyCode) -> String {
    match key {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Null => "Null".to_string(),
        other => format!("{other:?}"),
    }
}

fn string_to_key(s: &str) -> Option<KeyCode> {
    match s {
        "Enter" => Some(KeyCode::Enter),
        "Esc" => Some(KeyCode::Esc),
        "Up" => Some(KeyCode::Up),
        "Down" => Some(KeyCode::Down),
        "Left" => Some(KeyCode::Left),
        "Right" => Some(KeyCode::Right),
        "Null" => Some(KeyCode::Null),
        c if c.len() == 1 => c.chars().next().map(KeyCode::Char),
        _ => None,
    }
}

fn default_theme_key() -> String {
    key_to_string(KeyCode::Char('t'))
}

fn default_time_hack_key() -> String {
    key_to_string(KeyCode::Char('h'))
}

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
    /// Entering a time hack value.
    TimeHack,
    /// Selecting a key binding to change.
    KeyBindings,
    /// Capturing a new key for a binding.
    KeyCapture,
    /// Confirming replacement of an existing binding.
    ConfirmReplace,
    /// Warning shown when the bindings key would be lost.
    BindWarning,
    /// Selecting a color theme.
    ThemeSelect,
}

/// Actions that can be bound to keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Edit,
    NewNote,
    NewSection,
    Save,
    Load,
    Quit,
    Cancel,
    Bindings,
    Theme,
    /// Open the Time Hack input.
    TimeHack,
}

impl Action {
    pub const ALL: [Action; 12] = [
        Action::Up,
        Action::Down,
        Action::Edit,
        Action::NewNote,
        Action::NewSection,
        Action::Save,
        Action::Load,
        Action::Quit,
        Action::Cancel,
        Action::Bindings,
        Action::Theme,
        Action::TimeHack,
    ];
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Action::Up => "Up",
            Action::Down => "Down",
            Action::Edit => "Edit",
            Action::NewNote => "New Note",
            Action::NewSection => "New Section",
            Action::Save => "Save",
            Action::Load => "Load",
            Action::Quit => "Quit",
            Action::Cancel => "Cancel",
            Action::Bindings => "Key Menu",
            Action::Theme => "Theme Menu",
            Action::TimeHack => "Time Hack",
        };
        write!(f, "{name}")
    }
}

/// Collection of keys that control navigation and editing.
#[derive(Debug, Clone)]
pub struct KeyBindings {
    /// Move selection up or previous.
    pub up: KeyCode,
    /// Move selection down or next.
    pub down: KeyCode,
    /// Edit the currently selected entry.
    pub edit: KeyCode,
    /// Start entering a new note.
    pub new_note: KeyCode,
    /// Start entering a new section.
    pub new_section: KeyCode,
    /// Begin saving entries to a file.
    pub save: KeyCode,
    /// Begin loading entries from a file.
    pub load: KeyCode,
    /// Quit the application.
    pub quit: KeyCode,
    /// Cancel the current edit.
    pub cancel: KeyCode,
    /// Open the key bindings menu.
    pub bindings: KeyCode,
    /// Open the theme selection menu.
    pub theme: KeyCode,
    /// Open the time hack input.
    pub time_hack: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            up: KeyCode::Up,
            down: KeyCode::Down,
            edit: KeyCode::Char('e'),
            new_note: KeyCode::Enter,
            new_section: KeyCode::Char('s'),
            save: KeyCode::Char('w'),
            load: KeyCode::Char('l'),
            quit: KeyCode::Char('q'),
            cancel: KeyCode::Esc,
            bindings: KeyCode::Char('b'),
            theme: KeyCode::Char('t'),
            time_hack: KeyCode::Char('h'),
        }
    }
}

impl KeyBindings {
    fn config_path() -> PathBuf {
        if let Some(dirs) = ProjectDirs::from("com", "DFS", "rtnt") {
            dirs.config_dir().join("keybindings.json")
        } else {
            PathBuf::from("keybindings.json")
        }
    }

    #[cfg(test)]
    pub fn config_path_for_test() -> PathBuf {
        Self::config_path()
    }

    /// Loads key bindings from the configuration file or returns defaults.
    #[must_use]
    pub fn load_or_default() -> Self {
        let path = Self::config_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<KeyBindingsConfig>(&data) {
                return cfg.into();
            }
        }
        Self::default()
    }

    /// Saves the current key bindings to the configuration file.
    pub fn save(&self) {
        if let Some(parent) = Self::config_path().parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(&KeyBindingsConfig::from(self.clone())) {
            let _ = fs::write(Self::config_path(), data);
        }
    }

    /// Returns the key bound to the given action.
    #[must_use]
    pub fn get(&self, action: Action) -> KeyCode {
        match action {
            Action::Up => self.up,
            Action::Down => self.down,
            Action::Edit => self.edit,
            Action::NewNote => self.new_note,
            Action::NewSection => self.new_section,
            Action::Save => self.save,
            Action::Load => self.load,
            Action::Quit => self.quit,
            Action::Cancel => self.cancel,
            Action::Bindings => self.bindings,
            Action::Theme => self.theme,
            Action::TimeHack => self.time_hack,
        }
    }

    /// Sets the key for the given action.
    pub fn set(&mut self, action: Action, key: KeyCode) {
        match action {
            Action::Up => self.up = key,
            Action::Down => self.down = key,
            Action::Edit => self.edit = key,
            Action::NewNote => self.new_note = key,
            Action::NewSection => self.new_section = key,
            Action::Save => self.save = key,
            Action::Load => self.load = key,
            Action::Quit => self.quit = key,
            Action::Cancel => self.cancel = key,
            Action::Bindings => self.bindings = key,
            Action::Theme => self.theme = key,
            Action::TimeHack => self.time_hack = key,
        }
    }

    /// Returns the action currently bound to the specified key if any.
    #[must_use]
    pub fn action_for_key(&self, key: KeyCode) -> Option<Action> {
        if self.up == key {
            Some(Action::Up)
        } else if self.down == key {
            Some(Action::Down)
        } else if self.edit == key {
            Some(Action::Edit)
        } else if self.new_note == key {
            Some(Action::NewNote)
        } else if self.new_section == key {
            Some(Action::NewSection)
        } else if self.save == key {
            Some(Action::Save)
        } else if self.load == key {
            Some(Action::Load)
        } else if self.quit == key {
            Some(Action::Quit)
        } else if self.cancel == key {
            Some(Action::Cancel)
        } else if self.bindings == key {
            Some(Action::Bindings)
        } else if self.theme == key {
            Some(Action::Theme)
        } else if self.time_hack == key {
            Some(Action::TimeHack)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize)]
struct KeyBindingsConfig {
    up: String,
    down: String,
    edit: String,
    new_note: String,
    new_section: String,
    save: String,
    load: String,
    quit: String,
    cancel: String,
    bindings: String,
    #[serde(default = "default_theme_key")]
    theme: String,
    #[serde(default = "default_time_hack_key")]
    time_hack: String,
}

impl From<KeyBindings> for KeyBindingsConfig {
    fn from(k: KeyBindings) -> Self {
        Self {
            up: key_to_string(k.up),
            down: key_to_string(k.down),
            edit: key_to_string(k.edit),
            new_note: key_to_string(k.new_note),
            new_section: key_to_string(k.new_section),
            save: key_to_string(k.save),
            load: key_to_string(k.load),
            quit: key_to_string(k.quit),
            cancel: key_to_string(k.cancel),
            bindings: key_to_string(k.bindings),
            theme: key_to_string(k.theme),
            time_hack: key_to_string(k.time_hack),
        }
    }
}

impl From<KeyBindingsConfig> for KeyBindings {
    fn from(c: KeyBindingsConfig) -> Self {
        Self {
            up: string_to_key(&c.up).unwrap_or(KeyCode::Up),
            down: string_to_key(&c.down).unwrap_or(KeyCode::Down),
            edit: string_to_key(&c.edit).unwrap_or(KeyCode::Char('e')),
            new_note: string_to_key(&c.new_note).unwrap_or(KeyCode::Enter),
            new_section: string_to_key(&c.new_section).unwrap_or(KeyCode::Char('s')),
            save: string_to_key(&c.save).unwrap_or(KeyCode::Char('w')),
            load: string_to_key(&c.load).unwrap_or(KeyCode::Char('l')),
            quit: string_to_key(&c.quit).unwrap_or(KeyCode::Char('q')),
            cancel: string_to_key(&c.cancel).unwrap_or(KeyCode::Esc),
            bindings: string_to_key(&c.bindings).unwrap_or(KeyCode::Char('b')),
            theme: string_to_key(&c.theme).unwrap_or(KeyCode::Char('t')),
            time_hack: string_to_key(&c.time_hack).unwrap_or(KeyCode::Char('h')),
        }
    }
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

/// Main application state for the note taking application.
///
/// `App` stores all user facing data such as the list of notes, current input
/// buffer and UI navigation state.  It can be persisted to disk using
#[derive(Debug)]
pub struct App {
    /// Collected entries in the order they were created.
    pub entries: Vec<Entry>,
    /// Collected notes for convenience.
    pub notes: Vec<Note>,
    /// Current input buffer when editing.
    input: String,
    /// Cursor position within the input buffer.
    cursor: usize,
    /// Current mode.
    mode: InputMode,
    /// Timestamp captured when note editing started.
    note_time: Option<DateTime<Local>>,
    /// Active time hack offset.
    time_hack: Option<(NaiveTime, Instant)>,
    /// Selected entry index when navigating.
    selected: Option<usize>,
    /// Index of the entry currently being edited if editing an existing entry.
    edit_index: Option<usize>,
    /// Key bindings controlling the application.
    pub keys: KeyBindings,
    /// Directory used for saving and loading files.
    pub save_dir: PathBuf,
    /// Available files when loading from disk.
    pub load_files: Vec<PathBuf>,
    /// Selected file index when loading.
    pub load_selected: usize,
    /// Selected binding index when editing key bindings.
    pub keybind_selected: usize,
    /// Action being re-bound when capturing a key.
    pub capture_action: Option<Action>,
    /// Key to assign after confirmation.
    pub pending_key: Option<KeyCode>,
    /// Action being assigned during confirmation.
    pub pending_action: Option<Action>,
    /// Existing action that currently uses the pending key.
    pub pending_conflict: Option<Action>,
    /// Selected color theme.
    pub theme: ThemeName,
    /// Selected index when choosing a theme.
    pub theme_selected: usize,
}

impl Default for App {
    fn default() -> Self {
        let save_dir = Self::default_save_dir();
        let _ = fs::create_dir_all(&save_dir);
        Self {
            entries: Vec::new(),
            notes: Vec::new(),
            input: String::new(),
            cursor: 0,
            mode: InputMode::Normal,
            note_time: None,
            time_hack: None,
            selected: None,
            edit_index: None,
            keys: KeyBindings::load_or_default(),
            save_dir,
            load_files: Vec::new(),
            load_selected: 0,
            keybind_selected: 0,
            capture_action: None,
            pending_key: None,
            pending_action: None,
            pending_conflict: None,
            theme: ThemeName::load_or_default(),
            theme_selected: 0,
        }
    }
}

impl App {
    /// Returns the directory where files are saved by default.
    ///
    /// # Returns
    /// Path to the directory used for storing note CSV files.
    ///
    /// # Examples
    /// ```
    /// use real_time_note_taker::App;
    ///
    /// let path = App::default_save_dir();
    /// println!("{:?}", path);
    /// ```
    ///
    /// # See also
    /// [`App::save_to_file`], [`App::load_from_file`]
    #[must_use]
    pub fn default_save_dir() -> PathBuf {
        if let Some(dirs) = ProjectDirs::from("com", "DFS", "rtnt") {
            dirs.data_local_dir().to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        }
    }
    /// Creates a new [`App`] with default settings.
    ///
    /// # Returns
    /// A fresh [`App`] ready to be passed to [`crate::run`].
    ///
    /// # Examples
    /// ```
    /// use real_time_note_taker::App;
    ///
    /// let app = App::new();
    /// assert!(app.notes.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`App`] with custom key bindings.
    #[must_use]
    pub fn with_keybindings(keys: KeyBindings) -> Self {
        Self {
            keys,
            ..Self::default()
        }
    }

    /// Replaces the key bindings with the provided ones.
    pub fn set_keybindings(&mut self, keys: KeyBindings) {
        self.keys = keys;
    }

    /// Returns the current key bindings.
    #[must_use]
    pub fn keybindings(&self) -> &KeyBindings {
        &self.keys
    }

    /// Returns the active theme configuration.
    #[must_use]
    pub fn theme(&self) -> crate::Theme {
        self.theme.theme()
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

    /// Returns the cursor position within the input buffer.
    #[must_use]
    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    /// Returns the timestamp of the note being edited if present.
    #[must_use]
    pub fn note_time(&self) -> Option<DateTime<Local>> {
        self.note_time
    }

    /// Returns the active time hack if set.
    #[must_use]
    pub fn time_hack(&self) -> Option<(NaiveTime, Instant)> {
        self.time_hack
    }

    /// Returns the current timestamp considering any active time hack.
    #[must_use]
    pub fn current_time(&self) -> DateTime<Local> {
        if let Some((base, start)) = self.time_hack {
            let delta = Instant::now().saturating_duration_since(start);
            let base_dt = Local::now().date_naive().and_time(base);
            let base = Local.from_local_datetime(&base_dt).unwrap();
            base + Duration::from_std(delta).unwrap()
        } else {
            Local::now()
        }
    }

    /// Returns the current time source name.
    #[must_use]
    pub fn time_source(&self) -> &'static str {
        if self.time_hack.is_some() {
            "Hacked"
        } else {
            "System"
        }
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
                    self.cursor = self.input.len();
                    self.note_time = Some(n.timestamp);
                    self.edit_index = Some(idx);
                    self.mode = InputMode::EditingExistingNote;
                }
                Entry::Section(s) => {
                    self.input = s.title.clone();
                    self.cursor = self.input.len();
                    self.note_time = None;
                    self.edit_index = Some(idx);
                    self.mode = InputMode::EditingExistingSection;
                }
            }
        }
    }

    /// Starts a new note capturing the current timestamp.
    pub fn start_note(&mut self) {
        self.note_time = Some(self.current_time());
        self.input.clear();
        self.cursor = 0;
        self.edit_index = None;
        self.mode = InputMode::EditingNote;
    }

    /// Starts a new section entry.
    pub fn start_section(&mut self) {
        self.note_time = None;
        self.input.clear();
        self.cursor = 0;
        self.edit_index = None;
        self.mode = InputMode::EditingSection;
    }

    /// Begin entering a file path to save the current entries.
    pub fn start_save(&mut self) {
        self.input = self.save_dir.join("notes.csv").to_string_lossy().into();
        self.cursor = self.input.len();
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
        self.input.clear();
        self.cursor = 0;
        self.mode = InputMode::Loading;
    }

    /// Open the key bindings menu.
    pub fn start_keybindings(&mut self) {
        self.keybind_selected = 0;
        self.mode = InputMode::KeyBindings;
    }

    /// Open the theme selection menu.
    pub fn start_theme_menu(&mut self) {
        self.theme_selected = ThemeName::ALL
            .iter()
            .position(|t| *t == self.theme)
            .unwrap_or(0);
        self.mode = InputMode::ThemeSelect;
    }

    /// Open the time hack input.
    pub fn start_time_hack(&mut self) {
        self.input.clear();
        self.cursor = 0;
        self.mode = InputMode::TimeHack;
    }

    fn start_capture_binding(&mut self) {
        if let Some(action) = Action::ALL.get(self.keybind_selected).copied() {
            self.capture_action = Some(action);
            self.mode = InputMode::KeyCapture;
        }
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
            self.cursor = 0;
        } else if let Some(time) = self.note_time.take() {
            let note = Note {
                timestamp: time,
                text: self.input.drain(..).collect(),
            };
            self.notes.push(note.clone());
            self.entries.push(Entry::Note(note));
            self.selected = Some(self.entries.len() - 1);
            self.mode = InputMode::Normal;
            self.cursor = 0;
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
        self.cursor = 0;
    }

    fn finalize_time_hack(&mut self) {
        if self.input.is_empty() {
            self.time_hack = None;
        } else if let Ok(time) = NaiveTime::parse_from_str(&self.input, "%H:%M:%S%.f")
            .or_else(|_| NaiveTime::parse_from_str(&self.input, "%H:%M:%S"))
        {
            let now = Local::now();
            self.time_hack = Some((time, Instant::now()));
            let hacked = self.current_time();
            let note = Note {
                timestamp: now,
                text: format!(
                    "Time Hack: {} -> {}",
                    now.format("%H:%M:%S"),
                    hacked.format("%H:%M:%S")
                ),
            };
            self.notes.push(note.clone());
            self.entries.push(Entry::Note(note));
            self.selected = Some(self.entries.len() - 1);
        }
        self.input.clear();
        self.cursor = 0;
        self.mode = InputMode::Normal;
    }

    /// Cancels the current entry editing.
    pub fn cancel_entry(&mut self) {
        self.input.clear();
        self.note_time = None;
        self.edit_index = None;
        self.cursor = 0;
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
                        let note = Note {
                            timestamp: ts,
                            text: text.to_string(),
                        };
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
            c if c == self.keys.up => self.select_previous(),
            c if c == self.keys.down => self.select_next(),
            c if c == self.keys.edit => self.edit_selected(),
            c if c == self.keys.new_note => self.start_note(),
            c if c == self.keys.new_section => self.start_section(),
            c if c == self.keys.save => self.start_save(),
            c if c == self.keys.load => self.start_load(),
            c if c == self.keys.bindings => self.start_keybindings(),
            c if c == self.keys.theme => self.start_theme_menu(),
            c if c == self.keys.time_hack => self.start_time_hack(),
            _ => {}
        }
    }

    /// Processes a key event in any editing mode.
    fn handle_editing_key(&mut self, key: KeyEvent) {
        match key.code {
            c if c == self.keys.new_note => match self.mode {
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
                    self.cursor = 0;
                    self.mode = InputMode::Normal;
                }
                InputMode::TimeHack => {
                    self.finalize_time_hack();
                }
                InputMode::Normal
                | InputMode::Loading
                | InputMode::KeyBindings
                | InputMode::KeyCapture
                | InputMode::ConfirmReplace
                | InputMode::ThemeSelect
                | InputMode::BindWarning => {}
            },
            c if c == self.keys.cancel => self.cancel_entry(),
            KeyCode::Char('r') if matches!(self.mode, InputMode::TimeHack) => {
                self.time_hack = None;
                self.input.clear();
                self.cursor = 0;
                self.mode = InputMode::Normal;
            }
            KeyCode::Char(c)
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                if self.cursor >= self.input.len() {
                    self.input.push(c);
                } else {
                    self.input.insert(self.cursor, c);
                }
                self.cursor += 1;
            }
            KeyCode::Backspace => {
                if self.cursor > 0 && self.cursor <= self.input.len() {
                    self.cursor -= 1;
                    self.input.remove(self.cursor);
                }
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor < self.input.len() {
                    self.cursor += 1;
                }
            }
            _ => {}
        }
    }

    fn handle_loading_key(&mut self, key: KeyEvent) {
        match key.code {
            c if c == self.keys.up => {
                if self.load_selected > 0 {
                    self.load_selected -= 1;
                }
            }
            c if c == self.keys.down => {
                if self.load_selected + 1 < self.load_files.len() {
                    self.load_selected += 1;
                }
            }
            c if c == self.keys.new_note => {
                if let Some(path) = self.load_files.get(self.load_selected).cloned() {
                    self.load_from_file_in_place(path).ok();
                }
                self.mode = InputMode::Normal;
            }
            c if c == self.keys.cancel => {
                self.mode = InputMode::Normal;
            }
            _ => {}
        }
    }

    fn handle_keybindings_key(&mut self, key: KeyEvent) {
        match key.code {
            c if c == self.keys.up => {
                if self.keybind_selected > 0 {
                    self.keybind_selected -= 1;
                }
            }
            c if c == self.keys.down => {
                if self.keybind_selected + 1 < Action::ALL.len() {
                    self.keybind_selected += 1;
                }
            }
            KeyCode::Enter => self.start_capture_binding(),
            c if c == self.keys.cancel => self.mode = InputMode::Normal,
            _ => {}
        }
    }

    fn handle_capture_key(&mut self, key: KeyEvent) {
        if let Some(action) = self.capture_action {
            match key.code {
                c if c == self.keys.cancel => {
                    self.capture_action = None;
                    self.mode = InputMode::KeyBindings;
                }
                code if code == self.keys.bindings && action != Action::Bindings => {
                    self.mode = InputMode::BindWarning;
                }
                code => {
                    self.capture_action = None;
                    if let Some(conflict) = self.keys.action_for_key(code) {
                        if conflict != action {
                            self.pending_key = Some(code);
                            self.pending_action = Some(action);
                            self.pending_conflict = Some(conflict);
                            self.mode = InputMode::ConfirmReplace;
                            return;
                        }
                    }
                    self.keys.set(action, code);
                    self.keys.save();
                    self.mode = InputMode::KeyBindings;
                }
            }
        }
    }

    fn handle_bind_warning_key(&mut self, key: KeyEvent) {
        if key.code == self.keys.cancel {
            self.capture_action = None;
            self.mode = InputMode::KeyBindings;
        } else {
            self.mode = InputMode::KeyCapture;
        }
    }

    fn handle_confirm_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                if let (Some(new_key), Some(new_action), Some(conflict)) = (
                    self.pending_key.take(),
                    self.pending_action.take(),
                    self.pending_conflict.take(),
                ) {
                    self.keys.set(conflict, KeyCode::Null);
                    self.keys.set(new_action, new_key);
                    self.keys.save();
                }
                self.mode = InputMode::KeyBindings;
            }
            c if c == self.keys.cancel => {
                self.pending_key = None;
                self.pending_action = None;
                self.pending_conflict = None;
                self.mode = InputMode::KeyBindings;
            }
            _ => {}
        }
    }

    fn handle_theme_key(&mut self, key: KeyEvent) {
        match key.code {
            c if c == self.keys.up => {
                if self.theme_selected > 0 {
                    self.theme_selected -= 1;
                }
            }
            c if c == self.keys.down => {
                if self.theme_selected + 1 < ThemeName::ALL.len() {
                    self.theme_selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(t) = ThemeName::ALL.get(self.theme_selected).copied() {
                    self.theme = t;
                    t.save();
                }
                self.mode = InputMode::Normal;
            }
            c if c == self.keys.cancel => self.mode = InputMode::Normal,
            _ => {}
        }
    }

    /// Handles a terminal event.
    ///
    /// # Errors
    /// Propagates any I/O errors from the terminal event system.
    pub fn handle_event(&mut self, event: &Event) -> Result<(), AppError> {
        if let Event::Key(key) = *event {
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }
            match self.mode {
                InputMode::Normal => self.handle_normal_key(key),
                InputMode::Loading => self.handle_loading_key(key),
                InputMode::KeyBindings => self.handle_keybindings_key(key),
                InputMode::KeyCapture => self.handle_capture_key(key),
                InputMode::ConfirmReplace => self.handle_confirm_key(key),
                InputMode::BindWarning => self.handle_bind_warning_key(key),
                InputMode::ThemeSelect => self.handle_theme_key(key),
                InputMode::EditingNote
                | InputMode::EditingSection
                | InputMode::EditingExistingNote
                | InputMode::EditingExistingSection
                | InputMode::Saving
                | InputMode::TimeHack => self.handle_editing_key(key),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;
    use std::time::Instant;

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

    #[test]
    fn selection_navigation() {
        let mut app = App::new();
        app.start_note();
        app.finalize_note();
        app.start_section();
        app.input.push('a');
        app.finalize_section();

        assert_eq!(app.selected(), Some(1));
        app.select_previous();
        assert_eq!(app.selected(), Some(0));
        app.select_previous();
        assert_eq!(app.selected(), Some(0));
        app.select_next();
        assert_eq!(app.selected(), Some(1));
        app.select_next();
        assert_eq!(app.selected(), Some(1));
    }

    #[test]
    fn edit_existing_note() {
        let mut app = App::new();
        app.start_note();
        app.input.push_str("first");
        app.finalize_note();

        app.start_note();
        app.input.push_str("second");
        app.finalize_note();

        app.selected = Some(0);
        app.edit_selected();
        assert!(matches!(app.mode(), InputMode::EditingExistingNote));
        app.input.clear();
        app.input.push_str("updated");
        app.finalize_note();
        assert_eq!(app.notes[0].text, "updated");
    }

    #[test]
    fn edit_existing_section() {
        let mut app = App::new();
        app.start_section();
        app.input.push_str("sec");
        app.finalize_section();
        app.selected = Some(0);
        app.edit_selected();
        assert!(matches!(app.mode(), InputMode::EditingExistingSection));
        app.input.clear();
        app.input.push_str("new");
        app.finalize_section();
        if let Entry::Section(sec) = &app.entries[0] {
            assert_eq!(sec.title, "new");
        } else {
            panic!("Expected section");
        }
    }

    #[test]
    fn load_in_place() {
        let mut app = App::new();
        app.start_note();
        app.input.push_str("data");
        app.finalize_note();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.csv");
        app.save_to_file(&path).unwrap();

        let mut other = App::new();
        other.load_from_file_in_place(&path).unwrap();
        assert_eq!(other.entries.len(), 1);
        if let Entry::Note(n) = &other.entries[0] {
            assert_eq!(n.text, "data");
        } else {
            panic!("Expected note");
        }
    }

    #[test]
    fn handle_event_note_flow() {
        let mut app = App::new();
        let enter = Event::Key(KeyEvent::from(KeyCode::Enter));
        app.handle_event(&enter).unwrap();
        assert!(matches!(app.mode(), InputMode::EditingNote));
        let a = Event::Key(KeyEvent::from(KeyCode::Char('a')));
        let b = Event::Key(KeyEvent::from(KeyCode::Char('b')));
        app.handle_event(&a).unwrap();
        app.handle_event(&b).unwrap();
        app.handle_event(&enter).unwrap();
        assert_eq!(app.notes.len(), 1);
        assert_eq!(app.notes[0].text, "ab");
    }

    #[test]
    fn handle_left_right_navigation() {
        let mut app = App::new();
        let enter = Event::Key(KeyEvent::from(KeyCode::Enter));
        app.handle_event(&enter).unwrap();
        let a = Event::Key(KeyEvent::from(KeyCode::Char('a')));
        let b = Event::Key(KeyEvent::from(KeyCode::Char('b')));
        let c = Event::Key(KeyEvent::from(KeyCode::Char('c')));
        app.handle_event(&a).unwrap();
        app.handle_event(&b).unwrap();
        app.handle_event(&c).unwrap();
        let left = Event::Key(KeyEvent::from(KeyCode::Left));
        app.handle_event(&left).unwrap();
        app.handle_event(&left).unwrap();
        let x = Event::Key(KeyEvent::from(KeyCode::Char('X')));
        app.handle_event(&x).unwrap();
        let right = Event::Key(KeyEvent::from(KeyCode::Right));
        app.handle_event(&right).unwrap();
        app.handle_event(&enter).unwrap();
        assert_eq!(app.notes[0].text, "aXbc");
    }

    #[test]
    fn change_keybinding() {
        let mut keys = KeyBindings::default();
        keys.set(Action::Save, KeyCode::Char('z'));
        keys.save();
        let loaded = KeyBindings::load_or_default();
        assert_eq!(loaded.get(Action::Save), KeyCode::Char('z'));
        let _ = std::fs::remove_file(KeyBindings::config_path_for_test());
    }

    #[test]
    fn rebind_conflict() {
        let mut app = App::new();
        app.start_keybindings();
        app.capture_action = Some(Action::Load);
        // Press key already bound to Save
        let key = KeyEvent::from(KeyCode::Char('w'));
        app.handle_capture_key(key);
        assert!(matches!(app.mode(), InputMode::ConfirmReplace));
        assert_eq!(app.pending_action, Some(Action::Load));
        assert_eq!(app.pending_conflict, Some(Action::Save));
        // Confirm replacement
        app.handle_confirm_key(KeyEvent::from(KeyCode::Enter));
        assert_eq!(app.keys.get(Action::Load), KeyCode::Char('w'));
        assert_eq!(app.keys.get(Action::Save), KeyCode::Null);
    }

    #[test]
    fn rebind_bindings_key_warning() {
        let mut app = App::new();
        app.start_keybindings();
        app.capture_action = Some(Action::Save);
        let key = KeyEvent::from(app.keys.bindings);
        app.handle_capture_key(key);
        assert!(matches!(app.mode(), InputMode::BindWarning));
        assert_eq!(app.capture_action, Some(Action::Save));
        app.handle_bind_warning_key(KeyEvent::from(KeyCode::Char('x')));
        assert!(matches!(app.mode(), InputMode::KeyCapture));
        assert_eq!(app.capture_action, Some(Action::Save));
    }

    #[test]
    fn time_hack_sets_clock() {
        let mut app = App::new();
        app.time_hack = Some((NaiveTime::from_hms_opt(1, 2, 3).unwrap(), Instant::now()));
        app.start_note();
        let base_dt = Local
            .from_local_datetime(
                &Local::now()
                    .date_naive()
                    .and_time(NaiveTime::from_hms_opt(1, 2, 3).unwrap()),
            )
            .unwrap();
        let diff = app.note_time.unwrap() - base_dt;
        assert!(diff.num_seconds() >= 0 && diff.num_seconds() < 1);
    }

    #[test]
    fn finalize_time_hack_adds_entry() {
        let mut app = App::new();
        app.input.push_str("01:02:03");
        app.finalize_time_hack();
        assert!(app.time_hack.is_some());
        assert_eq!(app.entries.len(), 1);
        if let Entry::Note(n) = &app.entries[0] {
            assert!(n.text.contains("Time Hack"));
        } else {
            panic!("Expected note");
        }
    }
}
