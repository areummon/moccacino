use iced::Task;

use crate::state_machine;

use moca_data::finite_automata::FiniteAutomata;

use super::message::Message;

impl super::app::App {
    pub(crate) fn clear_active_tab(&mut self) -> Task<Message> {
        self.get_active_tab_mut().state_machine.reset_id_counter();
        self.get_active_tab_mut().state_machine.request_redraw();
        self.get_active_tab_mut().transitions.clear();
        self.get_active_tab_mut().states.clear();
        self.get_active_tab_mut().state_id_to_index.clear();
        self.get_active_tab_mut().initial_state = None;
        self.get_active_tab_mut().final_states.clear();
        self.get_active_tab_mut().machine = FiniteAutomata::default();
        self.get_active_tab_mut().check_input_dialog_open = false;
        self.get_active_tab_mut().check_input_text.clear();
        self.get_active_tab_mut().check_result_popup_open = false;
        self.get_active_tab_mut().check_input_result = None;
        self.get_active_tab_mut().deletion_mode = false;
        self.get_active_tab_mut().state_machine.set_deletion_mode(false);
        Task::none()
    }

    pub(crate) fn edit_text_changed(&mut self, text: String) -> Task<Message> {
        let text_clone = text.clone();
        self.get_active_tab_mut().edit_text = text_clone;
        self.get_active_tab_mut().pending_transition_label = text;
        Task::none()
    }

    pub(crate) fn finish_editing(&mut self) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        // Handle pending transition dialog
        if active_tab.pending_transition_dialog_open {
            if let Some((from_state_id, to_state_id, from_point, to_point)) = active_tab.pending_transition.take() {
                let label = if active_tab.pending_transition_label.trim().is_empty() {
                    Box::leak("ε".to_string().into_boxed_str())
                } else {
                    Box::leak(active_tab.pending_transition_label.clone().into_boxed_str())
                };
                let transition = state_machine::Transition {
                    from_state_id,
                    to_state_id,
                    from_point,
                    to_point,
                    label,
                };
                let key = (transition.from_state_id, transition.to_state_id);
                let entry = active_tab.transitions.entry(key).or_insert_with(indexmap::IndexSet::new);
                if !entry.contains(&transition.label.to_string()) {
                    entry.insert(transition.label.to_string());
                }
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
                    state.label = Box::leak(edit_text.into_boxed_str());
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
        // Cancel pending transition dialog
        self.get_active_tab_mut().pending_transition = None;
        self.get_active_tab_mut().pending_transition_label.clear();
        self.get_active_tab_mut().pending_transition_dialog_open = false;
        self.get_active_tab_mut().editing_state = None;
        self.get_active_tab_mut().editing_transition = None;
        self.get_active_tab_mut().edit_text.clear();
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
            // Convert empty strings to "ε" and filter out completely empty labels
            let new_labels: Vec<String> = active_tab.editing_transition_label_inputs.iter()
                .map(|s| if s.trim().is_empty() { "ε".to_string() } else { s.clone() })
                .filter(|s| s != "ε" || active_tab.editing_transition_label_inputs.iter().any(|input| !input.trim().is_empty())) // Keep ε only if there are other non-empty labels
                .collect();

            if new_labels.is_empty() {
                // If no labels remain, remove the entire transition
                active_tab.transitions.remove(&(from, to));
            } else {
                // Otherwise, update with the new labels
                let mut set = indexmap::IndexSet::new();
                for label in new_labels {
                    set.insert(label);
                }
                active_tab.transitions.insert((from, to), set);
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
