use iced::Task;

use crate::state_machine;
use crate::tikz_export;

use moca_data::state_machine::MachineKind;

use super::dialogs::{CHECK_INPUT, REGEX_INPUT};
use super::message::Message;
use super::tab::TabMachine;
use crate::gui::theme::Tone;

impl super::app::App {
    pub(crate) fn open_check_input(&mut self) -> Task<Message> {
        self.open_menu = None;
        let tab = self.get_active_tab_mut();
        tab.check_input_dialog_open = true;
        tab.check_input_text.clear();
        iced::widget::text_input::focus(CHECK_INPUT)
    }

    pub(crate) fn dfa_to_nfa(&mut self) -> Task<Message> {
        self.open_menu = None;

        if self.get_active_tab().machine.is_grammar() {
            return self.reject_operation_for_grammar("convert");
        }

        self.get_active_tab_mut().sync_gui_to_machine();

        match self.get_active_tab().machine.to_dfa() {
            Some(machine) => self.open_machine_in_new_tab("DFA".to_string(), machine),
            None => self.reject_non_finite("convert", "The automaton is already deterministic."),
        }
    }

    pub(crate) fn minimize(&mut self) -> Task<Message> {
        self.open_menu = None;

        if self.get_active_tab().machine.is_grammar() {
            return self.reject_operation_for_grammar("minimize");
        }

        self.get_active_tab_mut().sync_gui_to_machine();

        match self.get_active_tab().machine.minimized() {
            Some(machine) => self.open_machine_in_new_tab("Minimized".to_string(), machine),
            None => self.reject_non_finite("minimize", "The automaton must be deterministic."),
        }
    }

    fn reject_non_finite(&mut self, verb: &str, finite_reason: &str) -> Task<Message> {
        let holder = match self.get_active_tab().machine.machine_kind() {
            Some(MachineKind::Turing) => Some("a Turing machine"),
            Some(MachineKind::Pushdown) => Some("a pushdown automaton"),
            _ => None,
        };
        self.error_message = Some(match holder {
            Some(holder) => format!("Cannot {verb}: the tab holds {holder}, not a finite automaton."),
            None => format!("Cannot {verb}: {finite_reason}"),
        });
        Task::none()
    }

    pub(crate) fn check_input_text_changed(&mut self, text: String) -> Task<Message> {
        self.get_active_tab_mut().check_input_text = text;
        Task::none()
    }

    pub(crate) fn submit_check_input(&mut self) -> Task<Message> {
        let input = self.get_active_tab().check_input_text.clone();
        self.get_active_tab_mut().sync_gui_to_machine();
        if let Err(problem) = self.get_active_tab().machine.validate() {
            self.error_message = Some(problem);
            return Task::none();
        }
        let result = self.get_active_tab().machine.accepts(&input);
        self.get_active_tab_mut().check_input_dialog_open = false;
        let shown = if input.is_empty() { "ε (empty word)".to_string() } else { format!("'{input}'") };
        let (tone, verdict) = if result { (Tone::Success, "accepted") } else { (Tone::Danger, "rejected") };
        self.toast(tone, format!("{shown} is {verdict}"), None);
        Task::none()
    }

    pub(crate) fn cancel_check_input(&mut self) -> Task<Message> {
        self.get_active_tab_mut().check_input_dialog_open = false;
        Task::none()
    }

    pub(crate) fn open_regex_dialog(&mut self) -> Task<Message> {
        self.open_menu = None;
        let tab = self.get_active_tab_mut();
        tab.regex_dialog_open = true;
        tab.regex_text.clear();
        iced::widget::text_input::focus(REGEX_INPUT)
    }

    pub(crate) fn regex_text_changed(&mut self, text: String) -> Task<Message> {
        self.get_active_tab_mut().regex_text = text;
        Task::none()
    }

    pub(crate) fn cancel_regex(&mut self) -> Task<Message> {
        self.get_active_tab_mut().regex_dialog_open = false;
        Task::none()
    }

