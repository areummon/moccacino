use iced::Task;
use std::collections::HashMap;

use moca_data::entity_file::{parse_entity_file, LLM_PROMPT};

use super::message::Message;
use super::tab::Tab;
use crate::theme::Tone;
use crate::platform;

const NO_FILE_CHOSEN: &str =
    "no file chosen (the system file dialog may be unavailable on this system — type or paste the file path instead)";
const ENTER_PATH: &str = "Enter a file path first.";

#[derive(Debug, Default)]
pub(crate) struct PathDialog {
    pub(crate) path: String,
    pub(crate) error: Option<String>,
}

#[derive(Debug)]
pub(crate) struct SaveDialog {
    pub(crate) file: PathDialog,
    contents: String,
    tab_index: usize,
    fingerprint: u64,
}

pub(crate) const LOAD_INPUT: &str = "load-path";
pub(crate) const SAVE_INPUT: &str = "save-path";

impl super::app::App {
    pub(crate) fn open_load_dialog(&mut self) -> Task<Message> {
        if platform::IS_WEB {
            return self.load_browse_clicked();
        }
        self.load_dialog = Some(PathDialog::default());
        iced::widget::text_input::focus(LOAD_INPUT)
    }

    pub(crate) fn load_browse_clicked(&mut self) -> Task<Message> {
        let dialog = rfd::AsyncFileDialog::new()
            .add_filter("Computational entities", &["ce", "cm"])
            .add_filter("All files", &["*"])
            .pick_file();
        Task::future(async move {
            let picked = match dialog.await {
                Some(handle) => {
                    let file_name = handle.file_name();
                    let bytes = handle.read().await;
                    Some(match String::from_utf8(bytes) {
                        Ok(text) => Ok((file_name, text)),
                        Err(_) => Err(format!("{file_name} is not valid UTF-8 text")),
                    })
                }
                None => None,
            };
            Message::LoadBrowseResult { picked }
        })
    }

    pub(crate) fn load_browse_result(&mut self, picked: Option<Result<(String, String), String>>) -> Task<Message> {
        match (picked, self.load_dialog.as_mut()) {
            (Some(Ok((file_name, text))), _) => {
                self.load_dialog = None;
                return self.entities_loaded(file_name, text);
            }
            (Some(Err(message)), Some(dialog)) => dialog.error = Some(message),
            (Some(Err(message)), None) => self.error_message = Some(message),
            (None, Some(dialog)) => dialog.error = Some(NO_FILE_CHOSEN.to_string()),
            (None, None) => {}
        }
        Task::none()
    }

