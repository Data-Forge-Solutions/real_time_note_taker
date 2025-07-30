use directories::ProjectDirs;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemeName {
    Default,
    Matrix,
    CottonCandy,
    Rainbow,
    BlackWhite,
}

impl ThemeName {
    pub const ALL: [ThemeName; 5] = [
        ThemeName::Default,
        ThemeName::Matrix,
        ThemeName::CottonCandy,
        ThemeName::Rainbow,
        ThemeName::BlackWhite,
    ];

    fn config_path() -> PathBuf {
        if let Some(dirs) = ProjectDirs::from("com", "DFS", "rtnt") {
            dirs.config_dir().join("theme.json")
        } else {
            PathBuf::from("theme.json")
        }
    }

    #[cfg(test)]
    pub fn config_path_for_test() -> PathBuf {
        Self::config_path()
    }

    #[must_use]
    pub fn load_or_default() -> Self {
        let path = Self::config_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(t) = serde_json::from_str::<ThemeName>(&data) {
                return t;
            }
        }
        Self::Default
    }

    pub fn save(self) {
        if let Some(parent) = Self::config_path().parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(&self) {
            let _ = fs::write(Self::config_path(), data);
        }
    }

    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Matrix => "Matrix",
            Self::CottonCandy => "Cotton Candy",
            Self::Rainbow => "Rainbow",
            Self::BlackWhite => "Black & White",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub notes_border: Color,
    pub notes_highlight_bg: Color,
    pub notes_highlight_fg: Color,
    pub note_fg: Color,
    pub timestamp_fg: Color,
    pub input_fg: Color,
    pub help_key: Color,
    pub help_desc: Color,
    pub overlay_border: Color,
    pub overlay_highlight_bg: Color,
    pub overlay_highlight_fg: Color,
    pub editing_fg: Color,
    pub overlay_text: Color,
}

impl ThemeName {
    #[must_use]
    pub fn theme(self) -> Theme {
        match self {
            Self::Default => Theme {
                notes_border: Color::LightBlue,
                notes_highlight_bg: Color::LightBlue,
                notes_highlight_fg: Color::Black,
                note_fg: Color::White,
                timestamp_fg: Color::Yellow,
                input_fg: Color::White,
                help_key: Color::LightMagenta,
                help_desc: Color::LightCyan,
                overlay_border: Color::LightMagenta,
                overlay_highlight_bg: Color::LightMagenta,
                overlay_highlight_fg: Color::Black,
                editing_fg: Color::Yellow,
                overlay_text: Color::White,
            },
            Self::Matrix => Theme {
                notes_border: Color::LightGreen,
                notes_highlight_bg: Color::Green,
                notes_highlight_fg: Color::Black,
                note_fg: Color::LightGreen,
                timestamp_fg: Color::Green,
                input_fg: Color::Green,
                help_key: Color::LightGreen,
                help_desc: Color::Green,
                overlay_border: Color::LightGreen,
                overlay_highlight_bg: Color::Green,
                overlay_highlight_fg: Color::Black,
                editing_fg: Color::LightGreen,
                overlay_text: Color::LightGreen,
            },
            Self::CottonCandy => Theme {
                notes_border: Color::LightMagenta,
                notes_highlight_bg: Color::LightMagenta,
                notes_highlight_fg: Color::Black,
                note_fg: Color::LightMagenta,
                timestamp_fg: Color::Cyan,
                input_fg: Color::Cyan,
                help_key: Color::LightMagenta,
                help_desc: Color::LightCyan,
                overlay_border: Color::Cyan,
                overlay_highlight_bg: Color::LightMagenta,
                overlay_highlight_fg: Color::Cyan,
                editing_fg: Color::LightMagenta,
                overlay_text: Color::Cyan,
            },
            Self::Rainbow => Theme {
                notes_border: Color::LightRed,
                notes_highlight_bg: Color::LightBlue,
                notes_highlight_fg: Color::Black,
                note_fg: Color::LightMagenta,
                timestamp_fg: Color::LightYellow,
                input_fg: Color::LightGreen,
                help_key: Color::LightGreen,
                help_desc: Color::LightYellow,
                overlay_border: Color::LightGreen,
                overlay_highlight_bg: Color::LightBlue,
                overlay_highlight_fg: Color::LightMagenta,
                editing_fg: Color::LightCyan,
                overlay_text: Color::LightGreen,
            },
            Self::BlackWhite => Theme {
                notes_border: Color::White,
                notes_highlight_bg: Color::White,
                notes_highlight_fg: Color::Black,
                note_fg: Color::White,
                timestamp_fg: Color::White,
                input_fg: Color::White,
                help_key: Color::White,
                help_desc: Color::White,
                overlay_border: Color::White,
                overlay_highlight_bg: Color::White,
                overlay_highlight_fg: Color::Black,
                editing_fg: Color::White,
                overlay_text: Color::White,
            },
        }
    }
}

impl Default for ThemeName {
    fn default() -> Self {
        Self::Default
    }
}
