use iced::keyboard;
use iced::Task;

use crate::state_machine;

use super::message::Message;

impl super::app::App {
    pub(crate) fn handle_canvas_message(&mut self, canvas_message: state_machine::CanvasMessage) -> Task<Message> {
        match canvas_message {
            state_machine::CanvasMessage::AddState(state) => self.canvas_add_state(state),
            state_machine::CanvasMessage::AddTransition(transition) => self.canvas_add_transition(transition),
            state_machine::CanvasMessage::MoveState { state_id, new_position } => self.canvas_move_state(state_id, new_position),
            state_machine::CanvasMessage::StateClicked(state_id) => self.canvas_state_clicked(state_id),
            state_machine::CanvasMessage::TransitionClicked(transition_index) => self.canvas_transition_clicked(transition_index),
            state_machine::CanvasMessage::StateDoubleClicked(state_id) => self.canvas_state_double_clicked(state_id),
            // TODO: TransitionDoubleClicked editing logic will be updated later
            state_machine::CanvasMessage::TransitionDoubleClicked(_transition_index) => {
                // No-op for now
                Task::none()
            }
            state_machine::CanvasMessage::RequestTransitionLabel { from_state_id, to_state_id, from_point, to_point } => {
                self.canvas_request_transition_label(from_state_id, to_state_id, from_point, to_point)
            }
        }
    }

    fn canvas_add_state(&mut self, mut state: state_machine::StateNode) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        if !active_tab.deletion_mode {
            let assigned_id = active_tab.state_machine.next_id;
            state.id = assigned_id;
            let index = active_tab.states.len();
            active_tab.states.push(state);
            active_tab.state_id_to_index.insert(assigned_id, index);
            active_tab.state_machine.next_id += 1;
            active_tab.state_machine.request_redraw();
        }
        Task::none()
    }

    fn canvas_add_transition(&mut self, transition: state_machine::Transition) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        let key = (transition.from_state_id, transition.to_state_id);
        let label = transition.label.to_string();
        let entry = active_tab.transitions.entry(key).or_insert_with(indexmap::IndexSet::new);
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
        // No need to update transition points, as they are not stored per transition anymore
        active_tab.state_machine.request_redraw();
        active_tab.state_id_to_index.clear();
        for (index, state) in active_tab.states.iter().enumerate() {
            active_tab.state_id_to_index.insert(state.id, index);
        }
        Task::none()
    }

    fn canvas_state_clicked(&mut self, state_id: usize) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        if active_tab.deletion_mode {
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
        } else if let Some(index) = active_tab.state_id_to_index.get(&state_id) {
            if let Some(state) = active_tab.states.get(*index) {
                active_tab.editing_state = Some(state_id);
                active_tab.edit_text = state.label.to_string();
                active_tab.editing_transition = None;
            }
        }
        Task::none()
    }

    fn canvas_transition_clicked(&mut self, transition_index: usize) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        // Map index to (from, to) pair by order
        let (from, to) = active_tab.transitions.iter().enumerate()
            .find(|(idx, _)| *idx == transition_index)
            .map(|(_, ((f, t), _))| (*f, *t))
            .unwrap();

        if active_tab.deletion_mode {
            // Remove the transition in deletion mode
            active_tab.transitions.remove(&(from, to));
            active_tab.state_machine.request_redraw();
        } else {
            // Open edit dialog in normal mode
            active_tab.editing_transition_pair = Some((from, to));
            active_tab.editing_transition_labels = active_tab.transitions.get(&(from, to)).unwrap().iter().cloned().collect();
            active_tab.editing_transition_label_inputs = active_tab.editing_transition_labels.clone();
            active_tab.editing_transition_dialog_open = true;
            active_tab.state_machine.request_redraw();
        }
        Task::none()
    }

    fn canvas_state_double_clicked(&mut self, state_id: usize) -> Task<Message> {
        let active_tab = self.get_active_tab_mut();
        if let Some(index) = active_tab.state_id_to_index.get(&state_id) {
            if let Some(state) = active_tab.states.get(*index) {
                active_tab.editing_state = Some(state_id);
                active_tab.edit_text = state.label.to_string();
                active_tab.editing_transition = None;
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
        Task::none()
    }

    pub(crate) fn handle_key_pressed(&mut self, key: keyboard::Key) -> Task<Message> {
        match key {
            keyboard::Key::Named(keyboard::key::Named::Control) => {
                self.get_active_tab_mut().state_machine.set_ctrl_pressed(true);
            }
            keyboard::Key::Named(keyboard::key::Named::Shift) => {
                self.get_active_tab_mut().state_machine.set_shift_pressed(true);
            }
            keyboard::Key::Named(keyboard::key::Named::Alt) => {
                self.get_active_tab_mut().state_machine.set_alt_pressed(true);
            }
            keyboard::Key::Named(keyboard::key::Named::Tab) => {
                let active_tab = self.get_active_tab_mut();
                active_tab.deletion_mode = !active_tab.deletion_mode;
                active_tab.state_machine.set_deletion_mode(active_tab.deletion_mode);
                active_tab.state_machine.request_redraw();
            }
            keyboard::Key::Named(keyboard::key::Named::Delete) => {
                self.get_active_tab_mut().deletion_mode = true;
                self.get_active_tab_mut().state_machine.set_deletion_mode(true);
                self.get_active_tab_mut().state_machine.request_redraw();
            }
            _ => {}
        }
        Task::none()
    }

    pub(crate) fn handle_key_released(&mut self, key: keyboard::Key) -> Task<Message> {
        match key {
            keyboard::Key::Named(keyboard::key::Named::Control) => {
                self.get_active_tab_mut().state_machine.set_ctrl_pressed(false);
            }
            keyboard::Key::Named(keyboard::key::Named::Shift) => {
                self.get_active_tab_mut().state_machine.set_shift_pressed(false);
            }
            keyboard::Key::Named(keyboard::key::Named::Alt) => {
                self.get_active_tab_mut().state_machine.set_alt_pressed(false);
            }
            keyboard::Key::Named(keyboard::key::Named::Delete) => {
                self.get_active_tab_mut().deletion_mode = false;
                self.get_active_tab_mut().state_machine.set_deletion_mode(false);
                self.get_active_tab_mut().state_machine.request_redraw();
            }
            _ => {}
        }
        Task::none()
    }
}
