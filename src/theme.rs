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

impl Theme {
    #[must_use]
    pub const fn from_colors(primary: Color, secondary: Color, tertiary: Color) -> Self {
        Self {
            notes_border: primary,
            notes_title: secondary,
            notes_highlight_bg: secondary,
            notes_highlight_fg: tertiary,
            note_fg: secondary,
            section_fg: secondary,
            timestamp_fg: primary,
            input_fg: primary,
            input_title: secondary,
            help_key: secondary,
            help_desc: primary,
            overlay_border: primary,
            overlay_title: secondary,
            overlay_highlight_bg: secondary,
            overlay_highlight_fg: primary,
            overlay_bg: tertiary,
            editing_fg: secondary,
            editing_title: primary,
            overlay_text: primary,
        }
    }
}

impl ThemeName {
    #[must_use]
    /// Returns the [`Theme`] struct for this theme name.
    pub fn theme(self) -> Theme {
        match self {
            Self::Default => Theme::from_colors(Color::White, Color::Black, Color::Black),
            Self::Matrix => Theme::from_colors(Color::LightGreen, Color::Green, Color::Black),
            Self::CyanCrush => Theme::from_colors(Color::Cyan, Color::LightMagenta, Color::Black),
            Self::Embercore => Theme::from_colors(Color::Red, Color::Yellow, Color::Black),
            Self::ToxicOrchid => {
                Theme::from_colors(Color::LightMagenta, Color::LightGreen, Color::Black)
            }
            Self::Coldfire => Theme::from_colors(Color::LightBlue, Color::LightRed, Color::Black),
        }
    }
}

impl Default for ThemeName {
    fn default() -> Self {
        Self::Default
    }
}
