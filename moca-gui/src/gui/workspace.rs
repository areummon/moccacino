use iced::Task;

use super::message::Message;
use super::tab::Tab;

impl super::app::App {
    pub(crate) fn add_tab(&mut self) -> Task<Message> {
        self.tabs.push(Box::new(Tab::new()));
        self.active_tab = self.tabs.len() - 1;
        Task::none()
    }

    pub(crate) fn remove_tab(&mut self, index: usize) -> Task<Message> {
        if self.tabs.len() > 1 {
            self.tabs.remove(index);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
        Task::none()
    }

    pub(crate) fn switch_tab(&mut self, index: usize) -> Task<Message> {
        if index < self.tabs.len() {
            self.active_tab = index;
        }
        Task::none()
    }

    pub(crate) fn close_error(&mut self) -> Task<Message> {
        self.error_message = None;
        Task::none()
    }
}
