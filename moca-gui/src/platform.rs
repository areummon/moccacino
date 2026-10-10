pub(crate) use imp::*;

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::path::PathBuf;

    use iced::Task;

    fn settings_path() -> Option<PathBuf> {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
        Some(base.join("moccacino").join("settings"))
    }

    pub(crate) fn load_settings_text() -> Option<String> {
        std::fs::read_to_string(settings_path()?).ok()
    }

    pub(crate) fn store_settings_text(text: &str) {
        let Some(path) = settings_path() else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, text);
    }

    pub(crate) fn copy_text<M>(text: String) -> Task<M> {
        iced::clipboard::write(text)
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use iced::Task;
    use web_sys::Storage;

    const SETTINGS_KEY: &str = "moccacino.settings";

    fn storage() -> Option<Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    pub(crate) fn load_settings_text() -> Option<String> {
        storage()?.get_item(SETTINGS_KEY).ok()?
    }

    pub(crate) fn store_settings_text(text: &str) {
        if let Some(storage) = storage() {
            let _ = storage.set_item(SETTINGS_KEY, text);
        }
    }

    // iced's clipboard is a no-op on wasm, so write through the async Clipboard API.
    pub(crate) fn copy_text<M>(text: String) -> Task<M> {
        if let Some(window) = web_sys::window() {
            let _ = window.navigator().clipboard().write_text(&text);
        }
        Task::none()
    }
}
