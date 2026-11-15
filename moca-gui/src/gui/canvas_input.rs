use iced::keyboard::{Key, Modifiers};
use iced::Task;

use crate::platform;
use crate::state_machine::{CanvasMessage, EditorTool, Hit};

use super::dialogs::{EDIT_LABEL_INPUT, EDIT_TEXT_INPUT};
use super::message::{ContextMenu, Menu, Message};
use super::shortcuts::{self, Action};
use super::tab::LabelEdit;

impl super::app::App {
    pub(crate) fn handle_canvas_message(&mut self, canvas_message: CanvasMessage) -> Task<Message> {
        if !matches!(canvas_message, CanvasMessage::Viewport(_)) {
            self.context_menu = None;
        }
        let tab = self.get_active_tab_mut();
        match canvas_message {
            CanvasMessage::AddState(position) => tab.drawing.add_state(position),
            CanvasMessage::MoveState { state_id, new_position } => tab.move_state(state_id, new_position),
            CanvasMessage::DeleteState(state_id) => tab.drawing.delete_state(state_id),
            CanvasMessage::DeleteTransition(pair) => tab.drawing.delete_transition(pair),
            CanvasMessage::ToggleFinal(state_id) => tab.drawing.toggle_final_state(state_id),
            CanvasMessage::MakeInitial(state_id) => tab.drawing.set_initial_state(state_id),
            CanvasMessage::RenameState(state_id) => {
                let Some(label) = tab.drawing.node(state_id).map(|node| node.label.clone()) else {
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
                let Some(labels) = tab.drawing.transitions.get(&pair) else {
                    return Task::none();
                };
                tab.label_edit = Some(LabelEdit { pair, inputs: labels.iter().cloned().collect() });
                return iced::widget::text_input::focus(format!("{EDIT_LABEL_INPUT}-0"));
            }
            CanvasMessage::ConnectStates { from, to } => {
                // An undo can remove the source while the gesture is still pending.
                if tab.drawing.node(from).is_none() || tab.drawing.node(to).is_none() {
                    return Task::none();
                }
                tab.pending_transition = Some((from, to));
                tab.edit_text.clear();
                return iced::widget::text_input::focus(EDIT_TEXT_INPUT);
            }
            CanvasMessage::Select(selection) => tab.state_machine.set_selection(selection),
            CanvasMessage::ContextMenu { hit, at } => {
                if let Hit::State(state_id) = hit {
                    if !tab.state_machine.selection().contains(&state_id) {
                        tab.state_machine.set_selection([state_id].into());
                    }
                }
                self.context_menu = Some(ContextMenu { hit, at });
            }
            CanvasMessage::Scrolled(scroll) => tab.state_machine.set_scroll(scroll),
            CanvasMessage::Zoomed { zoom, scroll } => {
                tab.state_machine.set_zoom(zoom);
                tab.state_machine.set_scroll(scroll);
            }
            CanvasMessage::Viewport(size) => {
                let first_report = self.canvas_viewport.is_none();
                self.canvas_viewport = Some(size);
                if first_report && !self.get_active_tab().drawing.states.is_empty() {
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
        let typing = captured && (!with_ctrl || action.yields_to_text_fields());
        if self.any_modal_open() || typing || (action.needs_canvas() && self.get_active_tab().is_grammar()) {
            return Task::none();
        }
        self.perform(action)
    }

    fn escape(&mut self) -> Task<Message> {
        if self.renaming_tab.take().is_some() {
            return Task::none();
        }
        if self.open_menu.take().is_some() || self.context_menu.take().is_some() {
            return Task::none();
        }
        if self.any_modal_open() {
            return self.dismiss_modal();
        }
        let tab = self.get_active_tab_mut();
        tab.set_active_tool(EditorTool::Arrow);
        tab.state_machine.set_selection(Default::default());
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
            Action::HoldDelete if !tab.state_machine.selection().is_empty() => return self.delete_selection(),
            Action::HoldDelete => {
                tab.state_machine.stash_tool();
                tab.set_active_tool(EditorTool::Delete);
            }
            Action::Escape => return self.escape(),
            Action::Undo => return self.undo(),
            Action::Redo => return self.redo(),
            Action::SelectAll => return self.select_all(),
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
