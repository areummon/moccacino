use iced::{Element, Task};

use super::message::{Message, Modal};

impl super::app::App {
    fn is_open(&self, modal: Modal) -> bool {
        let tab = self.get_active_tab();
        match modal {
            Modal::RenameState => tab.editing_state.is_some(),
            Modal::CheckInput => tab.check_input.is_some(),
            Modal::BuildRegex => tab.regex_input.is_some(),
            Modal::Load => self.load_dialog.is_some(),
            Modal::Save => self.save_dialog.is_some(),
            Modal::LatexExport => self.latex_export.is_some(),
            Modal::RegexExport => self.regex_export.is_some(),
            Modal::NewTransition => tab.pending_transition.is_some(),
            Modal::TransitionLabels => tab.label_edit.is_some(),
            Modal::Shortcuts => self.shortcuts_open,
            Modal::Close => self.pending_close.is_some(),
            Modal::Error => self.error_message.is_some(),
        }
    }

    pub(crate) fn open_modals(&self) -> impl Iterator<Item = Modal> + '_ {
        Modal::STACK.into_iter().filter(|modal| self.is_open(*modal))
    }

    pub(crate) fn any_modal_open(&self) -> bool {
        self.startup_picker_open || self.open_modals().next().is_some()
    }

    pub(crate) fn dismiss_modal(&mut self) -> Task<Message> {
        if let Some(top) = self.open_modals().last() {
            self.close_modal(top);
        }
        Task::none()
    }

    pub(crate) fn close_modal(&mut self, modal: Modal) {
        let tab = self.get_active_tab_mut();
        match modal {
            Modal::RenameState | Modal::NewTransition => {
                tab.editing_state = None;
                tab.pending_transition = None;
                tab.edit_text.clear();
            }
            Modal::CheckInput => tab.check_input = None,
            Modal::BuildRegex => tab.regex_input = None,
            Modal::TransitionLabels => tab.label_edit = None,
            Modal::Load => self.load_dialog = None,
            Modal::Save => self.save_dialog = None,
            Modal::LatexExport => self.latex_export = None,
            Modal::RegexExport => self.regex_export = None,
            Modal::Shortcuts => self.shortcuts_open = false,
            Modal::Close => self.pending_close = None,
            Modal::Error => self.error_message = None,
        }
    }

    pub(crate) fn modal_view(&self, modal: Modal) -> Element<'_, Message> {
        match modal {
            Modal::RenameState => self.create_rename_dialog(),
            Modal::CheckInput => self.create_check_input_dialog(),
            Modal::BuildRegex => self.create_regex_dialog(),
            Modal::Load => self.create_load_dialog(),
            Modal::Save => self.create_save_dialog(),
            Modal::LatexExport => self.create_latex_export_dialog(),
            Modal::RegexExport => self.create_regex_export_dialog(),
            Modal::NewTransition => self.create_new_transition_dialog(),
            Modal::TransitionLabels => self.create_transition_labels_dialog(),
            Modal::Shortcuts => self.create_shortcuts_dialog(),
            Modal::Close => self.create_close_dialog(),
            Modal::Error => self.create_error_popup(),
        }
    }
}
