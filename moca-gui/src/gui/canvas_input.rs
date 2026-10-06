use iced::keyboard::{self, key::Named, Key, Modifiers};
use iced::Task;

use crate::state_machine::{self, EditorTool};

use super::dialogs::{EDIT_LABEL_INPUT, EDIT_TEXT_INPUT};
use super::message::{Menu, Message};

impl super::app::App {
    pub(crate) fn handle_canvas_message(&mut self, canvas_message: state_machine::CanvasMessage) -> Task<Message> {
        match canvas_message {
            state_machine::CanvasMessage::AddState(state) => self.canvas_add_state(state),
            state_machine::CanvasMessage::AddTransition(transition) => self.canvas_add_transition(transition),
            state_machine::CanvasMessage::MoveState { state_id, new_position } => self.canvas_move_state(state_id, new_position),
            state_machine::CanvasMessage::StateClicked(state_id) => self.canvas_state_clicked(state_id),
            state_machine::CanvasMessage::TransitionClicked(pair) => self.canvas_transition_clicked(pair),
            state_machine::CanvasMessage::StateDoubleClicked(state_id) => self.canvas_state_double_clicked(state_id),
            state_machine::CanvasMessage::TransitionDoubleClicked(_) => Task::none(),
            state_machine::CanvasMessage::RequestTransitionLabel { from_state_id, to_state_id, from_point, to_point } => {
                self.canvas_request_transition_label(from_state_id, to_state_id, from_point, to_point)
            }
            state_machine::CanvasMessage::Scrolled(scroll) => self.canvas_scrolled(scroll),
            state_machine::CanvasMessage::Zoomed { zoom, scroll } => {
                let active_tab = self.get_active_tab_mut();
                active_tab.state_machine.set_zoom(zoom);
                active_tab.state_machine.set_scroll(scroll);
                active_tab.state_machine.request_redraw();
                Task::none()
            }
            state_machine::CanvasMessage::Viewport(size) => {
                let first_report = self.canvas_viewport.is_none();
                self.canvas_viewport = Some(size);
                if first_report && !self.get_active_tab().states.is_empty() {
                    return self.fit_view();
                }
                Task::none()
            }
        }
    }