    pub(crate) fn load_path_submitted(&mut self) -> Task<Message> {
        let Some(dialog) = self.load_dialog.as_mut() else {
            return Task::none();
        };
        let path = dialog.path.trim().to_string();
        if path.is_empty() {
            dialog.error = Some(ENTER_PATH.to_string());
            return Task::none();
        }
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let file_name = std::path::Path::new(&path)
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or(path);
                self.load_dialog = None;
                self.entities_loaded(file_name, text)
            }
            Err(error) => {
                dialog.error = Some(format!("Cannot read {path}: {error}"));
                Task::none()
            }
        }
    }

    pub(crate) fn open_save_dialog(&mut self) -> Task<Message> {
        self.open_menu = None;
        let tab_index = self.active_tab;
        let tab = self.get_active_tab_mut();
        let contents = match tab.entity_text() {
            Ok(contents) => contents,
            Err(problem) => {
                self.error_message = Some(problem);
                return Task::none();
            }
        };
        self.save_dialog = Some(SaveDialog {
            file: PathDialog { path: format!("{}.ce", sanitize_file_name(&tab.name)), error: None },
            contents,
            tab_index,
            fingerprint: tab.content_fingerprint(),
        });
        if platform::IS_WEB {
            return self.download_pending_save();
        }
        iced::widget::text_input::focus(SAVE_INPUT)
    }

    fn download_pending_save(&mut self) -> Task<Message> {
        let Some(dialog) = self.save_dialog.take() else {
            return Task::none();
        };
        let file_name = dialog.file.path.clone();
        match platform::download(&file_name, &dialog.contents) {
            Ok(()) => {
                self.mark_saved(&dialog);
                self.toast(Tone::Success, format!("Downloaded {file_name}"), None);
            }
            Err(error) => self.error_message = Some(format!("Cannot download {file_name}: {error}")),
        }
        Task::none()
    }

    pub(crate) fn save_browse_clicked(&mut self) -> Task<Message> {
        let Some(dialog) = self.save_dialog.as_ref() else {
            return Task::none();
        };
        let contents = dialog.contents.clone();
        let default_name = std::path::Path::new(dialog.file.path.trim())
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "entity.ce".to_string());
        let picker = rfd::AsyncFileDialog::new()
            .add_filter("Computational entities", &["ce"])
            .set_file_name(&default_name)
            .save_file();
        Task::future(async move {
            let result = match picker.await {
                Some(handle) => match handle.write(contents.as_bytes()).await {
                    Ok(()) => Ok(handle.file_name()),
                    Err(error) => Err(format!("Cannot write {}: {}", handle.file_name(), error)),
                },
                None => Err(NO_FILE_CHOSEN.to_string()),
            };
            Message::SaveBrowseResult { result }
        })
    }

    pub(crate) fn save_browse_result(&mut self, result: Result<String, String>) -> Task<Message> {
        match result {
            Ok(file_name) => self.finish_save(format!("Saved {file_name}")),
            Err(message) => {
                if let Some(dialog) = self.save_dialog.as_mut() {
                    dialog.file.error = Some(message);
                }
            }
        }
        Task::none()
    }

    pub(crate) fn save_path_submitted(&mut self) -> Task<Message> {
        let Some(dialog) = self.save_dialog.as_mut() else {
            return Task::none();
        };
        let mut path = dialog.file.path.trim().to_string();
        if path.is_empty() {
            dialog.file.error = Some(ENTER_PATH.to_string());
            return Task::none();
        }
        if std::path::Path::new(&path).extension().is_none() {
            path.push_str(".ce");
        }
        match std::fs::write(&path, &dialog.contents) {
            Ok(()) => self.finish_save(format!("Saved {path}")),
            Err(error) => dialog.file.error = Some(format!("Cannot write {path}: {error}")),
        }
        Task::none()
    }

    fn finish_save(&mut self, confirmation: String) {
        if let Some(dialog) = self.save_dialog.take() {
            self.mark_saved(&dialog);
        }
        self.toast(Tone::Success, confirmation, None);
    }

    fn mark_saved(&mut self, dialog: &SaveDialog) {
        if let Some(tab) = self.tabs.get_mut(dialog.tab_index) {
            tab.saved_fingerprint = Some(dialog.fingerprint);
            tab.insight.unsaved = tab.has_unsaved_changes();
        }
    }

    fn entities_loaded(&mut self, file_name: String, text: String) -> Task<Message> {
        let (entities, errors) = parse_entity_file(&text);
        if entities.is_empty() {
            self.error_message = Some(format!("No entities loaded from {}:{}", file_name, summarize_errors(&errors)));
            return Task::none();
        }

        let entity_count = entities.len();
        let mut used: HashMap<String, usize> = HashMap::new();
        let mut tasks = Vec::new();
        for named in entities {
            let count = used.entry(named.name.clone()).or_insert(0);
            *count += 1;
            let tab_name = if *count == 1 { named.name.clone() } else { format!("{} {}", named.name, count) };
            let mut tab = Tab::from_entity(tab_name, named.entity);
            tab.mark_saved();
            tasks.push(self.open_in_new_tab(tab));
        }

        let plural = if entity_count == 1 { "y" } else { "ies" };
        if errors.is_empty() {
            self.toast(Tone::Success, format!("Loaded {entity_count} entit{plural} from {file_name}"), None);
        } else {
            self.toast(
                Tone::Warning,
                format!("Loaded {} entit{} from {}, skipped {}", entity_count, plural, file_name, errors.len()),
                Some(summarize_errors(&errors).trim_start().to_string()),
            );
        }
        Task::batch(tasks)
    }

    pub(crate) fn copy_llm_prompt(&mut self) -> Task<Message> {
        self.open_menu = None;
        self.toast(
            Tone::Success,
            "LLM prompt copied",
            Some("Paste it into a vision-capable model together with a picture of a state diagram.".to_string()),
        );
        iced::clipboard::write(LLM_PROMPT.to_string())
    }
}

fn summarize_errors(errors: &[moca_data::entity_file::EntityError]) -> String {
    let mut summary = String::new();
    for error in errors.iter().take(5) {
        if error.entity_name.is_empty() {
            summary.push_str(&format!("\n  • line {}: {}", error.line, error.message));
        } else {
            summary.push_str(&format!(
                "\n  • '{}' (line {}): {}",
                error.entity_name, error.line, error.message
            ));
        }
    }
    if errors.len() > 5 {
        summary.push_str(&format!("\n  • … and {} more", errors.len() - 5));
    }
    summary
}

fn sanitize_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if matches!(c, '/' | '\\' | '\n' | '\r') { '-' } else { c })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        "entity".to_string()
    } else {
        trimmed.to_string()
    }
}
