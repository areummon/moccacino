use iced::Task;

use moca_data::finite_automata::FiniteAutomata;
use moca_data::state_machine::StateMachine;

use super::dialogs::{CHECK_INPUT, REGEX_INPUT};
use super::message::Message;
use super::automaton::Automaton;
use super::tab::Tab;
use crate::theme::Tone;
use crate::tikz_export;

impl super::app::App {
    pub(crate) fn open_check_input(&mut self) -> Task<Message> {
        self.open_menu = None;
        self.get_active_tab_mut().check_input = Some(String::new());
        iced::widget::text_input::focus(CHECK_INPUT)
    }

    pub(crate) fn nfa_to_dfa(&mut self) -> Task<Message> {
        self.finite_operation("DFA", "convert", |finite| {
            if finite.is_deterministic() {
                Err("Cannot convert: The automaton is already deterministic.".to_string())
            } else {
                Ok(finite.to_dfa())
            }
        })
    }

    pub(crate) fn minimize(&mut self) -> Task<Message> {
        self.finite_operation("Minimized", "minimize", |finite| {
            if finite.is_deterministic() {
                Ok(finite.minimize())
            } else {
                Err("Cannot minimize: The automaton must be deterministic.".to_string())
            }
        })
    }

    fn finite_operation(
        &mut self,
        tab_name: &str,
        verb: &str,
        operation: impl Fn(&FiniteAutomata) -> Result<FiniteAutomata, String>,
    ) -> Task<Message> {
        self.open_menu = None;
        match self.get_active_tab_mut().finite(verb).and_then(operation) {
            Ok(result) => self.open_in_new_tab(Tab::from_automaton(tab_name.to_string(), Automaton::Finite(result))),
            Err(problem) => {
                self.error_message = Some(problem);
                Task::none()
            }
        }
    }

    pub(crate) fn submit_check_input(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        let input = tab.check_input.clone().unwrap_or_default();
        match tab.accepts(&input) {
            Ok(accepted) => {
                tab.check_input = None;
                let shown = if input.is_empty() { "ε (empty word)".to_string() } else { format!("'{input}'") };
                let (tone, verdict) = if accepted { (Tone::Success, "accepted") } else { (Tone::Danger, "rejected") };
                self.toast(tone, format!("{shown} is {verdict}"), None);
            }
            Err(problem) => self.error_message = Some(problem),
        }
        Task::none()
    }

    pub(crate) fn open_regex_dialog(&mut self) -> Task<Message> {
        self.open_menu = None;
        self.get_active_tab_mut().regex_input = Some(String::new());
        iced::widget::text_input::focus(REGEX_INPUT)
    }

    pub(crate) fn submit_regex(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        let pattern = tab.regex_input.as_deref().unwrap_or_default().trim().to_string();
        let machine = match moca_data::regex::compile_str(&pattern) {
            Ok(machine) => machine,
            Err(error) => {
                self.error_message = Some(format!("Invalid regular expression: {error}"));
                return Task::none();
            }
        };
        tab.regex_input = None;
        let tab_name = if pattern.chars().count() > 18 {
            format!("Regex: {}...", pattern.chars().take(15).collect::<String>())
        } else if pattern.is_empty() {
            "Regex: ε".to_string()
        } else {
            format!("Regex: {pattern}")
        };
        self.open_in_new_tab(Tab::from_automaton(tab_name, Automaton::Finite(machine)))
    }

    pub(crate) fn open_latex_export(&mut self) -> Task<Message> {
        self.open_menu = None;
        let tab = self.get_active_tab();
        let code = match tab.grammar() {
            Some(editor) => editor
                .parse()
                .map(|grammar| tikz_export::export_grammar_to_latex(&grammar))
                .map_err(|problem| format!("Cannot export: {problem}")),
            None if tab.states.is_empty() && tab.transitions.is_empty() => {
                Err("Nothing to export: the canvas is empty.".to_string())
            }
            None => Ok(tikz_export::export_to_tikz(&tab.states, &tab.transitions, tab.initial_state, &tab.final_states)),
        };
        match code {
            Ok(code) => self.latex_export = Some(code),
            Err(problem) => self.error_message = Some(problem),
        }
        Task::none()
    }

    pub(crate) fn open_regex_export(&mut self) -> Task<Message> {
        self.open_menu = None;
        match self.get_active_tab_mut().finite("export a regex") {
            Ok(finite) => self.regex_export = Some(finite.to_regex().to_string()),
            Err(problem) => self.error_message = Some(problem),
        }
        Task::none()
    }

    pub(crate) fn copy_export(&mut self, code: Option<String>, confirmation: &str) -> Task<Message> {
        let Some(code) = code else {
            return Task::none();
        };
        self.toast(Tone::Success, confirmation, None);
        iced::clipboard::write(code)
    }
}
