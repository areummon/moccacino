
use std::path::PathBuf;

use crate::gui::theme::ThemeMode;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Settings {
    pub(crate) theme: ThemeMode,
}

fn settings_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("moccacino").join("settings"))
}

impl Settings {
    pub(crate) fn load() -> Self {
        let mut settings = Settings::default();
        let Some(contents) = settings_path().and_then(|path| std::fs::read_to_string(path).ok()) else {
            return settings;
        };
        for line in contents.lines() {
            let Some((key, value)) = line.split_once('=') else { continue };
            if key.trim() == "theme" {
                settings.theme = match value.trim() {
                    "dark" => ThemeMode::Dark,
                    _ => ThemeMode::Light,
                };
            }
        }
        settings
    }

    pub(crate) fn save(&self) {
        let Some(path) = settings_path() else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let theme = match self.theme {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        };
        let _ = std::fs::write(path, format!("theme={theme}\n"));
    }
}
