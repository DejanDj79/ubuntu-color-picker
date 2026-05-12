use crate::color::Rgb;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

const RECENT_LIMIT: usize = 24;
const FAVORITES_LIMIT: usize = 48;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ColorCollections {
    recent: Vec<Rgb>,
    favorites: Vec<Rgb>,
}

impl ColorCollections {
    pub fn load() -> Self {
        let Some(path) = storage_path() else {
            return Self::default();
        };

        let Ok(contents) = fs::read_to_string(path) else {
            return Self::default();
        };

        serde_json::from_str(&contents).unwrap_or_default()
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

    pub fn recent(&self) -> &[Rgb] {
        &self.recent
    }

    pub fn favorites(&self) -> &[Rgb] {
        &self.favorites
    }

    pub fn add_recent(&mut self, color: Rgb) {
        move_to_front(&mut self.recent, color);
        self.recent.truncate(RECENT_LIMIT);
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

    pub fn is_favorite(&self, color: Rgb) -> bool {
        self.favorites.contains(&color)
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
