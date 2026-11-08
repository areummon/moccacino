use iced::keyboard::{Key, Modifiers};
use iced::Task;

use crate::platform;
use crate::state_machine::{CanvasMessage, EditorTool};

use super::dialogs::{EDIT_LABEL_INPUT, EDIT_TEXT_INPUT};
use super::message::{Menu, Message};
use super::shortcuts::{self, Action};
use super::tab::LabelEdit;

impl super::app::App {
    pub(crate) fn handle_canvas_message(&mut self, canvas_message: CanvasMessage) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        match canvas_message {
            CanvasMessage::AddState(position) => tab.add_state(position),
            CanvasMessage::MoveState { state_id, new_position } => tab.move_state(state_id, new_position),
            CanvasMessage::DeleteState(state_id) => tab.delete_state(state_id),
            CanvasMessage::DeleteTransition(pair) => tab.delete_transition(pair),
            CanvasMessage::ToggleFinal(state_id) => tab.toggle_final_state(state_id),
            CanvasMessage::MakeInitial(state_id) => tab.set_initial_state(state_id),
            CanvasMessage::RenameState(state_id) => {
                let Some(label) = tab.node(state_id).map(|node| node.label.clone()) else {
                    return Task::none();
                };
                tab.editing_state = Some(state_id);
                tab.edit_text = label;
                return Task::batch([
                    iced::widget::text_input::focus(EDIT_TEXT_INPUT),
                    iced::widget::text_input::select_all(EDIT_TEXT_INPUT),
                ]);
            }
            CanvasMessage::EditTransition(pair) => {
                let Some(labels) = tab.transitions.get(&pair) else {
                    return Task::none();
                };
                tab.label_edit = Some(LabelEdit { pair, inputs: labels.iter().cloned().collect() });
                return iced::widget::text_input::focus(format!("{EDIT_LABEL_INPUT}-0"));
            }
            CanvasMessage::ConnectStates { from, to } => {
                tab.pending_transition = Some((from, to));
                tab.edit_text.clear();
                return iced::widget::text_input::focus(EDIT_TEXT_INPUT);
            }
            CanvasMessage::Scrolled(scroll) => tab.state_machine.set_scroll(scroll),
            CanvasMessage::Zoomed { zoom, scroll } => {
                tab.state_machine.set_zoom(zoom);
                tab.state_machine.set_scroll(scroll);
            }
            CanvasMessage::Viewport(size) => {
                let first_report = self.canvas_viewport.is_none();
                self.canvas_viewport = Some(size);
                if first_report && !self.get_active_tab().states.is_empty() {
                    return self.fit_view();
                }
            }
        }
        Task::none()
    }

    pub(crate) fn handle_key_pressed(&mut self, key: Key, modifiers: Modifiers, captured: bool) -> Task<Message> {
        if self.startup_picker_open {
            return match shortcuts::find(shortcuts::STARTUP_SHORTCUTS, &key, modifiers) {
                Some((Action::StartupMove(delta), _)) => self.move_startup_selection(delta),
                Some((Action::StartupChoose, _)) => self.choose_startup_module(self.startup_selected),
                Some((Action::Escape, _)) if !platform::IS_WEB => iced::exit(),
                _ => Task::none(),
            };
        }
        let Some((action, with_ctrl)) = shortcuts::find(shortcuts::SHORTCUTS, &key, modifiers) else {
            return Task::none();
        };
        if action == Action::Escape {
            return self.escape();
        }
        if self.any_modal_open() || (captured && !with_ctrl) || (action.needs_canvas() && self.get_active_tab().is_grammar()) {
            return Task::none();
        }
        self.perform(action)
    }

    fn escape(&mut self) -> Task<Message> {
        if self.renaming_tab.take().is_some() {
            return Task::none();
        }
        if self.open_menu.take().is_some() {
            return Task::none();
        }
        if self.any_modal_open() {
            return self.dismiss_modal();
        }
        self.get_active_tab_mut().set_active_tool(EditorTool::Arrow);
        Task::none()
    }

    fn perform(&mut self, action: Action) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        match action {
            Action::Tool(tool) => tab.set_active_tool(tool),
            Action::ToggleDelete => {
                let next = if tab.state_machine.is_deletion_mode() { EditorTool::Arrow } else { EditorTool::Delete };
                tab.set_active_tool(next);
            }
            Action::HoldDelete => {
                tab.state_machine.stash_tool();
                tab.set_active_tool(EditorTool::Delete);
            }
            Action::Escape => return self.escape(),
            Action::ZoomIn => return self.zoom_by(1.2),
            Action::ZoomOut => return self.zoom_by(1.0 / 1.2),
            Action::ZoomReset => return self.zoom_reset(),
            Action::Fit => return self.fit_view(),
            Action::PlayPause => {
                if tab.run.is_some() {
                    return self.run_toggle_play();
                }
            }
            Action::Step => return self.run_step(),
            Action::NewTabMenu => self.open_menu = Some(Menu::NewFromTabs),
            Action::CloseTab => return self.request_tab_close(self.active_tab),
            Action::PreviousTab => return self.cycle_tab(-1),
            Action::NextTab => return self.cycle_tab(1),
            Action::Open => return self.open_load_dialog(),
            Action::Save => return self.open_save_dialog(),
            Action::ToggleTheme => return self.toggle_theme(),
            Action::Shortcuts => return self.toggle_shortcuts(),
            // The grammar editor binds Ctrl+Enter itself so the key never inserts a newline.
            Action::ParseGrammar => {}
            Action::StartupMove(_) | Action::StartupChoose => {}
        }
        Task::none()
    }

    pub(crate) fn handle_key_released(&mut self, key: Key) -> Task<Message> {
        if shortcuts::matches(Action::HoldDelete, &key, Modifiers::default()) && self.get_active_tab().state_machine.is_deletion_mode() {
            self.get_active_tab_mut().state_machine.restore_tool();
        }
        Task::none()
    }
}
