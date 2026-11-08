use std::time::Duration;

use iced::time::Instant;
use iced::Task;

use super::message::Message;
use super::tab::Tab;
use crate::family::Family;

const TAB_DOUBLE_CLICK: Duration = Duration::from_millis(350);
pub(crate) const TAB_RENAME_INPUT: &str = "tab-rename";
pub(crate) const TAB_STRIP: &str = "tab-strip";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CloseRequest {
    App,
    Tab(usize),
}

impl super::app::App {
    pub(crate) fn push_tab(&mut self, tab: Tab) {
        self.open_menu = None;
        self.commit_pending_rename();
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
    }

    pub(crate) fn open_in_new_tab(&mut self, tab: Tab) -> Task<Message> {
        let fit = !tab.is_grammar();
        self.push_tab(tab);
        if fit {
            return self.fit_view();
        }
        Task::none()
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

    fn remove_tab(&mut self, index: usize) -> Task<Message> {
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

    pub(crate) fn commit_pending_rename(&mut self) {
        if let Some(index) = self.renaming_tab.take() {
            let name = self.tab_rename_text.trim();
            if !name.is_empty() {
                if let Some(tab) = self.tabs.get_mut(index) {
                    tab.name = name.to_string();
                }
            }
        }
    }
}