    pub(crate) fn canvas_scrolled(&mut self, scroll: iced::Vector) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        active_tab.state_machine.set_scroll(scroll);
        active_tab.state_machine.request_redraw();
        Task::none()
    }

    fn canvas_add_state(&mut self, mut state: state_machine::StateNode) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        let assigned_id = active_tab.state_machine.next_id;
        state.id = assigned_id;
        let index = active_tab.states.len();
        active_tab.states.push(state);
        active_tab.state_id_to_index.insert(assigned_id, index);
        active_tab.state_machine.next_id += 1;
        if active_tab.states.len() == 1 && active_tab.initial_state.is_none() {
            active_tab.initial_state = Some(assigned_id);
        }
        active_tab.state_machine.request_redraw();
        Task::none()
    }

    fn canvas_add_transition(&mut self, transition: state_machine::Transition) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        let key = (transition.from_state_id, transition.to_state_id);
        let label = transition.label.to_string();
        let entry = active_tab.transitions.entry(key).or_default();
        if !entry.contains(&label) {
            entry.insert(label);
            active_tab.state_machine.request_redraw();
        }
        Task::none()
    }

    fn canvas_move_state(&mut self, state_id: usize, new_position: iced::Point) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        if let Some(index) = active_tab.state_id_to_index.get(&state_id) {
            if let Some(state) = active_tab.states.get_mut(*index) {
                state.position = new_position;
            }
        }
        active_tab.state_machine.request_redraw();
        Task::none()
    }

    fn canvas_state_clicked(&mut self, state_id: usize) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        if active_tab.state_machine.is_deletion_mode() {
            active_tab.states.retain(|s| s.id != state_id);
            active_tab.transitions.retain(|&(from, to), _| from != state_id && to != state_id);
            if active_tab.initial_state == Some(state_id) {
                active_tab.initial_state = None;
            }
            active_tab.final_states.remove(&state_id);
            active_tab.state_id_to_index.clear();
            for (index, state) in active_tab.states.iter().enumerate() {
                active_tab.state_id_to_index.insert(state.id, index);
            }
            active_tab.state_machine.request_redraw();
        } else if active_tab.state_machine.is_shift_pressed() {
            active_tab.toggle_final_state(state_id);
        } else if active_tab.state_machine.is_alt_pressed() {
            active_tab.set_initial_state(state_id);
        }
        Task::none()
    }

    fn canvas_transition_clicked(&mut self, (from, to): (usize, usize)) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        let Some(labels) = active_tab.transitions.get(&(from, to)) else {
            return Task::none();
        };
        if active_tab.state_machine.is_deletion_mode() {
            active_tab.transitions.remove(&(from, to));
            active_tab.state_machine.request_redraw();
            Task::none()
        } else {
            active_tab.editing_transition_labels = labels.iter().cloned().collect();
            active_tab.editing_transition_pair = Some((from, to));
            active_tab.editing_transition_label_inputs = active_tab.editing_transition_labels.clone();
            active_tab.editing_transition_dialog_open = true;
            active_tab.state_machine.request_redraw();
            iced::widget::text_input::focus(format!("{EDIT_LABEL_INPUT}-0"))
        }
    }

    fn canvas_state_double_clicked(&mut self, state_id: usize) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        if let Some(index) = active_tab.state_id_to_index.get(&state_id) {
            if let Some(state) = active_tab.states.get(*index) {
                active_tab.editing_state = Some(state_id);
                active_tab.edit_text = state.label.to_string();
                active_tab.editing_transition = None;
                return Task::batch([
                    iced::widget::text_input::focus(EDIT_TEXT_INPUT),
                    iced::widget::text_input::select_all(EDIT_TEXT_INPUT),
                ]);
            }
        }
        Task::none()
    }

    fn canvas_request_transition_label(
        &mut self,
        from_state_id: usize,
        to_state_id: usize,
        from_point: iced::Point,
        to_point: iced::Point,
    ) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        active_tab.pending_transition = Some((from_state_id, to_state_id, from_point, to_point));
        active_tab.pending_transition_label = String::new();
        active_tab.pending_transition_dialog_open = true;
        active_tab.state_machine.request_redraw();
        iced::widget::text_input::focus(EDIT_TEXT_INPUT)
    }

    pub(crate) fn handle_key_pressed(&mut self, key: Key, modifiers: Modifiers, captured: bool) -> Task<Message> {
        match key {
            Key::Named(Named::Control) => self.get_active_tab_mut().state_machine.set_ctrl_pressed(true),
            Key::Named(Named::Shift) => self.get_active_tab_mut().state_machine.set_shift_pressed(true),
            Key::Named(Named::Alt) => self.get_active_tab_mut().state_machine.set_alt_pressed(true),
            _ => {}
        }

        if self.startup_picker_open {
            return match key {
                Key::Named(Named::ArrowLeft) => self.move_startup_selection(-1),
                Key::Named(Named::ArrowRight) => self.move_startup_selection(1),
                Key::Named(Named::ArrowUp) => self.move_startup_selection(-2),
                Key::Named(Named::ArrowDown) => self.move_startup_selection(2),
                Key::Named(Named::Enter) => self.choose_startup_module(self.startup_selected),
                Key::Named(Named::Escape) => iced::exit(),
                _ => Task::none(),
            };
        }

        if let Key::Named(Named::Escape) = key {
            if self.renaming_tab.is_some() {
                self.cancel_tab_rename();
                return Task::none();
            }
            if self.open_menu.is_some() {
                return self.close_menus();
            }
            if self.any_modal_open() {
                return self.dismiss_modal();
            }
            self.get_active_tab_mut().set_active_tool(EditorTool::Arrow);
            return Task::none();
        }

        if self.any_modal_open() {
            return Task::none();
        }

        if modifiers.command() {
            return match key.as_ref() {
                Key::Character("t") => {
                    self.open_menu = Some(Menu::NewFromTabs);
                    Task::none()
                }
                Key::Character("w") => self.request_tab_close(self.active_tab),
                Key::Character("o") => self.open_load_dialog(),
                Key::Character("s") => self.open_save_dialog(),
                Key::Character("=") | Key::Character("+") => self.update(Message::ZoomIn),
                Key::Character("-") => self.update(Message::ZoomOut),
                Key::Character("0") => self.update(Message::ZoomReset),
                Key::Character("l") if modifiers.shift() => self.update(Message::ToggleTheme),
                Key::Character("L") => self.update(Message::ToggleTheme),
                Key::Named(Named::Tab) => self.cycle_tab(if modifiers.shift() { -1 } else { 1 }),
                _ => Task::none(),
            };
        }

        if captured {
            return Task::none();
        }

        let on_canvas = !self.get_active_tab().machine.is_grammar();
        match key.as_ref() {
            Key::Named(Named::F1) | Key::Character("?") => self.update(Message::ToggleShortcuts),
            Key::Named(Named::Tab) if on_canvas => {
                let active_tab = self.get_active_tab_mut();
                let next = if active_tab.state_machine.is_deletion_mode() {
                    EditorTool::Arrow
                } else {
                    EditorTool::Delete
                };
                active_tab.state_machine.set_tool(next);
                active_tab.state_machine.request_redraw();
                Task::none()
            }
            Key::Named(Named::Delete) if on_canvas => {
                let active_tab = self.get_active_tab_mut();
                active_tab.state_machine.stash_tool();
                active_tab.state_machine.set_tool(EditorTool::Delete);
                active_tab.state_machine.request_redraw();
                Task::none()
            }
            Key::Named(Named::Space) if on_canvas => self.toggle_active_play(),
            Key::Named(Named::ArrowRight) if on_canvas => self.run_step(),
            Key::Character("f") if on_canvas => self.fit_view(),
            Key::Character(digit) if on_canvas => {
                let tool = EditorTool::ALL.into_iter().find(|tool| tool.shortcut() == digit);
                if let Some(tool) = tool {
                    self.get_active_tab_mut().set_active_tool(tool);
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub(crate) fn handle_key_released(&mut self, key: keyboard::Key) -> Task<Message> {
        match key {
            Key::Named(Named::Control) => {
                self.get_active_tab_mut().state_machine.set_ctrl_pressed(false);
            }
            Key::Named(Named::Shift) => {
                self.get_active_tab_mut().state_machine.set_shift_pressed(false);
            }
            Key::Named(Named::Alt) => {
                self.get_active_tab_mut().state_machine.set_alt_pressed(false);
            }
            Key::Named(Named::Delete) => {
                if self.get_active_tab().state_machine.is_deletion_mode() {
                    self.get_active_tab_mut().state_machine.restore_tool();
                    self.get_active_tab_mut().state_machine.request_redraw();
                }
            }
            _ => {}
        }
        Task::none()
    }

    pub(crate) fn select_tool(&mut self, tool: EditorTool) -> Task<Message> {
        self.get_active_tab_mut().set_active_tool(tool);
        Task::none()
    }

    fn toggle_active_play(&mut self) -> Task<Message> {
        let (loaded, _, _) = self.get_active_tab().run_state();
        if !loaded {
            return Task::none();
        }
        self.run_toggle_play()
    }

}
