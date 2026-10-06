use iced::Task;

use crate::state_machine;

use super::message::Message;

impl super::app::App {
    pub(crate) fn clear_active_tab(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        tab.state_machine.reset_id_counter();
        tab.state_machine.set_scroll(iced::Vector::new(0.0, 0.0));
        tab.state_machine.set_zoom(1.0);
        tab.transitions.clear();
        tab.states.clear();
        tab.state_id_to_index.clear();
        tab.initial_state = None;
        tab.final_states.clear();
        tab.check_input_dialog_open = false;
        tab.check_input_text.clear();
        tab.regex_dialog_open = false;
        tab.run = None;
        tab.playing = false;
        tab.set_active_tool(state_machine::EditorTool::Arrow);
        Task::none()
    }

    pub(crate) fn edit_text_changed(&mut self, text: String) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        tab.edit_text = text.clone();
        tab.pending_transition_label = text;
        Task::none()
    }

    pub(crate) fn finish_editing(&mut self) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        if active_tab.pending_transition_dialog_open {
            if let Some((from_state_id, to_state_id, _, _)) = active_tab.pending_transition.take() {
                let label = if active_tab.pending_transition_label.trim().is_empty() {
                    "ε".to_string()
                } else {
                    active_tab.pending_transition_label.clone()
                };
                active_tab.transitions.entry((from_state_id, to_state_id)).or_default().insert(label);
                active_tab.pending_transition_dialog_open = false;
                active_tab.pending_transition_label.clear();
                active_tab.state_machine.request_redraw();
                return Task::none();
            }
        }
        if let Some(state_id) = active_tab.editing_state {
            if let Some(index) = active_tab.state_id_to_index.get(&state_id) {
                if let Some(state) = active_tab.states.get_mut(*index) {
                    let edit_text = active_tab.edit_text.clone();
                    state.label = edit_text;
                }
            }
        }
        active_tab.editing_state = None;
        active_tab.editing_transition = None;
        active_tab.edit_text.clear();
        active_tab.state_machine.request_redraw();
        Task::none()
    }

    pub(crate) fn cancel_editing(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        tab.pending_transition = None;
        tab.pending_transition_label.clear();
        tab.pending_transition_dialog_open = false;
        tab.editing_state = None;
        tab.editing_transition = None;
        tab.edit_text.clear();
        Task::none()
    }

    pub(crate) fn open_edit_transition(&mut self, (from, to): (usize, usize)) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        if let Some(labels) = active_tab.transitions.get(&(from, to)) {
            active_tab.editing_transition_pair = Some((from, to));
            active_tab.editing_transition_labels = labels.iter().cloned().collect();
            active_tab.editing_transition_label_inputs = active_tab.editing_transition_labels.clone();
            active_tab.editing_transition_dialog_open = true;
        }
        Task::none()
    }

    pub(crate) fn edit_transition_label_changed(&mut self, idx: usize, text: String) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        if idx < active_tab.editing_transition_label_inputs.len() {
            active_tab.editing_transition_label_inputs[idx] = text;
        }
        Task::none()
    }

    pub(crate) fn save_edit_transition_labels(&mut self) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        if let Some((from, to)) = active_tab.editing_transition_pair {
            let new_labels: Vec<String> = active_tab.editing_transition_label_inputs.iter()
                .map(|s| if s.trim().is_empty() { "ε".to_string() } else { s.clone() })
                .filter(|s| s != "ε" || active_tab.editing_transition_label_inputs.iter().any(|input| !input.trim().is_empty()))
                .collect();

            if new_labels.is_empty() {
                active_tab.transitions.remove(&(from, to));
            } else {
                active_tab.transitions.insert((from, to), new_labels.into_iter().collect());
            }
        }
        active_tab.editing_transition_pair = None;
        active_tab.editing_transition_labels.clear();
        active_tab.editing_transition_label_inputs.clear();
        active_tab.editing_transition_dialog_open = false;
        active_tab.state_machine.request_redraw();
        Task::none()
    }

    pub(crate) fn delete_edit_transition_label(&mut self, idx: usize) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        if idx < active_tab.editing_transition_label_inputs.len() {
            active_tab.editing_transition_label_inputs.remove(idx);
        }
        Task::none()
    }

    pub(crate) fn add_edit_transition_label(&mut self) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        active_tab.editing_transition_label_inputs.push(String::new());
        Task::none()
    }

    pub(crate) fn cancel_edit_transition_labels(&mut self) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        active_tab.editing_transition_pair = None;
        active_tab.editing_transition_labels.clear();
        active_tab.editing_transition_label_inputs.clear();
        active_tab.editing_transition_dialog_open = false;
        Task::none()
    }
}
