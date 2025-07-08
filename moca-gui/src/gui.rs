use iced::keyboard;
use iced::widget::{button, container, horizontal_space, hover, row, text, column, stack};
use iced::{Element, Alignment, Event, Subscription, Task, Length};
use std::collections::HashMap;

use crate::state_machine;
use crate::tikz_export;

use moca_data::finite_automata::FiniteAutomata;
use moca_data::state_machine::StateMachine;

#[derive(Debug, Clone)]
pub enum Message {
    Canvas(state_machine::CanvasMessage), 
    KeyPressed(keyboard::Key),
    KeyReleased(keyboard::Key),
    Clear,
    EditTextChanged(String),
    FinishEditing,
    CancelEditing,
    ToggleOperationsMenu,
    CheckInput,
    DfaToNfa,
    Minimize,
    CheckInputTextChanged(String),
    SubmitCheckInput,
    CancelCheckInput,
    CloseCheckResultPopup,
    AddTab,
    RemoveTab(usize),
    SwitchTab(usize),
    CloseError,
    OpenLatexExport,
    CloseLatexExport,
    CopyLatexExport,
}

#[derive(Default)]
struct Tab {
    state_machine: state_machine::State,
    transitions: Vec<state_machine::Transition>,
    states: Vec<state_machine::StateNode>,
    state_id_to_index: HashMap<usize, usize>,
    machine: FiniteAutomata,
    initial_state: Option<usize>, 
    final_states: std::collections::HashSet<usize>, 
    editing_state: Option<usize>,
    editing_transition: Option<usize>,
    edit_text: String,
    operations_menu_open: bool,
    check_input_dialog_open: bool,
    check_input_text: String,
    check_result_popup_open: bool,
    check_input_result: Option<bool>,
    deletion_mode: bool,
    name: String, 
}

impl Tab {
    fn new() -> Self {
        let mut tab = Self::default();
        tab.state_machine.reset_id_counter();
        tab.machine = FiniteAutomata::default(); 
        tab.name = "Machine".to_string(); 
        tab
    }

    fn new_with_name(name: String) -> Self {
        let mut tab = Self::new();
        tab.name = name;
        tab
    }
}

