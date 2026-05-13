use crate::color::Rgb;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

const DEFAULT_PALETTE_LIMIT: usize = 18;
const FAVORITES_LIMIT: usize = 48;
pub const PALETTE_LIMIT_OPTIONS: [usize; 4] = [12, 18, 24, 36];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ToastDuration {
    Short,
    Normal,
    Long,
}

impl Default for ToastDuration {
    fn default() -> Self {
        Self::Normal
    }
}

impl ToastDuration {
    pub const OPTIONS: [Self; 3] = [Self::Short, Self::Normal, Self::Long];

    pub fn label(self) -> &'static str {
        match self {
            Self::Short => "Short",
            Self::Normal => "Normal",
            Self::Long => "Long",
        }
    }

    pub fn millis(self) -> u64 {
        match self {
            Self::Short => 1_200,
            Self::Normal => 2_000,
            Self::Long => 3_500,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct AppSettings {
    palette_limit: usize,
    toast_duration: ToastDuration,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            palette_limit: DEFAULT_PALETTE_LIMIT,
            toast_duration: ToastDuration::default(),
        }
    }
}

impl AppSettings {
    pub fn palette_limit(&self) -> usize {
        if PALETTE_LIMIT_OPTIONS.contains(&self.palette_limit) {
            self.palette_limit
        } else {
            DEFAULT_PALETTE_LIMIT
        }
    }

    pub fn toast_duration(&self) -> ToastDuration {
        self.toast_duration
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ColorCollections {
    palette: Vec<Rgb>,
    favorites: Vec<Rgb>,
    settings: AppSettings,
}

impl ColorCollections {
    pub fn load() -> Self {
        let Some(path) = storage_path() else {
            return Self::default();
        };

        let Ok(contents) = fs::read_to_string(path) else {
            return Self::default();
        };

        let mut collections: Self = serde_json::from_str(&contents).unwrap_or_default();
        collections.normalize();
        collections
    }

    pub fn save(&self) {
        let Some(path) = storage_path() else {
            return;
        };

        if let Some(parent) = path.parent() {
            if fs::create_dir_all(parent).is_err() {
                return;
            }
        }

        if let Ok(contents) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, contents);
        }
    }

    pub fn palette(&self) -> &[Rgb] {
        &self.palette
    }

    pub fn favorites(&self) -> &[Rgb] {
        &self.favorites
    }

    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    pub fn add_palette_color(&mut self, color: Rgb) {
        move_to_front(&mut self.palette, color);
        self.palette.truncate(self.settings.palette_limit());
    }

    pub fn set_palette_limit(&mut self, limit: usize) {
        if !PALETTE_LIMIT_OPTIONS.contains(&limit) {
            return;
        }

        self.settings.palette_limit = limit;
        self.palette.truncate(limit);
    }

    pub fn set_toast_duration(&mut self, duration: ToastDuration) {
        self.settings.toast_duration = duration;
    }

    pub fn clear_palette(&mut self) {
        self.palette.clear();
    }

    pub fn clear_favorites(&mut self) {
        self.favorites.clear();
    }

    pub fn toggle_favorite(&mut self, color: Rgb) -> bool {
        if let Some(index) = self.favorites.iter().position(|saved| *saved == color) {
            self.favorites.remove(index);
            return false;
        }

        self.favorites.insert(0, color);
        self.favorites.truncate(FAVORITES_LIMIT);
        true
    }

    pub fn remove_favorite(&mut self, color: Rgb) {
        if let Some(index) = self.favorites.iter().position(|saved| *saved == color) {
            self.favorites.remove(index);
        }
    }

    pub fn is_favorite(&self, color: Rgb) -> bool {
        self.favorites.contains(&color)
    }

    fn normalize(&mut self) {
        let palette_limit = self.settings.palette_limit();
        self.settings.palette_limit = palette_limit;
        self.palette.truncate(palette_limit);
    }
}

fn move_to_front(colors: &mut Vec<Rgb>, color: Rgb) {
    if let Some(index) = colors.iter().position(|saved| *saved == color) {
        colors.remove(index);
    }

    colors.insert(0, color);
}

fn storage_path() -> Option<PathBuf> {
    let base = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))?;

    Some(base.join("ubuntu-color-picker").join("colors.json"))
}
