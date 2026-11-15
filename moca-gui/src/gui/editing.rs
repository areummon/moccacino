use iced::Task;

use super::message::Message;
use super::shortcuts::{hint, Action};
use crate::theme::Tone;

impl super::app::App {
    pub(crate) fn finish_editing(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        let text = std::mem::take(&mut tab.edit_text);
        if let Some(pair) = tab.pending_transition.take() {
            tab.add_transition(pair, &text);
        } else if let Some(state_id) = tab.editing_state.take() {
            tab.drawing.rename_state(state_id, text);
        }
        Task::none()
    }

    pub(crate) fn clear_canvas(&mut self) -> Task<Message> {
        self.get_active_tab_mut().clear();
        self.toast(Tone::Neutral, format!("Canvas cleared. {} brings it back.", hint(Action::Undo)), None);
        Task::none()
    }

    pub(crate) fn undo(&mut self) -> Task<Message> {
        self.get_active_tab_mut().undo();
        Task::none()
    }

    pub(crate) fn redo(&mut self) -> Task<Message> {
        self.get_active_tab_mut().redo();
        Task::none()
    }

    pub(crate) fn select_all(&mut self) -> Task<Message> {
        self.get_active_tab_mut().select_all();
        Task::none()
    }

    pub(crate) fn delete_selection(&mut self) -> Task<Message> {
        self.context_menu = None;
        self.get_active_tab_mut().delete_selection();
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
