#![allow(clippy::too_many_lines)]
use directories::ProjectDirs;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemeName {
    Default,
    Matrix,
    CyanCrush,
    Embercore,
    ToxicOrchid,
    Coldfire,
}

impl ThemeName {
    pub const ALL: [ThemeName; 6] = [
        ThemeName::Default,
        ThemeName::Matrix,
        ThemeName::CyanCrush,
        ThemeName::Embercore,
        ThemeName::ToxicOrchid,
        ThemeName::Coldfire,
    ];

    fn config_path() -> PathBuf {
        if let Some(dirs) = ProjectDirs::from("com", "DFS", "rtnt") {
            dirs.config_dir().join("theme.json")
        } else {
            PathBuf::from("theme.json")
        }
    }

    #[cfg(test)]
    #[must_use]
    /// Returns the configuration path used during testing.
    pub fn config_path_for_test() -> PathBuf {
        Self::config_path()
    }

    #[must_use]
    /// Loads the theme configuration from disk or returns [`ThemeName::Default`].
    pub fn load_or_default() -> Self {
        let path = Self::config_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(t) = serde_json::from_str::<ThemeName>(&data) {
                return t;
            }
        }
        Self::Default
    }

    /// Persists the theme configuration to disk.
    pub fn save(self) {
        if let Some(parent) = Self::config_path().parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(&self) {
            let _ = fs::write(Self::config_path(), data);
        }
    }

    #[must_use]
    /// Human readable display name for the theme.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Matrix => "Matrix",
            Self::CyanCrush => "Cyan Crush",
            Self::Embercore => "Embercore",
            Self::ToxicOrchid => "Toxic Orchid",
            Self::Coldfire => "Coldfire",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub notes_border: Color,
    pub notes_title: Color,
    pub notes_highlight_bg: Color,
    pub notes_highlight_fg: Color,
    pub note_fg: Color,
    pub section_fg: Color,
    pub timestamp_fg: Color,
    pub input_fg: Color,
    pub input_title: Color,
    pub help_key: Color,
    pub help_desc: Color,
    pub overlay_border: Color,
    pub overlay_title: Color,
    pub overlay_highlight_bg: Color,
    pub overlay_highlight_fg: Color,
    pub overlay_bg: Color,
    pub editing_fg: Color,
    pub editing_title: Color,
    pub overlay_text: Color,
}

impl ThemeName {
    #[must_use]
    /// Returns the [`Theme`] struct for this theme name.
    pub fn theme(self) -> Theme {
        match self {
            Self::Default => Theme {
                notes_border: Color::White,
                notes_title: Color::Black,
                notes_highlight_bg: Color::White,
                notes_highlight_fg: Color::Black,
                note_fg: Color::White,
                section_fg: Color::White,
                timestamp_fg: Color::White,
                input_fg: Color::White,
                input_title: Color::Black,
                help_key: Color::White,
                help_desc: Color::White,
                overlay_border: Color::White,
                overlay_title: Color::Black,
                overlay_highlight_bg: Color::White,
                overlay_highlight_fg: Color::Black,
                overlay_bg: Color::Black,
                editing_fg: Color::White,
                editing_title: Color::Black,
                overlay_text: Color::White,
            },
            Self::Matrix => Theme {
                notes_border: Color::LightGreen,
                notes_title: Color::LightGreen,
                notes_highlight_bg: Color::Green,
                notes_highlight_fg: Color::Black,
                note_fg: Color::LightGreen,
                section_fg: Color::LightGreen,
                timestamp_fg: Color::Green,
                input_fg: Color::Green,
                input_title: Color::LightGreen,
                help_key: Color::LightGreen,
                help_desc: Color::Green,
                overlay_border: Color::LightGreen,
                overlay_title: Color::LightGreen,
                overlay_highlight_bg: Color::Green,
                overlay_highlight_fg: Color::Black,
                overlay_bg: Color::Black,
                editing_fg: Color::LightGreen,
                editing_title: Color::Green,
                overlay_text: Color::LightGreen,
            },
            Self::CyanCrush => Theme {
                notes_border: Color::Cyan,
                notes_title: Color::LightMagenta,
                notes_highlight_bg: Color::LightMagenta,
                notes_highlight_fg: Color::Black,
                note_fg: Color::LightMagenta,
                section_fg: Color::LightMagenta,
                timestamp_fg: Color::Cyan,
                input_fg: Color::Cyan,
                input_title: Color::LightMagenta,
                help_key: Color::LightMagenta,
                help_desc: Color::LightCyan,
                overlay_border: Color::Cyan,
                overlay_title: Color::LightMagenta,
                overlay_highlight_bg: Color::LightMagenta,
                overlay_highlight_fg: Color::Cyan,
                overlay_bg: Color::Black,
                editing_fg: Color::LightMagenta,
                editing_title: Color::Cyan,
                overlay_text: Color::Cyan,
            },
            Self::Embercore => Theme {
                notes_border: Color::Red,
                notes_title: Color::Yellow,
                notes_highlight_bg: Color::LightRed,
                notes_highlight_fg: Color::Black,
                note_fg: Color::LightYellow,
                section_fg: Color::Yellow,
                timestamp_fg: Color::Red,
                input_fg: Color::Red,
                input_title: Color::Yellow,
                help_key: Color::Red,
                help_desc: Color::Yellow,
                overlay_border: Color::LightRed,
                overlay_title: Color::Yellow,
                overlay_highlight_bg: Color::Red,
                overlay_highlight_fg: Color::Yellow,
                overlay_bg: Color::Black,
                editing_fg: Color::Yellow,
                editing_title: Color::Red,
                overlay_text: Color::LightRed,
            },
            Self::ToxicOrchid => Theme {
                notes_border: Color::LightMagenta,
                notes_title: Color::LightGreen,
                notes_highlight_bg: Color::LightGreen,
                notes_highlight_fg: Color::Black,
                note_fg: Color::LightMagenta,
                section_fg: Color::LightMagenta,
                timestamp_fg: Color::LightGreen,
                input_fg: Color::LightMagenta,
                input_title: Color::LightGreen,
                help_key: Color::LightGreen,
                help_desc: Color::LightMagenta,
                overlay_border: Color::LightGreen,
                overlay_title: Color::LightMagenta,
                overlay_highlight_bg: Color::LightGreen,
                overlay_highlight_fg: Color::LightMagenta,
                overlay_bg: Color::Black,
                editing_fg: Color::LightMagenta,
                editing_title: Color::LightGreen,
                overlay_text: Color::LightGreen,
            },
            Self::Coldfire => Theme {
                notes_border: Color::LightBlue,
                notes_title: Color::LightRed,
                notes_highlight_bg: Color::LightBlue,
                notes_highlight_fg: Color::Black,
                note_fg: Color::LightRed,
                section_fg: Color::LightBlue,
                timestamp_fg: Color::LightBlue,
                input_fg: Color::LightBlue,
                input_title: Color::LightRed,
                help_key: Color::LightRed,
                help_desc: Color::LightBlue,
                overlay_border: Color::LightRed,
                overlay_title: Color::LightBlue,
                overlay_highlight_bg: Color::LightRed,
                overlay_highlight_fg: Color::LightBlue,
                overlay_bg: Color::Black,
                editing_fg: Color::LightBlue,
                editing_title: Color::LightRed,
                overlay_text: Color::LightRed,
            },
        }
    }
}

impl Default for ThemeName {
    fn default() -> Self {
        Self::Default
    }
}