#[derive(Default)]
pub struct App {
    tabs: Vec<Box<Tab>>,
    active_tab: usize,
    error_message: Option<String>, 
    latex_export_dialog_open: bool,
    latex_export_code: Option<String>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let mut app = Self::default();
        app.tabs.push(Box::new(Tab::new()));
        (app, Task::none())
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::event::listen_with(|event, _status, _| match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                Some(Message::KeyPressed(key))
            }
            Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => {
                Some(Message::KeyReleased(key))
            }
            _ => None,
        })
    }

    fn get_active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    fn get_active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Canvas(canvas_message) => {
                let active_tab = self.get_active_tab_mut();
                match canvas_message {
                    state_machine::CanvasMessage::AddState(mut state) => {
                        if !active_tab.deletion_mode {
                            let assigned_id = active_tab.state_machine.next_id;
                            state.id = assigned_id;
                            let index = active_tab.states.len();
                            active_tab.states.push(state);
                            active_tab.state_id_to_index.insert(assigned_id, index);
                            active_tab.state_machine.next_id += 1;
                            active_tab.state_machine.request_redraw();
                        }
                    }
                    state_machine::CanvasMessage::AddTransition(transition) => {
                        active_tab.transitions.push(transition);
                        active_tab.state_machine.request_redraw();
                    }
                    state_machine::CanvasMessage::MoveState { state_id, new_position } => {
                        if let Some(index) = active_tab.state_id_to_index.get(&state_id) {
                            if let Some(state) = active_tab.states.get_mut(*index) {
                                state.position = new_position;
                            }
                        }
                        for transition in &mut active_tab.transitions {
                            if transition.from_state_id == state_id {
                                transition.from_point = new_position;
                            }
                            if transition.to_state_id == state_id {
                                transition.to_point = new_position;
                            }
                        }
                        active_tab.state_machine.request_redraw();
                        active_tab.state_id_to_index.clear();
                        for (index, state) in active_tab.states.iter().enumerate() {
                            active_tab.state_id_to_index.insert(state.id, index);
                        }
                    }
                    state_machine::CanvasMessage::StateClicked(state_id) => {
                        if active_tab.deletion_mode {
                            active_tab.states.retain(|s| s.id != state_id);
                            active_tab.transitions.retain(|transition| {
                                transition.from_state_id != state_id && transition.to_state_id != state_id
                            });
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
                            self.toggle_final_state(state_id);
                        } else if active_tab.state_machine.is_alt_pressed() {
                            self.set_initial_state(state_id);
                        } else if let Some(index) = active_tab.state_id_to_index.get(&state_id) {
                            if let Some(state) = active_tab.states.get(*index) {
                                active_tab.editing_state = Some(state_id);
                                active_tab.edit_text = state.label.to_string();
                                active_tab.editing_transition = None;
                            }
                        }
                    }
                    state_machine::CanvasMessage::TransitionClicked(transition_index) => {
                        if active_tab.deletion_mode {
                            if transition_index < active_tab.transitions.len() {
                                active_tab.transitions.remove(transition_index);
                                active_tab.state_machine.request_redraw();
                            }
                        } else if let Some(transition) = active_tab.transitions.get(transition_index) {
                            active_tab.editing_transition = Some(transition_index);
                            active_tab.edit_text = transition.label.to_string();
                            active_tab.editing_state = None;
                        }
                    }
                    state_machine::CanvasMessage::StateDoubleClicked(state_id) => {
                        if let Some(index) = active_tab.state_id_to_index.get(&state_id) {
                            if let Some(state) = active_tab.states.get(*index) {
                                active_tab.editing_state = Some(state_id);
                                active_tab.edit_text = state.label.to_string();
                                active_tab.editing_transition = None;
                            }
                        }
                    }
                    state_machine::CanvasMessage::TransitionDoubleClicked(transition_index) => {
                        if let Some(transition) = active_tab.transitions.get(transition_index) {
                            active_tab.editing_transition = Some(transition_index);
                            active_tab.edit_text = transition.label.to_string();
                            active_tab.editing_state = None;
                        }
                    }
                }
                Task::none()
            }
            Message::KeyPressed(key) => {
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
            Message::KeyReleased(key) => {
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
            Message::Clear => {
                self.get_active_tab_mut().state_machine.reset_id_counter();
                self.get_active_tab_mut().state_machine.request_redraw();
                self.get_active_tab_mut().transitions.clear();
                self.get_active_tab_mut().states.clear();
                self.get_active_tab_mut().state_id_to_index.clear();
                self.get_active_tab_mut().initial_state = None;
                self.get_active_tab_mut().final_states.clear();
                self.get_active_tab_mut().machine = FiniteAutomata::default();
                self.get_active_tab_mut().check_input_dialog_open = false;
                self.get_active_tab_mut().check_input_text.clear();
                self.get_active_tab_mut().check_result_popup_open = false;
                self.get_active_tab_mut().check_input_result = None;
                self.get_active_tab_mut().deletion_mode = false;
                self.get_active_tab_mut().state_machine.set_deletion_mode(false);
                Task::none()
            }
            Message::EditTextChanged(text) => {
                self.get_active_tab_mut().edit_text = text;
                Task::none()
            }
            Message::FinishEditing => {
                let active_tab = self.get_active_tab_mut();
                if let Some(state_id) = active_tab.editing_state {
                    if let Some(index) = active_tab.state_id_to_index.get(&state_id) {
                        if let Some(state) = active_tab.states.get_mut(*index) {
                            let edit_text = active_tab.edit_text.clone();
                            state.label = Box::leak(edit_text.into_boxed_str());
                        }
                    }
                }
                if let Some(transition_index) = active_tab.editing_transition {
                    if let Some(transition) = active_tab.transitions.get_mut(transition_index) {
                        let edit_text = active_tab.edit_text.clone();
                        transition.label = Box::leak(edit_text.into_boxed_str());
                    }
                }
                active_tab.editing_state = None;
                active_tab.editing_transition = None;
                active_tab.edit_text.clear();
                active_tab.state_machine.request_redraw();
                Task::none()
            }
            Message::CancelEditing => {
                self.get_active_tab_mut().editing_state = None;
                self.get_active_tab_mut().editing_transition = None;
                self.get_active_tab_mut().edit_text.clear();
                Task::none()
            }
            Message::ToggleOperationsMenu => {
                self.get_active_tab_mut().operations_menu_open = !self.get_active_tab_mut().operations_menu_open;
                Task::none()
            }
            Message::CheckInput => {
                self.get_active_tab_mut().operations_menu_open = false;
                self.get_active_tab_mut().check_input_dialog_open = true;
                self.get_active_tab_mut().check_input_text = String::new();
                Task::none()
            }
            Message::DfaToNfa => {
                self.get_active_tab_mut().operations_menu_open = false;
                
                self.sync_gui_to_finite_automata();
                
                if self.get_active_tab().machine.is_deterministic() {
                    self.error_message = Some("Cannot convert: The automaton is already deterministic.".to_string());
                    return Task::none();
                }

                let dfa = self.get_active_tab().machine.to_dfa();
                
                let mut new_tab = Tab::new_with_name("DFA".to_string());
                new_tab.machine = dfa;
                self.tabs.push(Box::new(new_tab));
                self.active_tab = self.tabs.len() - 1;
                
                self.load_finite_automata_to_gui();
                Task::none()
            }
            Message::Minimize => {
                self.get_active_tab_mut().operations_menu_open = false;
                
                self.sync_gui_to_finite_automata();
                
                if !self.get_active_tab().machine.is_deterministic() {
                    self.error_message = Some("Cannot minimize: The automaton must be deterministic.".to_string());
                    return Task::none();
                }

                let minimized = self.get_active_tab().machine.minimize();
                
                let mut new_tab = Tab::new_with_name("Minimized".to_string());
                new_tab.machine = minimized;
                self.tabs.push(Box::new(new_tab));
                self.active_tab = self.tabs.len() - 1;
                
                self.load_finite_automata_to_gui();
                Task::none()
            }
            Message::CheckInputTextChanged(text) => {
                self.get_active_tab_mut().check_input_text = text;
                Task::none()
            }
            Message::SubmitCheckInput => {
                let mut input = self.get_active_tab().check_input_text.clone();
                if !input.is_empty() {
                    self.sync_gui_to_finite_automata();
                    let result = self.get_active_tab().machine.check_input(&mut input);
                    self.get_active_tab_mut().check_input_result = Some(result);
                    self.get_active_tab_mut().check_result_popup_open = true;
                }
                self.get_active_tab_mut().check_input_dialog_open = false;
                Task::none()
            }
            Message::CancelCheckInput => {
                self.get_active_tab_mut().check_input_dialog_open = false;
                Task::none()
            }
            Message::CloseCheckResultPopup => {
                self.get_active_tab_mut().check_result_popup_open = false;
                Task::none()
            }
            Message::AddTab => {
                self.tabs.push(Box::new(Tab::new()));
                self.active_tab = self.tabs.len() - 1;
                Task::none()
            }
            Message::RemoveTab(index) => {
                if self.tabs.len() > 1 {
                    self.tabs.remove(index);
                    if self.active_tab >= self.tabs.len() {
                        self.active_tab = self.tabs.len() - 1;
                    }
                }
                Task::none()
            }
            Message::SwitchTab(index) => {
                if index < self.tabs.len() {
                    self.active_tab = index;
                }
                Task::none()
            }
            Message::CloseError => {
                self.error_message = None;
                Task::none()
            }
            Message::OpenLatexExport => {
                let code = tikz_export::export_to_tikz(
                    &self.get_active_tab().states,
                    &self.get_active_tab().transitions,
                    self.get_active_tab().initial_state,
                    &self.get_active_tab().final_states,
                );
                self.latex_export_code = Some(code);
                self.latex_export_dialog_open = true;
                Task::none()
            }
            Message::CloseLatexExport => {
                self.latex_export_dialog_open = false;
                self.latex_export_code = None;
                Task::none()
            }
            Message::CopyLatexExport => {
                if let Some(code) = &self.latex_export_code {
                    return iced::clipboard::write(code.clone()).map(|_msg: ()| Message::CopyLatexExport);
                }
                Task::none()
            }
        }
    }

    fn set_initial_state(&mut self, state_id: usize) {
        if self.get_active_tab_mut().initial_state == Some(state_id) {
            self.get_active_tab_mut().initial_state = None;
        } else {
            self.get_active_tab_mut().initial_state = Some(state_id);
        }
        self.get_active_tab_mut().state_machine.request_redraw();
    }

    fn toggle_final_state(&mut self, state_id: usize) {
        if self.get_active_tab_mut().final_states.contains(&state_id) {
            self.get_active_tab_mut().final_states.remove(&state_id);
        } else {
            self.get_active_tab_mut().final_states.insert(state_id);
        }
        self.get_active_tab_mut().state_machine.request_redraw();
    }

    fn sync_gui_to_finite_automata(&mut self) {
        let active_tab = self.get_active_tab_mut();
        active_tab.machine.clear();

        for state_node in &active_tab.states {
            active_tab.machine.add_state_with_id_label(state_node.id as u64, state_node.label);
        }

        for &state_id in &active_tab.final_states {
            active_tab.machine.make_final(state_id as u64);
        }

        if let Some(initial_id) = active_tab.initial_state {
            active_tab.machine.make_initial(initial_id as u64);
        }

        for gui_transition in &active_tab.transitions {
            active_tab.machine.add_transition(
                gui_transition.from_state_id as u64,
                gui_transition.to_state_id as u64,
                gui_transition.label.to_string()
            );
        }
    }

    fn load_finite_automata_to_gui(&mut self) {
        let active_tab = self.get_active_tab_mut();
        
        active_tab.states.clear();
        active_tab.transitions.clear();
        active_tab.state_id_to_index.clear();
        active_tab.initial_state = None;
        active_tab.final_states.clear();

        let mut max_id_after_load = 0;
        for (id, state) in active_tab.machine.get_states_by_id_ref() {
            let state_node = state_machine::StateNode::new(
                *id as usize,
                iced::Point::new(100.0, 100.0), 
                30.0, 
                Box::leak(state.name.clone().into_boxed_str())
            );
            let index = active_tab.states.len();
            active_tab.states.push(state_node);
            active_tab.state_id_to_index.insert(*id as usize, index);
            max_id_after_load = max_id_after_load.max(*id as usize);
        }

        active_tab.state_machine.next_id = max_id_after_load + 1;

        for (from_id, state) in active_tab.machine.get_states_by_id_ref() {
            for (to_id, inputs) in state.iter_by_transition() {
                let from_state = active_tab.states.iter()
                    .find(|s| s.id == *from_id as usize)
                    .unwrap();
                let to_state = active_tab.states.iter()
                    .find(|s| s.id == *to_id as usize)
                    .unwrap();

                let label = inputs.iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ");

                let gui_transition = state_machine::Transition {
                    from_state_id: *from_id as usize,
                    to_state_id: *to_id as usize,
                    from_point: from_state.position,
                    to_point: to_state.position,
                    label: Box::leak(label.into_boxed_str())
                };
                active_tab.transitions.push(gui_transition);
            }
        }

        if let Some(initial_id) = active_tab.machine.get_initial_state_id() {
            if let Some(state) = active_tab.states.iter()
                .find(|s| s.id == *initial_id as usize) {
                active_tab.initial_state = Some(state.id);
            }
        }

        for final_id in active_tab.machine.get_final_states() {
            if let Some(state) = active_tab.states.iter()
                .find(|s| s.id == *final_id as usize) {
                active_tab.final_states.insert(state.id);
            }
        }

        if active_tab.initial_state.is_some() {
            Self::apply_tree_layout_to_tab(active_tab);
        } else {
            Self::apply_grid_layout_to_tab(active_tab);
        }

        active_tab.state_machine.request_redraw();
    }

    fn apply_tree_layout_to_tab(active_tab: &mut Tab) {
        use std::collections::{HashMap, HashSet};

        let mut children_map: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut parent_map: HashMap<usize, usize> = HashMap::new();
        let mut all_ids: HashSet<usize> = HashSet::new();
        for state in &active_tab.states {
            all_ids.insert(state.id);
        }
        for transition in &active_tab.transitions {
            if !parent_map.contains_key(&transition.to_state_id) {
                children_map.entry(transition.from_state_id).or_default().push(transition.to_state_id);
                parent_map.insert(transition.to_state_id, transition.from_state_id);
            }
        }

        let root_id = match active_tab.initial_state {
            Some(id) => id,
            None => return,
        };

        let mut x_counter = 0.0;
        let x_spacing = 90.0;
        let y_spacing = 120.0;
        let start_x = 100.0;
        let start_y = 100.0;

        fn assign_positions(
            node_id: usize,
            depth: usize,
            children_map: &HashMap<usize, Vec<usize>>,
            state_map: &mut HashMap<usize, &mut state_machine::StateNode>,
            x_counter: &mut f32,
            x_spacing: f32,
            y_spacing: f32,
            start_x: f32,
            start_y: f32,
        ) -> f32 {
            let children = children_map.get(&node_id);
            let mut x = 0.0;
            if let Some(children) = children {
                let mut child_xs = Vec::new();
                for &child_id in children {
                    let cx = assign_positions(child_id, depth + 1, children_map, state_map, x_counter, x_spacing, y_spacing, start_x, start_y);
                    child_xs.push(cx);
                }
                if !child_xs.is_empty() {
                    x = (child_xs[0] + child_xs[child_xs.len() - 1]) / 2.0;
                } else {
                    x = *x_counter;
                    *x_counter += x_spacing;
                }
            } else {
                x = *x_counter;
                *x_counter += x_spacing;
            }
            if let Some(state) = state_map.get_mut(&node_id) {
                state.position = iced::Point::new(start_x + x, start_y + (depth as f32) * y_spacing);
            }
            x
        }

        let mut state_map: HashMap<usize, &mut state_machine::StateNode> =
            active_tab.states.iter_mut().map(|s| (s.id, s)).collect();
        assign_positions(
            root_id,
            0,
            &children_map,
            &mut state_map,
            &mut x_counter,
            x_spacing,
            y_spacing,
            start_x,
            start_y,
        );

        let placed: HashSet<usize> = state_map.keys().copied().collect();
        let unreachable: Vec<usize> = all_ids.difference(&placed).copied().collect();
        let unreachable_y = start_y + 4.0 * y_spacing;
        for (i, id) in unreachable.iter().enumerate() {
            if let Some(state) = state_map.get_mut(id) {
                state.position = iced::Point::new(start_x + (i as f32) * x_spacing, unreachable_y);
            }
        }

        for transition in &mut active_tab.transitions {
            if let (Some(from_state), Some(to_state)) = (
                active_tab.states.iter().find(|s| s.id == transition.from_state_id),
                active_tab.states.iter().find(|s| s.id == transition.to_state_id)
            ) {
                transition.from_point = from_state.position;
                transition.to_point = to_state.position;
            }
        }
    }

    fn apply_grid_layout_to_tab(active_tab: &mut Tab) {
        let states = &mut active_tab.states;
        
        if states.is_empty() {
            return;
        }

        let grid_size = (states.len() as f32).sqrt().ceil() as usize;
        let spacing = 150.0; 
        let start_x = 100.0;
        let start_y = 100.0;

        for (i, state) in states.iter_mut().enumerate() {
            let row = i / grid_size;
            let col = i % grid_size;
            state.position = iced::Point::new(
                start_x + (col as f32 * spacing),
                start_y + (row as f32 * spacing)
            );
        }

        for transition in &mut active_tab.transitions {
            if let (Some(from_state), Some(to_state)) = (
                active_tab.states.iter().find(|s| s.id == transition.from_state_id),
                active_tab.states.iter().find(|s| s.id == transition.to_state_id)
            ) {
                transition.from_point = from_state.position;
                transition.to_point = to_state.position;
            }
        }
    }

    fn create_menu_bar(&self) -> Element<Message> {
        let abstract_machine_button = button(text("Abstract Machine"))
            .style(|_theme: &iced::Theme, _status| {
                button::Style {
                    background: Some(iced::Color::from_rgba(0.176, 0.172, 0.176, 1.0).into()), 
                    text_color: iced::Color::from_rgba(0.6, 0.6, 0.6, 1.0), 
                    border: iced::Border::default(),
                    ..Default::default()
                }
            })
            .padding([4, 12]);

        let operations_button = button(text("Operation"))
            .on_press(Message::ToggleOperationsMenu)
            .style(|_theme: &iced::Theme, status| {
                let background_color = iced::Color::from_rgba(0.176, 0.172, 0.176, 1.0); 
                let hover_color = iced::Color::from_rgba(0.25, 0.24, 0.25, 1.0); 
                let text_color = iced::Color::WHITE;
                
                match status {
                    button::Status::Hovered => button::Style {
                        background: Some(hover_color.into()),
                        text_color,
                        border: iced::Border::default(),
                        ..Default::default()
                    },
                    _ => button::Style {
                        background: Some(background_color.into()),
                        text_color,
                        border: iced::Border::default(),
                        ..Default::default()
                    }
                }
            })
            .padding([4, 12]);

        let latex_button = button(text("LaTeX"))
            .on_press(Message::OpenLatexExport)
            .style(|_theme: &iced::Theme, status| {
                let background_color = iced::Color::from_rgba(0.176, 0.172, 0.176, 1.0);
                let hover_color = iced::Color::from_rgba(0.25, 0.24, 0.25, 1.0);
                let text_color = iced::Color::WHITE;
                match status {
                    button::Status::Hovered => button::Style {
                        background: Some(hover_color.into()),
                        text_color,
                        border: iced::Border::default(),
                        ..Default::default()
                    },
                    _ => button::Style {
                        background: Some(background_color.into()),
                        text_color,
                        border: iced::Border::default(),
                        ..Default::default()
                    }
                }
            })
            .padding([4, 12]);

        let menu_bar = container(
            row![
                abstract_machine_button,
                operations_button,
                latex_button,
                horizontal_space(),
            ]
            .spacing(4)
            .align_y(Alignment::Center)
        )
        .style(|_theme: &iced::Theme| {
            container::Style {
                background: Some(iced::Color::from_rgba(0.176, 0.172, 0.176, 1.0).into()), 
                border: iced::Border {
                    color: iced::Color::from_rgba(0.3, 0.3, 0.3, 1.0), 
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .padding([4, 8])
        .width(Length::Fill);

        menu_bar.into()
    }

    fn create_operations_menu(&self) -> Element<Message> {
        let menu_background_color = iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0); 
        let menu_button_hover_color = iced::Color::from_rgba(0.0, 0.5, 1.0, 1.0); 
        let text_color = iced::Color::WHITE;

        let menu_items = column![
            button(text("Check Input"))
                .on_press(Message::CheckInput)
                .width(Length::Fill)
                .style(move |_theme: &iced::Theme, status| {
                    match status {
                        button::Status::Hovered => button::Style {
                            background: Some(menu_button_hover_color.into()),
                            text_color,
                            border: iced::Border::default(),
                            ..Default::default()
                        },
                        _ => button::Style {
                            background: Some(menu_background_color.into()),
                            text_color,
                            border: iced::Border::default(),
                            ..Default::default()
                        }
                    }
                })
                .padding([4, 8]),
            button(text("DFA to NFA"))
                .on_press(Message::DfaToNfa)
                .width(Length::Fill)
                .style(move |_theme: &iced::Theme, status| {
                    match status {
                        button::Status::Hovered => button::Style {
                            background: Some(menu_button_hover_color.into()),
                            text_color,
                            border: iced::Border::default(),
                            ..Default::default()
                        },
                        _ => button::Style {
                            background: Some(menu_background_color.into()),
                            text_color,
                            border: iced::Border::default(),
                            ..Default::default()
                        }
                    }
                })
                .padding([4, 8]),
            button(text("Minimize"))
                .on_press(Message::Minimize)
                .width(Length::Fill)
                .style(move |_theme: &iced::Theme, status| {
                    match status {
                        button::Status::Hovered => button::Style {
                            background: Some(menu_button_hover_color.into()),
                            text_color,
                            border: iced::Border::default(),
                            ..Default::default()
                        },
                        _ => button::Style {
                            background: Some(menu_background_color.into()),
                            text_color,
                            border: iced::Border::default(),
                            ..Default::default()
                        }
                    }
                })
                .padding([4, 8]),
        ]
        .spacing(2)
        .width(120);

        container(menu_items)
            .style(move |_theme: &iced::Theme| {
                container::Style {
                    background: Some(menu_background_color.into()),
                    border: iced::Border {
                        color: iced::Color::from_rgba(0.4, 0.4, 0.4, 1.0),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
            .padding(4)
            .into()
    }

    fn create_check_input_dialog(&self) -> Element<Message> {
        let menu_background_color = iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0);
        let text_color = iced::Color::WHITE;
        let border_color = iced::Color::from_rgba(0.4, 0.4, 0.4, 1.0);

        let dialog = container(
            container(
                iced::widget::column![
                    iced::widget::text("Input String:")
                        .size(17)
                        .color(text_color),
                    iced::widget::text_input("Enter input string...", &self.get_active_tab().check_input_text)
                        .on_input(Message::CheckInputTextChanged)
                        .on_submit(Message::SubmitCheckInput)
                        .width(200)
                        .style(|_theme: &iced::Theme, _status| {
                            iced::widget::text_input::Style {
                                background: iced::Background::Color(iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0)),
                                border: iced::Border {
                                    color: iced::Color::from_rgba(0.0, 0.5, 1.0, 1.0),
                                    width: 2.0,
                                    radius: 4.0.into(),
                                },
                                icon: iced::Color::WHITE,
                                placeholder: iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0),
                                value: iced::Color::WHITE,
                                selection: iced::Color::from_rgba(0.0, 0.5, 1.0, 0.3),
                            }
                        }),
                    row![
                        button("Accept")
                            .on_press(Message::SubmitCheckInput)
                            .padding([4, 8]),
                        button("Cancel")
                            .on_press(Message::CancelCheckInput)
                            .padding([4, 8])
                    ]
                    .spacing(8)
                ]
                .spacing(8)
                .padding(12)
                .width(250)
            )
            .style(move |_theme: &iced::Theme| {
                container::Style {
                    background: Some(menu_background_color.into()),
                    border: iced::Border {
                        color: border_color,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
        )
        .center(iced::Length::Fill)
        .style(|_theme: &iced::Theme| {
            container::Style {
                background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3).into()),
                ..Default::default()
            }
        });

        dialog.into()
    }

    fn create_check_result_popup(&self) -> Element<Message> {
        let menu_background_color = iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0);
        let text_color = iced::Color::WHITE;
        let border_color = iced::Color::from_rgba(0.4, 0.4, 0.4, 1.0);

        let result_text = if let Some(result) = self.get_active_tab().check_input_result {
            if result {
                "Input is accepted by the automaton :)"
            } else {
                "Input is rejected by the automaton :("
            }
        } else {
            "No result available"
        };

        let dialog = container(
            container(
                iced::widget::column![
                    iced::widget::text(result_text)
                        .size(17)
                        .color(text_color),
                    button("Close")
                        .on_press(Message::CloseCheckResultPopup)
                        .padding([4, 8])
                ]
                .spacing(8)
                .padding(12)
                .width(250)
            )
            .style(move |_theme: &iced::Theme| {
                container::Style {
                    background: Some(menu_background_color.into()),
                    border: iced::Border {
                        color: border_color,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
        )
        .center(iced::Length::Fill)
        .style(|_theme: &iced::Theme| {
            container::Style {
                background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3).into()),
                ..Default::default()
            }
        });

        dialog.into()
    }

    fn create_tab_bar(&self) -> Element<Message> {
        let tab_bar_background = iced::Color::from_rgba(0.176, 0.172, 0.176, 1.0);
        let active_tab_background = iced::Color::from_rgba(0.25, 0.24, 0.25, 1.0);
        let text_color = iced::Color::WHITE;
        let hover_color = iced::Color::from_rgba(0.3, 0.29, 0.3, 1.0);

        let mut tab_buttons = row![].spacing(2);

        for (index, tab) in self.tabs.iter().enumerate() {
            let is_active = index == self.active_tab;
            let tab_button = button(
                row![
                    text(&tab.name)
                        .size(14)
                        .color(text_color),
                    if self.tabs.len() > 1 {
                        button("×")
                            .on_press(Message::RemoveTab(index))
                            .style(move |_theme: &iced::Theme, status| {
                                match status {
                                    button::Status::Hovered => button::Style {
                                        background: Some(iced::Color::from_rgba(0.8, 0.2, 0.2, 1.0).into()),
                                        text_color: iced::Color::WHITE,
                                        border: iced::Border::default(),
                                        ..Default::default()
                                    },
                                    _ => button::Style {
                                        background: Some(iced::Color::TRANSPARENT.into()),
                                        text_color: iced::Color::WHITE,
                                        border: iced::Border::default(),
                                        ..Default::default()
                                    }
                                }
                            })
                            .padding([2, 6])
                    } else {
                        button("")
                            .style(|_theme: &iced::Theme, _status| {
                                button::Style {
                                    background: Some(iced::Color::TRANSPARENT.into()),
                                    text_color: iced::Color::TRANSPARENT,
                                    border: iced::Border::default(),
                                    ..Default::default()
                                }
                            })
                            .padding([2, 6])
                    }
                ]
                .spacing(8)
                .align_y(Alignment::Center)
            )
            .on_press(Message::SwitchTab(index))
            .style(move |_theme: &iced::Theme, status| {
                let background = match (is_active, status) {
                    (true, _) => active_tab_background,
                    (false, button::Status::Hovered) => hover_color,
                    _ => tab_bar_background,
                };
                button::Style {
                    background: Some(background.into()),
                    text_color,
                    border: iced::Border {
                        color: iced::Color::from_rgba(0.3, 0.3, 0.3, 1.0),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            })
            .padding([4, 12]);

            tab_buttons = tab_buttons.push(tab_button);
        }

        let new_tab_button = button(
            text("+")
                .size(16)
                .color(text_color)
        )
        .on_press(Message::AddTab)
        .style(move |_theme: &iced::Theme, status| {
            let background = match status {
                button::Status::Hovered => hover_color,
                _ => tab_bar_background,
            };
            button::Style {
                background: Some(background.into()),
                text_color,
                border: iced::Border {
                    color: iced::Color::from_rgba(0.3, 0.3, 0.3, 1.0),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .padding([4, 12]);

        tab_buttons = tab_buttons.push(new_tab_button);

        container(tab_buttons)
            .style(move |_theme: &iced::Theme| {
                container::Style {
                    background: Some(tab_bar_background.into()),
                    border: iced::Border {
                        color: iced::Color::from_rgba(0.3, 0.3, 0.3, 1.0),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            })
            .padding([4, 8])
            .width(Length::Fill)
            .into()
    }

    fn create_error_popup(&self) -> Element<Message> {
        if let Some(error_message) = &self.error_message {
            let menu_background_color = iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0);
            let text_color = iced::Color::WHITE;
            let border_color = iced::Color::from_rgba(0.4, 0.4, 0.4, 1.0);

            let dialog = container(
                container(
                    iced::widget::column![
                        iced::widget::text(error_message)
                            .size(17)
                            .color(text_color),
                        button("Close")
                            .on_press(Message::CloseError)
                            .padding([4, 8])
                    ]
                    .spacing(8)
                    .padding(12)
                    .width(250)
                )
                .style(move |_theme: &iced::Theme| {
                    container::Style {
                        background: Some(menu_background_color.into()),
                        border: iced::Border {
                            color: border_color,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                })
            )
            .center(iced::Length::Fill)
            .style(|_theme: &iced::Theme| {
                container::Style {
                    background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3).into()),
                    ..Default::default()
                }
            });

            dialog.into()
        } else {
            container(horizontal_space()).into()
        }
    }

    fn create_edit_dialog(&self) -> Element<Message> {
        let menu_background_color = iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0); 
        let text_color = iced::Color::WHITE;
        let border_color = iced::Color::from_rgba(0.4, 0.4, 0.4, 1.0);

        let edit_dialog = container(
            container(
                iced::widget::column![
                    iced::widget::text(if self.get_active_tab().editing_state.is_some() { "Edit State:" } else { "Edit Transition:" })
                        .size(17)
                        .color(text_color),
                    iced::widget::text_input("Enter label...", &self.get_active_tab().edit_text)
                        .on_input(Message::EditTextChanged)
                        .on_submit(Message::FinishEditing)
                        .width(150)
                        .style(|_theme: &iced::Theme, _status| {
                            iced::widget::text_input::Style {
                                background: iced::Background::Color(iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0)),
                                border: iced::Border {
                                    color: iced::Color::from_rgba(0.0, 0.5, 1.0, 1.0),
                                    width: 2.0,
                                    radius: 4.0.into(), 
                                },
                                icon: iced::Color::WHITE,
                                placeholder: iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0),
                                value: iced::Color::WHITE,
                                selection: iced::Color::from_rgba(0.0, 0.5, 1.0, 0.3),
                            }
                        }),
                    row![
                        button("Save")
                            .on_press(Message::FinishEditing)
                            .padding([4, 8]),
                        button("Cancel")
                            .on_press(Message::CancelEditing)
                            .padding([4, 8])
                    ]
                    .spacing(8)
                ]
                .spacing(8)
                .padding(12)
                .width(200)
            )
            .style(move |_theme: &iced::Theme| {
                container::Style {
                    background: Some(menu_background_color.into()),
                    border: iced::Border {
                        color: border_color,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
        )
        .center(iced::Length::Fill)
        .style(|_theme: &iced::Theme| {
            container::Style {
                background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3).into()),
                ..Default::default()
            }
        });

        edit_dialog.into()
    }

    fn create_latex_export_dialog(&self) -> Element<Message> {
        let menu_background_color = iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0);
        let text_color = iced::Color::WHITE;
        let border_color = iced::Color::from_rgba(0.4, 0.4, 0.4, 1.0);
        let code = self.latex_export_code.as_deref().unwrap_or("");
        let dialog = container(
            container(
                iced::widget::column![
                    iced::widget::text("LaTeX (TikZ) code for this automaton:")
                        .size(17)
                        .color(text_color),
                    iced::widget::text_input("", code)
                        .width(400)
                        .on_input(|_| Message::OpenLatexExport) 
                        .style(move |_theme: &iced::Theme, _status| {
                            iced::widget::text_input::Style {
                                background: iced::Background::Color(menu_background_color),
                                border: iced::Border {
                                    color: border_color,
                                    width: 1.0,
                                    radius: 4.0.into(),
                                },
                                icon: iced::Color::WHITE,
                                placeholder: iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0),
                                value: iced::Color::WHITE,
                                selection: iced::Color::from_rgba(0.0, 0.5, 1.0, 0.3),
                            }
                        }),
                    row![
                        button("Copy")
                            .on_press(Message::CopyLatexExport)
                            .padding([4, 8]),
                        button("Close")
                            .on_press(Message::CloseLatexExport)
                            .padding([4, 8])
                    ]
                    .spacing(8)
                ]
                .spacing(8)
                .padding(12)
                .width(500)
            )
            .style(move |_theme: &iced::Theme| {
                container::Style {
                    background: Some(menu_background_color.into()),
                    border: iced::Border {
                        color: border_color,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
        )
        .center(iced::Length::Fill)
        .style(|_theme: &iced::Theme| {
            container::Style {
                background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3).into()),
                ..Default::default()
            }
        });
        dialog.into()
    }

    pub fn view(&self) -> Element<Message> {
        let menu_bar = self.create_menu_bar();
        let tab_bar = self.create_tab_bar();

        let main_content = container(hover(
            self.get_active_tab().state_machine.view(
                &self.get_active_tab().states,
                &self.get_active_tab().transitions,
                self.get_active_tab().initial_state,
                &self.get_active_tab().final_states
            ).map(Message::Canvas),
            if self.get_active_tab().states.is_empty() && self.get_active_tab().transitions.is_empty() {
                container(horizontal_space())
            } else {
                container(row![
                    horizontal_space(),
                    button("Clear")
                        .style(button::danger)
                        .on_press(Message::Clear)
                ]
                .align_y(Alignment::Center)
                )
                .padding(10)
            },
        ))
        .style(|_theme: &iced::Theme| {
            container::Style {
                background: None,
                border: iced::Border::default(),
                ..Default::default()
            }
        })
        .padding(0);

        let content_with_menu = column![
            menu_bar,
            tab_bar,
            main_content
        ]
        .spacing(0);
        
        let mut final_content: Element<Message> = if self.get_active_tab().operations_menu_open {
            let operations_menu = container(
                self.create_operations_menu()
            );

            stack![
                content_with_menu,
                container(
                    container(operations_menu)
                        .style(|_theme: &iced::Theme| {
                            container::Style {
                                background: Some(iced::Color::TRANSPARENT.into()),
                                ..Default::default()
                            }
                        })
                )
                .style(|_theme: &iced::Theme| {
                    container::Style {
                        background: Some(iced::Color::TRANSPARENT.into()),
                        ..Default::default()
                    }
                })
                .padding(iced::Padding {
                    top: 40.0,  
                    left: 141.0, 
                    right: 0.0,
                    bottom: 0.0,
                })
            ].into()
        } else {
            content_with_menu.into()
        };

        if self.get_active_tab().editing_state.is_some() || self.get_active_tab().editing_transition.is_some() {
            let edit_dialog = self.create_edit_dialog();
            final_content = iced::widget::stack![final_content, edit_dialog].into();
        }

        if self.get_active_tab().check_input_dialog_open {
            let check_input_dialog = self.create_check_input_dialog();
            final_content = iced::widget::stack![final_content, check_input_dialog].into();
        }

        if self.get_active_tab().check_result_popup_open {
            let check_result_popup = self.create_check_result_popup();
            final_content = iced::widget::stack![final_content, check_result_popup].into();
        }

        if self.error_message.is_some() {
            let error_popup = self.create_error_popup();
            final_content = iced::widget::stack![final_content, error_popup].into();
        }

        if self.latex_export_dialog_open {
            let latex_dialog = self.create_latex_export_dialog();
            final_content = iced::widget::stack![final_content, latex_dialog].into();
        }

        container(final_content)
            .style(|_theme: &iced::Theme| {
                container::Style {
                    background: Some(iced::Color::from_rgb(0.1, 0.1, 0.1).into()), 
                    border: iced::Border::default(),
                    ..Default::default()
                }
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(0)
            .into()
    }
}

