
use iced::Task;
use std::collections::HashMap;

use moca_data::entity_file::{
    parse_entity_file, write_finite_entity, write_grammar_entity, write_pushdown_entity,
    write_turing_entity, Entity,
};

use super::dialogs::{LOAD_INPUT, SAVE_INPUT};
use super::message::Message;
use super::tab::TabMachine;
use crate::gui::theme::Tone;
use crate::platform;

pub(crate) const LLM_PROMPT: &str = r#"You are given an image of one or more computational models (a state diagram of an automaton, a Turing machine, a pushdown automaton, a regular expression, or a context-free grammar).

Transcribe what you see into a `.ce` file with the exact syntax below, then output ONLY the file contents (no explanations, no code fences).

SYNTAX
- One entity per block, each block starts with `entity: <kind>` where kind is one of:
  dfa | nfa | fa | automata | finite | tm | turing | pda | pushdown | regex | grammar | cfg
- Optional `name: <label>` line right after `entity:`.
- Lines starting with `#` are comments. Blank lines are ignored.
- State lists and finals are comma-separated. Multiple `transitions:` / `productions:` lines accumulate.

TRANSITION SYNTAX (different per family - use the right one!)
- dfa/nfa/fa/automata/finite:  (from, symbol) -> to
    symbols may be single characters; use ε for epsilon moves.
- pda/pushdown:  (from, input, pop, push...) -> to
    4 parts: input read, stack symbol popped, stack symbols pushed (comma-separated = pushed in order, last on top). Use ε for no-ops.
    Example: (q0, a, Z, A, Z) -> q1   (read a, pop Z, push A then Z)
- tm/turing:  (from, read, write, dir[, read, write, dir...]) -> to
    one (read, write, dir) group per tape; dir is L, R or S.
    Example single tape: (q0, 0, 1, R) -> q1
    Example two tapes:   (q0, 0, 1, R, _, _, S) -> q1
- regex: a line `regex: <pattern>` with operators | * + ? and ε.
- grammar: `productions: <lhs> -> body | body` lines; ε is an empty body.

COMMON KEYS
- `states: a, b, c` (optional for regex/grammar)
- `initial: <state>` (machines only)
- `final: <state>, <state>` also accepts `finals:` / `halt:` / `final/halt:` (machines only)
- `blank: <char>` (turing only, default _)
- `stack: <symbol>` (pushdown only, default Z)

EXAMPLE FILE
# Balanced parentheses recognizer and a regex, together in one file
entity: pda
name: balanced-parens
states: q0, q1, q2
transitions: (q0, ε, Z, PZ) -> q0, (q0, (, P, PP) -> q0, (q0, ), P, ε) -> q1, (q1, ε, Z, Z) -> q2
initial: q0
final: q2

entity: regex
name: abb
regex: (a|b)*abb

entity: grammar
name: anbn
productions: S -> a S b | ε

TASK
Look at the attached image carefully (states, arrows, labels, initial arrow, double circles for accepting states, stack/tape annotations) and produce the matching `.ce` file. Use short state names exactly as drawn or numbered q0, q1, ... if unlabeled. Output only the .ce file contents."#;

impl super::app::App {
    pub(crate) fn open_load_dialog(&mut self) -> Task<Message> {
        if platform::IS_WEB {
            return self.load_browse_clicked();
        }
        self.load_dialog_open = true;
        self.load_path_text.clear();
        self.load_dialog_error = None;
        iced::widget::text_input::focus(LOAD_INPUT)
    }

