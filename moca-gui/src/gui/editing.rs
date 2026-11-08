use iced::Task;

use super::message::Message;

impl super::app::App {
    pub(crate) fn finish_editing(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        let text = std::mem::take(&mut tab.edit_text);
        if let Some((from, to)) = tab.pending_transition.take() {
            tab.add_transition(from, to, &text);
        } else if let Some(state_id) = tab.editing_state.take() {
            tab.rename_state(state_id, text);
        }
        Task::none()
    }

    pub(crate) fn label_inputs_mut(&mut self) -> Option<&mut Vec<String>> {
        self.get_active_tab_mut().label_edit.as_mut().map(|edit| &mut edit.inputs)
    }

    pub(crate) fn save_transition_labels(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        if let Some(edit) = tab.label_edit.take() {
            tab.set_transition_labels(edit.pair, &edit.inputs);
        }
        Task::none()
    }
}
