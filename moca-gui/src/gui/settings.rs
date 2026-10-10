use crate::gui::theme::ThemeMode;
use crate::platform;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Settings {
    pub(crate) theme: ThemeMode,
}

impl Settings {
    pub(crate) fn load() -> Self {
        let mut settings = Settings::default();
        let Some(contents) = platform::load_settings_text() else {
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
        let theme = match self.theme {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        };
        platform::store_settings_text(&format!("theme={theme}\n"));
    }
}
