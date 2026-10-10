pub(crate) use imp::*;

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::path::PathBuf;

    use iced::Task;

    pub(crate) const IS_WEB: bool = false;

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

    pub(crate) fn download(_file_name: &str, _contents: &str) -> Result<(), String> {
        unreachable!("native saves go through the save dialog; download is only reached when IS_WEB")
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use iced::Task;
    use wasm_bindgen::{closure::Closure, JsCast, JsValue};
    use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Storage, Url};

    pub(crate) const IS_WEB: bool = true;

    const SETTINGS_KEY: &str = "moccacino.settings";

    // Browsers may still be reading the blob after the click returns.
    const REVOKE_DELAY_MS: i32 = 60_000;

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

    pub(crate) fn download(file_name: &str, contents: &str) -> Result<(), String> {
        let describe = |error: JsValue| format!("{error:?}");
        let window = web_sys::window().ok_or("no browser window")?;
        let document = window.document().ok_or("no document")?;
        let body = document.body().ok_or("no document body")?;

        let options = BlobPropertyBag::new();
        options.set_type("text/plain;charset=utf-8");
        let parts = js_sys::Array::of1(&JsValue::from_str(contents));
        let blob = Blob::new_with_str_sequence_and_options(&parts, &options).map_err(describe)?;
        let url = Url::create_object_url_with_blob(&blob).map_err(describe)?;

        let anchor: HtmlAnchorElement = document
            .create_element("a")
            .map_err(describe)?
            .dyn_into()
            .map_err(|_| "cannot create a link element".to_string())?;
        anchor.set_href(&url);
        anchor.set_download(file_name);
        body.append_child(&anchor).map_err(describe)?;
        anchor.click();
        anchor.remove();

        let revoke = Closure::once_into_js(move || {
            let _ = Url::revoke_object_url(&url);
        });
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                revoke.unchecked_ref(),
                REVOKE_DELAY_MS,
            )
            .map_err(describe)?;
        Ok(())
    }
}