    pub(crate) fn load_path_changed(&mut self, text: String) -> Task<Message> {
        self.load_path_text = text;
        Task::none()
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

    pub(crate) fn load_browse_result(
        &mut self,
        picked: Option<Result<(String, String), String>>,
    ) -> Task<Message> {
        match picked {
            Some(Ok((file_name, text))) => {
                self.load_dialog_open = false;
                self.load_dialog_error = None;
                self.entities_loaded(file_name, Ok(text))
            }
            Some(Err(message)) if self.load_dialog_open => {
                self.load_dialog_error = Some(message);
                Task::none()
            }
            Some(Err(message)) => {
                self.error_message = Some(message);
                Task::none()
            }
            None if self.load_dialog_open => {
                self.load_dialog_error = Some(
                    "no file chosen (the system file dialog may be unavailable on this \
                     system — type or paste the file path instead)"
                        .to_string(),
                );
                Task::none()
            }
            None => Task::none(),
        }
    }

    pub(crate) fn load_path_submitted(&mut self) -> Task<Message> {
        let path = self.load_path_text.trim().to_string();
        if path.is_empty() {
            self.load_dialog_error = Some("Enter a file path first.".to_string());
            return Task::none();
        }
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let file_name = std::path::Path::new(&path)
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or(path);
                self.load_dialog_open = false;
                self.load_dialog_error = None;
                self.entities_loaded(file_name, Ok(text))
            }
            Err(error) => {
                self.load_dialog_error =
                    Some(format!("Cannot read {path}: {error}"));
                Task::none()
            }
        }
    }

    pub(crate) fn cancel_load_dialog(&mut self) -> Task<Message> {
        self.load_dialog_open = false;
        self.load_dialog_error = None;
        Task::none()
    }

    pub(crate) fn open_save_dialog(&mut self) -> Task<Message> {
        self.open_menu = None;

        if self.get_active_tab().machine.is_grammar() {
            if self.get_active_tab().grammar_text.trim().is_empty() {
                self.error_message = Some(
                    "The grammar is empty — add at least one production before saving."
                        .to_string(),
                );
                return Task::none();
            }
            let grammar = match self.parsed_grammar() {
                Ok(grammar) => grammar,
                Err(problem) => {
                    self.error_message = Some(problem);
                    return Task::none();
                }
            };
            if grammar.productions().is_empty() {
                self.error_message = Some(
                    "The grammar is empty — add at least one production before saving."
                        .to_string(),
                );
                return Task::none();
            }
            match write_grammar_entity(&self.get_active_tab().name, &grammar) {
                Ok(text) => self.stash_pending_save(text),
                Err(message) => self.error_message = Some(message),
            }
        } else {
            let tab = self.get_active_tab_mut();
            tab.sync_gui_to_machine();
            if tab.machine.states_ref().is_empty() {
                self.error_message = Some(
                    "The machine is empty — add at least one state before saving.".to_string(),
                );
                return Task::none();
            }
            if let Err(problem) = tab.machine.validate() {
                self.error_message = Some(problem);
                return Task::none();
            }
            let result = match &tab.machine {
                TabMachine::Finite(finite) => write_finite_entity(&tab.name, finite),
                TabMachine::Pushdown(pda) => write_pushdown_entity(&tab.name, pda),
                TabMachine::Turing(turing) => write_turing_entity(&tab.name, turing),
                TabMachine::Grammar(_) => unreachable!("handled above"),
            };
            match result {
                Ok(text) => self.stash_pending_save(text),
                Err(message) => self.error_message = Some(message),
            }
        }
        if self.save_dialog_open {
            if platform::IS_WEB {
                return self.download_pending_save();
            }
            return iced::widget::text_input::focus(SAVE_INPUT);
        }
        Task::none()
    }

    fn download_pending_save(&mut self) -> Task<Message> {
        let file_name = self.save_path_text.clone();
        let Some(contents) = self.pending_save.take() else {
            return Task::none();
        };
        match platform::download(&file_name, &contents) {
            Ok(()) => {
                self.mark_pending_save_done();
                self.toast(Tone::Success, format!("Downloaded {file_name}"), None);
            }
            Err(error) => {
                self.error_message = Some(format!("Cannot download {file_name}: {error}"));
            }
        }
        self.close_save_dialog();
        Task::none()
    }

    fn stash_pending_save(&mut self, contents: String) {
        self.pending_save = Some(contents);
        let fingerprint = self.get_active_tab().content_fingerprint();
        self.pending_save_fingerprint = Some((self.active_tab, fingerprint));
        self.save_dialog_error = None;
        self.save_path_text = format!("{}.ce", sanitize_file_name(&self.get_active_tab().name));
        self.save_dialog_open = true;
    }

    pub(crate) fn save_path_changed(&mut self, text: String) -> Task<Message> {
        self.save_path_text = text;
        Task::none()
    }

    pub(crate) fn save_browse_clicked(&mut self) -> Task<Message> {
        let Some(contents) = self.pending_save.clone() else {
            self.save_dialog_error =
                Some("Nothing to save — close the dialog and press Save .ce again.".to_string());
            return Task::none();
        };
        let default_name = std::path::Path::new(self.save_path_text.trim())
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "entity.ce".to_string());
        let dialog = rfd::AsyncFileDialog::new()
            .add_filter("Computational entities", &["ce"])
            .set_file_name(&default_name)
            .save_file();
        Task::future(async move {
            match dialog.await {
                Some(handle) => match handle.write(contents.as_bytes()).await {
                    Ok(()) => Message::SaveBrowseResult { result: Ok(handle.file_name()) },
                    Err(error) => Message::SaveBrowseResult {
                        result: Err(format!("Cannot write {}: {}", handle.file_name(), error)),
                    },
                },
                None => Message::SaveBrowseResult {
                    result: Err(
                        "no file chosen (the system file dialog may be unavailable on this \
                         system — type or paste the file path instead)"
                            .to_string(),
                    ),
                },
            }
        })
    }

    pub(crate) fn save_browse_result(&mut self, result: Result<String, String>) -> Task<Message> {
        match result {
            Ok(file_name) => {
                self.mark_pending_save_done();
                self.close_save_dialog();
                self.toast(Tone::Success, format!("Saved {file_name}"), None);
                Task::none()
            }
            Err(message) => {
                self.save_dialog_error = Some(message);
                Task::none()
            }
        }
    }

    pub(crate) fn save_path_submitted(&mut self) -> Task<Message> {
        let Some(contents) = self.pending_save.clone() else {
            self.save_dialog_error =
                Some("Nothing to save — close the dialog and press Save .ce again.".to_string());
            return Task::none();
        };
        let mut path = self.save_path_text.trim().to_string();
        if path.is_empty() {
            self.save_dialog_error = Some("Enter a file path first.".to_string());
            return Task::none();
        }
        if std::path::Path::new(&path).extension().is_none() {
            path.push_str(".ce");
        }
        match std::fs::write(&path, contents) {
            Ok(()) => {
                self.mark_pending_save_done();
                self.close_save_dialog();
                self.toast(Tone::Success, format!("Saved {path}"), None);
                Task::none()
            }
            Err(error) => {
                self.save_dialog_error = Some(format!("Cannot write {path}: {error}"));
                Task::none()
            }
        }
    }

    pub(crate) fn cancel_save_dialog(&mut self) -> Task<Message> {
        self.close_save_dialog();
        Task::none()
    }

    fn close_save_dialog(&mut self) {
        self.save_dialog_open = false;
        self.save_dialog_error = None;
        self.pending_save = None;
        self.pending_save_fingerprint = None;
    }

    fn mark_pending_save_done(&mut self) {
        if let Some((index, fingerprint)) = self.pending_save_fingerprint {
            if let Some(tab) = self.tabs.get_mut(index) {
                tab.saved_fingerprint = Some(fingerprint);
                tab.insight.unsaved = tab.has_unsaved_changes();
            }
        }
    }

    pub(crate) fn entities_loaded(
        &mut self,
        file_name: String,
        contents: Result<String, String>,
    ) -> Task<Message> {
        let text = match contents {
            Ok(text) => text,
            Err(error) => {
                self.error_message = Some(format!("Cannot read {file_name}: {error}"));
                return Task::none();
            }
        };

        let (entities, errors) = parse_entity_file(&text);
        if entities.is_empty() {
            self.error_message = Some(format!(
                "No entities loaded from {}:{}",
                file_name,
                summarize_errors(&errors)
            ));
            return Task::none();
        }

        let entity_count = entities.len();
        let mut used: HashMap<String, usize> = HashMap::new();
        let mut tasks = Vec::new();
        for named in entities {
            let count = used.entry(named.name.clone()).or_insert(0);
            *count += 1;
            let tab_name = if *count == 1 {
                named.name.clone()
            } else {
                format!("{} {}", named.name, count)
            };
            match named.entity {
                Entity::Grammar(grammar) => {
                    let output = format!("Loaded from .ce file. Start symbol: {}.", grammar.start_symbol());
                    self.open_grammar_in_new_tab(tab_name, grammar, output);
                }
                Entity::Finite(finite) => tasks.push(self.open_machine_in_new_tab(tab_name, TabMachine::Finite(finite))),
                Entity::Pushdown(pda) => tasks.push(self.open_machine_in_new_tab(tab_name, TabMachine::Pushdown(pda))),
                Entity::Turing(turing) => tasks.push(self.open_machine_in_new_tab(tab_name, TabMachine::Turing(turing))),
            }
            self.get_active_tab_mut().mark_saved();
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
