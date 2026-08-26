use iced::Task;

use crate::state_machine;
use crate::tikz_export;

use moca_data::state_machine::StateMachine;

use super::message::Message;
use super::tab::Tab;

impl super::app::App {
    pub(crate) fn toggle_operations_menu(&mut self) -> Task<Message> {
        self.get_active_tab_mut().operations_menu_open = !self.get_active_tab_mut().operations_menu_open;
        Task::none()
    }

    pub(crate) fn open_check_input(&mut self) -> Task<Message> {
        self.get_active_tab_mut().operations_menu_open = false;
        self.get_active_tab_mut().check_input_dialog_open = true;
        self.get_active_tab_mut().check_input_text = String::new();
        Task::none()
    }

    pub(crate) fn dfa_to_nfa(&mut self) -> Task<Message> {
        self.get_active_tab_mut().operations_menu_open = false;

        self.get_active_tab_mut().sync_gui_to_finite_automata();

        if self.get_active_tab().machine.is_deterministic() {
            self.error_message = Some("Cannot convert: The automaton is already deterministic.".to_string());
            return Task::none();
        }

        let dfa = self.get_active_tab().machine.to_dfa();

        let mut new_tab = Tab::new_with_name("DFA".to_string());
        new_tab.machine = dfa;
        self.tabs.push(Box::new(new_tab));
        self.active_tab = self.tabs.len() - 1;

        self.get_active_tab_mut().load_finite_automata_to_gui();
        Task::none()
    }

    pub(crate) fn minimize(&mut self) -> Task<Message> {
        self.get_active_tab_mut().operations_menu_open = false;

        self.get_active_tab_mut().sync_gui_to_finite_automata();

        if !self.get_active_tab().machine.is_deterministic() {
            self.error_message = Some("Cannot minimize: The automaton must be deterministic.".to_string());
            return Task::none();
        }

        let minimized = self.get_active_tab().machine.minimize();

        let mut new_tab = Tab::new_with_name("Minimized".to_string());
        new_tab.machine = minimized;
        self.tabs.push(Box::new(new_tab));
        self.active_tab = self.tabs.len() - 1;

        self.get_active_tab_mut().load_finite_automata_to_gui();
        Task::none()
    }

    pub(crate) fn check_input_text_changed(&mut self, text: String) -> Task<Message> {
        self.get_active_tab_mut().check_input_text = text;
        Task::none()
    }

    pub(crate) fn submit_check_input(&mut self) -> Task<Message> {
        let mut input = self.get_active_tab().check_input_text.clone();
        // Allow blank inputs to be processed (don't convert to epsilon)
        self.get_active_tab_mut().sync_gui_to_finite_automata();
        let result = self.get_active_tab().machine.check_input(&mut input);
        self.get_active_tab_mut().check_input_result = Some(result);
        self.get_active_tab_mut().check_result_popup_open = true;
        self.get_active_tab_mut().check_input_dialog_open = false;
        Task::none()
    }

    pub(crate) fn cancel_check_input(&mut self) -> Task<Message> {
        self.get_active_tab_mut().check_input_dialog_open = false;
        Task::none()
    }

    pub(crate) fn close_check_result_popup(&mut self) -> Task<Message> {
        self.get_active_tab_mut().check_result_popup_open = false;
        Task::none()
    }

    pub(crate) fn open_latex_export(&mut self) -> Task<Message> {
        // Convert transitions HashMap to Vec<Transition> for export
        let mut export_transitions = Vec::new();
        for (&(from, to), labels) in &self.get_active_tab().transitions {
            for label in labels {
                export_transitions.push(state_machine::Transition {
                    from_state_id: from,
                    to_state_id: to,
                    from_point: iced::Point::ORIGIN, // dummy
                    to_point: iced::Point::ORIGIN,   // dummy
                    label: Box::leak(label.clone().into_boxed_str()),
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

    pub(crate) fn copy_latex_export(&mut self) -> Task<Message> {
        if let Some(code) = &self.latex_export_code {
            return iced::clipboard::write(code.clone()).map(|_msg: ()| Message::CopyLatexExport);
        }
        Task::none()
    }
}
