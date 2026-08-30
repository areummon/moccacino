use iced::Task;

use super::message::Message;
use super::tab::{Tab, TabMachine};

impl super::app::App {
    /* Opens a generated machine in a fresh named tab. Every transformation
     * that produces a new machine funnels through here, so the layered
     * layout and scroll reset are applied structurally. */
    pub(crate) fn open_machine_in_new_tab(
        &mut self,
        name: String,
        machine: TabMachine,
    ) -> Task<Message> {
        let mut new_tab = Tab::new_with_name(name);
        new_tab.machine = machine;
        self.tabs.push(Box::new(new_tab));
        self.active_tab = self.tabs.len() - 1;
        self.get_active_tab_mut().load_machine_to_gui();
        Task::none()
    }

    /* Startup picker choice: swap the placeholder tab for the chosen
     * family and unlock the main GUI. */
    pub(crate) fn choose_startup_module(&mut self, index: usize) -> Task<Message> {
        self.tabs[0] = match index {
            1 => Box::new(Tab::new_turing()),
            2 => Box::new(Tab::new_pda()),
            3 => Box::new(Tab::new_grammar()),
            _ => Box::new(Tab::new()),
        };
        self.active_tab = 0;
        self.startup_picker_open = false;
        Task::none()
    }

    /* Arrow-key navigation of the startup picker, wrapping at both ends. */
    pub(crate) fn move_startup_selection(&mut self, delta: i32) -> Task<Message> {
        const OPTION_COUNT: i32 = 4;
        let next = (self.startup_selected as i32 + delta).rem_euclid(OPTION_COUNT);
        self.startup_selected = next as usize;
        Task::none()
    }

    pub(crate) fn add_tab(&mut self) -> Task<Message> {
        self.tabs.push(Box::new(Tab::new()));
        self.active_tab = self.tabs.len() - 1;
        Task::none()
    }

    pub(crate) fn add_turing_tab(&mut self) -> Task<Message> {
        self.tabs.push(Box::new(Tab::new_turing()));
        self.active_tab = self.tabs.len() - 1;
        Task::none()
    }

    pub(crate) fn add_pda_tab(&mut self) -> Task<Message> {
        self.tabs.push(Box::new(Tab::new_pda()));
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

    pub(crate) fn toggle_machine_menu(&mut self) -> Task<Message> {
        self.machine_menu_open = !self.machine_menu_open;
        if self.machine_menu_open {
            self.get_active_tab_mut().operations_menu_open = false;
            self.file_menu_open = false;
        }
        Task::none()
    }

    pub(crate) fn toggle_file_menu(&mut self) -> Task<Message> {
        self.file_menu_open = !self.file_menu_open;
        if self.file_menu_open {
            self.machine_menu_open = false;
            self.get_active_tab_mut().operations_menu_open = false;
        }
        Task::none()
    }

    /* Click-away closes whichever dropdown is open. */
    pub(crate) fn close_menus(&mut self) -> Task<Message> {
        self.machine_menu_open = false;
        self.file_menu_open = false;
        self.get_active_tab_mut().operations_menu_open = false;
        Task::none()
    }

    /* Backdrop click on a modal dialog: dismiss the topmost open popup
     * with its own cancel/close semantics. The order mirrors the stack
     * layering in `view` (later layers sit on top). */
    pub(crate) fn dismiss_modal(&mut self) -> Task<Message> {
        if self.get_active_tab().editing_transition_dialog_open {
            return self.cancel_edit_transition_labels();
        }
        if self.get_active_tab().pending_transition_dialog_open
            || self.get_active_tab().editing_state.is_some()
            || self.get_active_tab().editing_transition.is_some()
        {
            return self.cancel_editing();
        }
        if self.regex_export_dialog_open {
            return self.close_regex_export();
        }
        if self.latex_export_dialog_open {
            return self.close_latex_export();
        }
        if self.error_message.is_some() {
            return self.close_error();
        }
        if self.save_dialog_open {
            return self.cancel_save_dialog();
        }
        if self.load_dialog_open {
            return self.cancel_load_dialog();
        }
        if self.get_active_tab().check_result_popup_open {
            return self.close_check_result_popup();
        }
        if self.get_active_tab().regex_dialog_open {
            return self.cancel_regex();
        }
        if self.get_active_tab().check_input_dialog_open {
            return self.cancel_check_input();
        }
        Task::none()
    }

    pub(crate) fn close_error(&mut self) -> Task<Message> {
        self.error_message = None;
        Task::none()
    }
}
