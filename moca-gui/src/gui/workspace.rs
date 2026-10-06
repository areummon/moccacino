use std::time::{Duration, Instant};

use iced::Task;

use super::app::CloseRequest;
use super::message::{Menu, Message};
use super::tab::{Tab, TabMachine};
use crate::gui::theme::Family;

const TAB_DOUBLE_CLICK: Duration = Duration::from_millis(350);
pub(crate) const TAB_RENAME_INPUT: &str = "tab-rename";

impl super::app::App {
    pub(crate) fn open_machine_in_new_tab(
        &mut self,
        name: String,
        machine: TabMachine,
    ) -> Task<Message> {
        let mut new_tab = Tab::new_with_name(name);
        new_tab.machine = machine;
        self.tabs.push(new_tab);
        self.active_tab = self.tabs.len() - 1;
        self.get_active_tab_mut().load_machine_to_gui();
        self.fit_view()
    }

    pub(crate) fn choose_startup_module(&mut self, index: usize) -> Task<Message> {
        let family = Family::ALL.get(index).copied().unwrap_or(Family::Finite);
        self.tabs[0] = Tab::new_of(family);
        self.active_tab = 0;
        self.startup_picker_open = false;
        Task::none()
    }

    pub(crate) fn move_startup_selection(&mut self, delta: i32) -> Task<Message> {
        let count = Family::ALL.len() as i32;
        let next = (self.startup_selected as i32 + delta).rem_euclid(count);
        self.startup_selected = next as usize;
        Task::none()
    }

    pub(crate) fn new_tab(&mut self, family: Family) -> Task<Message> {
        self.open_menu = None;
        self.commit_pending_rename();
        self.tabs.push(Tab::new_of(family));
        self.active_tab = self.tabs.len() - 1;
        Task::none()
    }

    pub(crate) fn request_app_close(&mut self) -> Task<Message> {
        self.open_menu = None;
        if self.tabs.iter().any(|tab| tab.insight.unsaved) {
            self.pending_close = Some(CloseRequest::App);
            Task::none()
        } else {
            iced::exit()
        }
    }

    pub(crate) fn request_tab_close(&mut self, index: usize) -> Task<Message> {
        let unsaved = self.tabs.get(index).is_some_and(|tab| tab.insight.unsaved);
        if unsaved && self.tabs.len() > 1 {
            self.pending_close = Some(CloseRequest::Tab(index));
            Task::none()
        } else {
            self.remove_tab(index)
        }
    }

    pub(crate) fn confirm_close_discard(&mut self) -> Task<Message> {
        match self.pending_close.take() {
            Some(CloseRequest::App) => iced::exit(),
            Some(CloseRequest::Tab(index)) => self.remove_tab(index),
            None => Task::none(),
        }
    }

    pub(crate) fn confirm_close_save(&mut self) -> Task<Message> {
        let target = match self.pending_close.take() {
            Some(CloseRequest::Tab(index)) => Some(index),
            Some(CloseRequest::App) => self.tabs.iter().position(|tab| tab.insight.unsaved),
            None => None,
        };
        match target {
            Some(index) if index < self.tabs.len() => {
                self.active_tab = index;
                self.open_save_dialog()
            }
            _ => Task::none(),
        }
    }

    pub(crate) fn remove_tab(&mut self, index: usize) -> Task<Message> {
        if self.tabs.len() > 1 && index < self.tabs.len() {
            self.renaming_tab = None;
            self.tabs.remove(index);
            if self.active_tab > index || self.active_tab >= self.tabs.len() {
                self.active_tab = self.active_tab.saturating_sub(1).min(self.tabs.len() - 1);
            }
        }
        Task::none()
    }

    pub(crate) fn switch_tab(&mut self, index: usize) -> Task<Message> {
        if index >= self.tabs.len() {
            return Task::none();
        }
        self.open_menu = None;
        if self.renaming_tab == Some(index) {
            return Task::none();
        }
        self.commit_pending_rename();
        let now = Instant::now();
        let double = matches!(
            self.last_tab_click,
            Some((last, at)) if last == index && now.duration_since(at) < TAB_DOUBLE_CLICK
        );
        self.active_tab = index;
        if double {
            self.last_tab_click = None;
            self.renaming_tab = Some(index);
            self.tab_rename_text = self.tabs[index].name.clone();
            return iced::widget::text_input::focus(TAB_RENAME_INPUT);
        }
        self.last_tab_click = Some((index, now));
        Task::none()
    }

    pub(crate) fn cycle_tab(&mut self, delta: i32) -> Task<Message> {
        self.commit_pending_rename();
        let count = self.tabs.len() as i32;
        self.active_tab = (self.active_tab as i32 + delta).rem_euclid(count) as usize;
        Task::none()
    }

    pub(crate) fn commit_tab_rename(&mut self) -> Task<Message> {
        self.commit_pending_rename();
        Task::none()
    }

    fn commit_pending_rename(&mut self) {
        if let Some(index) = self.renaming_tab.take() {
            let name = self.tab_rename_text.trim();
            if !name.is_empty() {
                if let Some(tab) = self.tabs.get_mut(index) {
                    tab.name = name.to_string();
                }
            }
        }
    }

    pub(crate) fn cancel_tab_rename(&mut self) {
        self.renaming_tab = None;
    }

    pub(crate) fn toggle_menu(&mut self, menu: Menu) -> Task<Message> {
        self.open_menu = if self.open_menu == Some(menu) { None } else { Some(menu) };
        Task::none()
    }

    pub(crate) fn close_menus(&mut self) -> Task<Message> {
        self.open_menu = None;
        Task::none()
    }

    pub(crate) fn dismiss_modal(&mut self) -> Task<Message> {
        if self.error_message.is_some() {
            return self.close_error();
        }
        if self.pending_close.is_some() {
            self.pending_close = None;
            return Task::none();
        }
        if self.shortcuts_open {
            self.shortcuts_open = false;
            return Task::none();
        }
        if self.get_active_tab().editing_transition_dialog_open {
            return self.cancel_edit_transition_labels();
        }
        if self.get_active_tab().pending_transition_dialog_open {
            return self.cancel_editing();
        }
        if self.regex_export_dialog_open {
            return self.close_regex_export();
        }
        if self.latex_export_dialog_open {
            return self.close_latex_export();
        }
        if self.save_dialog_open {
            return self.cancel_save_dialog();
        }
        if self.load_dialog_open {
            return self.cancel_load_dialog();
        }
        if self.get_active_tab().regex_dialog_open {
            return self.cancel_regex();
        }
        if self.get_active_tab().check_input_dialog_open {
            return self.cancel_check_input();
        }
        if self.get_active_tab().editing_state.is_some() || self.get_active_tab().editing_transition.is_some() {
            return self.cancel_editing();
        }
        Task::none()
    }

    pub(crate) fn close_error(&mut self) -> Task<Message> {
        self.error_message = None;
        Task::none()
    }
}