    pub(crate) fn submit_regex(&mut self) -> Task<Message> {
        let pattern = self.get_active_tab().regex_text.trim().to_string();

        let machine = match moca_data::regex::compile_str(&pattern) {
            Ok(machine) => machine,
            Err(error) => {
                self.error_message = Some(format!("Invalid regular expression: {error}"));
                return Task::none();
            }
        };

        self.get_active_tab_mut().regex_dialog_open = false;

        let tab_name = if pattern.chars().count() > 18 {
            format!("Regex: {}...", pattern.chars().take(15).collect::<String>())
        } else if pattern.is_empty() {
            "Regex: ε".to_string()
        } else {
            format!("Regex: {pattern}")
        };
        self.open_machine_in_new_tab(tab_name, TabMachine::Finite(machine))
    }

    pub(crate) fn open_latex_export(&mut self) -> Task<Message> {
        self.open_menu = None;
        if matches!(self.get_active_tab().machine, TabMachine::Grammar(_)) {
            let source = self.get_active_tab().grammar_text.clone();
            return match moca_data::grammar::parse_grammar(&source) {
                Ok(grammar) => {
                    if grammar.productions().is_empty() {
                        self.error_message =
                            Some("Cannot export: the grammar has no productions.".to_string());
                        return Task::none();
                    }
                    self.latex_export_code = Some(tikz_export::export_grammar_to_latex(&grammar));
                    self.latex_export_dialog_open = true;
                    Task::none()
                }
                Err(error) => {
                    self.error_message = Some(format!("Cannot export the grammar: {error}"));
                    Task::none()
                }
            };
        }

        if self.get_active_tab().states.is_empty() && self.get_active_tab().transitions.is_empty() {
            self.error_message = Some("Nothing to export: the canvas is empty.".to_string());
            return Task::none();
        }

        let mut export_transitions = Vec::new();
        for (&(from, to), labels) in &self.get_active_tab().transitions {
            for label in labels {
                export_transitions.push(state_machine::Transition {
                    from_state_id: from,
                    to_state_id: to,
                    from_point: iced::Point::ORIGIN,
                    to_point: iced::Point::ORIGIN,
                    label: label.clone(),
                });
            }
        }
        let code = tikz_export::export_to_tikz(
            &self.get_active_tab().states,
            &export_transitions,
            self.get_active_tab().initial_state,
            &self.get_active_tab().final_states,
        );
        self.latex_export_code = Some(code);
        self.latex_export_dialog_open = true;
        Task::none()
    }

    pub(crate) fn close_latex_export(&mut self) -> Task<Message> {
        self.latex_export_dialog_open = false;
        self.latex_export_code = None;
        Task::none()
    }

    pub(crate) fn open_regex_export(&mut self) -> Task<Message> {
        self.open_menu = None;

        if !matches!(
            self.get_active_tab().machine,
            TabMachine::Finite(_)
        ) {
            self.error_message = Some(
                "Cannot export a regex: the tab must hold a finite automaton.".to_string(),
            );
            return Task::none();
        }

        self.get_active_tab_mut().sync_gui_to_machine();

        let expression = match &self.get_active_tab().machine {
            TabMachine::Finite(finite) => finite.to_regex().to_string(),
            _ => return Task::none(),
        };
        self.regex_export_code = Some(expression);
        self.regex_export_dialog_open = true;
        Task::none()
    }

    pub(crate) fn close_regex_export(&mut self) -> Task<Message> {
        self.regex_export_dialog_open = false;
        self.regex_export_code = None;
        Task::none()
    }

    pub(crate) fn copy_regex_export(&mut self) -> Task<Message> {
        if let Some(code) = self.regex_export_code.clone() {
            self.toast(Tone::Success, "Regular expression copied", None);
            return crate::platform::copy_text(code);
        }
        Task::none()
    }

    pub(crate) fn copy_latex_export(&mut self) -> Task<Message> {
        if let Some(code) = self.latex_export_code.clone() {
            self.toast(Tone::Success, "LaTeX copied to the clipboard", None);
            return crate::platform::copy_text(code);
        }
        Task::none()
    }
}
