use iced::keyboard;
use iced::widget::{button, container, horizontal_space, hover, row, text_input, text, column, stack};
use iced::{Element, Alignment, Point, Event, Subscription, Task, Length};
use std::collections::HashMap;

use crate::state_machine;

use moca_data::finite_automata::FiniteAutomata;
use moca_data::state_machine::StateMachine;

#[derive(Debug, Clone)]
pub enum Message {
    Canvas(state_machine::CanvasMessage), 
    KeyPressed(keyboard::Key),
    KeyReleased(keyboard::Key),
    Clear,
    SyncToFiniteAutomata,
    LoadFromFiniteAutomata,
    EditTextChanged(String),
    FinishEditing,
    CancelEditing,
    ToggleOperationsMenu,
    CloseMenus,
    CheckInput,
    DfaToNfa,
    Minimize,
    OpenCheckInputDialog,
    CheckInputTextChanged(String),
    SubmitCheckInput,
    CancelCheckInput,
    CloseCheckResultPopup,
}

#[derive(Default)]
pub struct App {
    state_machine: state_machine::State,
    transitions: Vec<state_machine::Transition>,
    states: Vec<state_machine::StateNode>,
    // Map from state ID to index in the states vector for quick lookup
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
    check_result: Option<bool>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
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

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Canvas(canvas_msg) => {
                match canvas_msg {
                    state_machine::CanvasMessage::AddState(mut state) => {
                        // Assign a unique ID to the state
                        let id = self.state_machine.get_next_id();
                        state.id = id;
                        
                        // Add to states vector and update the ID-to-index mapping
                        let index = self.states.len();
                        self.states.push(state);
                        self.state_id_to_index.insert(id, index);
                        
                        self.state_machine.request_redraw();
                    }
                    state_machine::CanvasMessage::AddTransition(transition) => {
                        self.transitions.push(transition);
                        self.state_machine.request_redraw();
                    }
                    state_machine::CanvasMessage::MoveState { state_id, new_position } => {
                        if let Some(&index) = self.state_id_to_index.get(&state_id) {
                            if let Some(state) = self.states.get_mut(index) {
                                state.position = new_position;
                                
                                // Update transition points
                                for transition in &mut self.transitions {
                                    if transition.from_state_id == state_id {
                                        transition.from_point = new_position;
                                    }
                                    if transition.to_state_id == state_id {
                                        transition.to_point = new_position;
                                    }
                                }
                                
                                self.state_machine.request_redraw();
                            }
                        }
                    }
                    state_machine::CanvasMessage::StateClicked(state_id) => {
                        if self.state_machine.is_shift_pressed() {
                            self.toggle_final_state(state_id);
                        } else if self.state_machine.is_alt_pressed() {
                            self.set_initial_state(state_id);
                        }
                    }
                    state_machine::CanvasMessage::StateDoubleClicked(state_id) => {
                        if let Some(&index) = self.state_id_to_index.get(&state_id) {
                            if let Some(state) = self.states.get(index) {
                                self.editing_state = Some(state_id);
                                self.edit_text = state.label.to_string();
                                self.editing_transition = None;
                            }
                        }
                    }
                    state_machine::CanvasMessage::TransitionDoubleClicked(transition_index) => {
                        if let Some(transition) = self.transitions.get(transition_index) {
                            self.editing_transition = Some(transition_index);
                            self.edit_text = transition.label.to_string();
                            self.editing_state = None;
                        }
                    }
                }
                // Close menus when interacting with canvas
                self.operations_menu_open = false;
            }
            Message::EditTextChanged(text) => {
                self.edit_text = text;
            }
            Message::FinishEditing => {
                if let Some(state_id) = self.editing_state {
                    if let Some(&index) = self.state_id_to_index.get(&state_id) {
                        if let Some(state) = self.states.get_mut(index) {
                            state.label = Box::leak(self.edit_text.clone().into_boxed_str());
                        }
                    }
                } else if let Some(transition_index) = self.editing_transition {
                    if let Some(transition) = self.transitions.get_mut(transition_index) {
                        transition.label = Box::leak(self.edit_text.clone().into_boxed_str());
                    }
                }
                self.editing_state = None;
                self.editing_transition = None;
                self.edit_text.clear();
                self.state_machine.request_redraw();
            }
            Message::CancelEditing => {
                self.editing_state = None;
                self.editing_transition = None;
                self.edit_text.clear();
            }
            Message::KeyPressed(key) => {
                match key {
                    keyboard::Key::Named(keyboard::key::Named::Control) => {
                        self.state_machine.set_ctrl_pressed(true);
                    }
                    keyboard::Key::Named(keyboard::key::Named::Shift) => {
                        self.state_machine.set_shift_pressed(true);
                    }
                    keyboard::Key::Named(keyboard::key::Named::Alt) => {
                        self.state_machine.set_alt_pressed(true);
                    }
                    keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        if self.editing_state.is_some() || self.editing_transition.is_some() {
                            self.update(Message::FinishEditing);
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::Escape) => {
                        if self.editing_state.is_some() || self.editing_transition.is_some() {
                            self.update(Message::CancelEditing);
                        } else {
                            self.update(Message::CloseMenus);
                        }
                    }
                    _ => {}
                }
            }
            Message::KeyReleased(key) => {
                match key {
                    keyboard::Key::Named(keyboard::key::Named::Control) => {
                        self.state_machine.set_ctrl_pressed(false);
                    }
                    keyboard::Key::Named(keyboard::key::Named::Shift) => {
                        self.state_machine.set_shift_pressed(false);
                    }
                    keyboard::Key::Named(keyboard::key::Named::Alt) => {
                        self.state_machine.set_alt_pressed(false);
                    }
                    _ => {}
                }
            }
            Message::Clear => {
                self.state_machine = state_machine::State::default();
                self.state_machine.reset_id_counter();
                self.transitions.clear();
                self.states.clear();
                self.state_id_to_index.clear();
                self.initial_state = None;
                self.final_states.clear();
                self.machine = FiniteAutomata::default();
                self.check_input_dialog_open = false;
                self.check_input_text.clear();
                self.check_result_popup_open = false;
                self.check_result = None;
            }
            Message::SyncToFiniteAutomata => {
                self.sync_gui_to_finite_automata();
            }
            Message::LoadFromFiniteAutomata => {
                self.load_finite_automata_to_gui();
            }
            Message::ToggleOperationsMenu => {
                self.operations_menu_open = !self.operations_menu_open;
            }
            Message::CloseMenus => {
                self.operations_menu_open = false;
            }
            Message::CheckInput => {
                self.update(Message::OpenCheckInputDialog);
                self.operations_menu_open = false;            
            }
            Message::OpenCheckInputDialog => {
                self.check_input_dialog_open = true;
                self.check_input_text.clear();
            }
            Message::CheckInputTextChanged(text) => {
                self.check_input_text = text;
            }
            Message::SubmitCheckInput => {
                let result = self.machine.check_input(&mut self.check_input_text);
                self.check_result = Some(result);
                self.check_input_dialog_open = false;
                self.check_result_popup_open = true;
                self.check_input_text.clear();
            }
            Message::CancelCheckInput => {
                self.check_input_dialog_open = false;
                self.check_input_text.clear();
            }
            Message::CloseCheckResultPopup => {
                self.check_result_popup_open = false;
                self.check_result = None;
            }
            Message::DfaToNfa => {
                // TODO: Implement DFA to NFA conversion
                println!("DFA to NFA clicked");
                self.operations_menu_open = false;
            }
            Message::Minimize => {
                // TODO: Implement minimize functionality
                println!("Minimize clicked");
                self.operations_menu_open = false;
            }

        }
        // I will move this function call
        self.sync_gui_to_finite_automata();
    }

    fn set_initial_state(&mut self, state_id: usize) {
        if self.initial_state == Some(state_id) {
            self.initial_state = None;
        } else {
            self.initial_state = Some(state_id);
        }
        self.state_machine.request_redraw();
    }

    fn toggle_final_state(&mut self, state_id: usize) {
        if self.final_states.contains(&state_id) {
            self.final_states.remove(&state_id);
        } else {
            self.final_states.insert(state_id);
        }
        self.state_machine.request_redraw();
    }

    fn sync_gui_to_finite_automata(&mut self) {
        self.machine.clear();
        // Create a map from state ID to state label for easy lookup
        let mut state_map: HashMap<usize, String> = HashMap::new();
        let mut states_list = Vec::new();
        
        for state_node in &self.states {
            let state_label = state_node.label.to_string();
            state_map.insert(state_node.id, state_label.clone());
            states_list.push(state_label);
        }
        
        for state_node in &self.states {
            let is_initial = self.initial_state == Some(state_node.id);
            let is_final = self.final_states.contains(&state_node.id);
            let id = state_node.id as u64;
            self.machine.add_state();
            self.machine.modify_name(id, state_node.label.to_string());
            if is_final {
                self.machine.make_final(id);
            }
            if is_initial {
                self.machine.make_initial(id);
            }
        }
        
        for gui_transition in &self.transitions {
            if let (Some(_), Some(_)) = (
                state_map.get(&gui_transition.from_state_id),
                state_map.get(&gui_transition.to_state_id)
            ) {
                self.machine.add_transition(gui_transition.from_state_id as u64, gui_transition.to_state_id as u64, gui_transition.label.to_string());
            }
        }
    }

    fn load_finite_automata_to_gui(&mut self) {
        self.states.clear();
        self.transitions.clear();
        self.state_id_to_index.clear();
        self.initial_state = None;
        self.final_states.clear();
        self.state_machine.reset_id_counter();
        
        // Convert FiniteAutomata to GUI representation
        let mut states_from_machine: Vec<_> = self.machine.get_states_by_id_ref().into_iter().collect();
        states_from_machine.sort_by(|x,y| x.0.cmp(&y.0));
        
        // Basic positioning algorithm, will change to something that looks better
        for (idx, state) in states_from_machine.iter() {
            let position = Point::new(
                100.0 + (**idx as f32 * 150.0) % 600.0,
                100.0 + ((**idx as f32 * 150.0) / 600.0).floor() * 100.0,
            );
            
            let id = self.state_machine.get_next_id();
            let state_node = state_machine::StateNode::new(
                id,
                position,
                30.0,
                Box::leak(state.name.clone().into_boxed_str())
            );
            
            let index = self.states.len();
            self.states.push(state_node);
            self.state_id_to_index.insert(id, index);
        }
        
        // Convert transitions
        for (state_id, state) in states_from_machine {
            for (transition_id, input) in state.iter_by_transition() {
                if let (Some(from_state), Some(to_state)) = (
                    self.states.iter().find(|s| s.id as u64 == *state_id),
                    self.states.iter().find(|s| s.id as u64 == *transition_id),
                ) {
                    // This joins the hashset as "word1, word2, word3, ...
                    let label: String = input
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>()
                        .join(", ");
                    let gui_transition = state_machine::Transition {
                        from_state_id: from_state.id,
                        to_state_id: to_state.id,
                        from_point: from_state.position,
                        to_point: to_state.position,
                        label: Box::leak(label.into_boxed_str()),
                    };
                    self.transitions.push(gui_transition);
                }
            }
        }
        
        // Set initial state
        if let Some(initial) = self.machine.get_initial_state_id() {
            if let Some(state) = self.states.iter().find(|s| s.id as u64 == *initial) {
                self.initial_state = Some(state.id);
            }
        }
        
        // Set final states
        for final_state in self.machine.get_final_states() {
            if let Some(state) = self.states.iter().find(|s| s.id as u64 == *final_state) {
                self.final_states.insert(state.id);
            }
        }
        
        self.state_machine.request_redraw();
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

        let menu_bar = container(
            row![
                abstract_machine_button,
                operations_button,
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
                    iced::widget::text_input("Enter input string...", &self.check_input_text)
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
        
        let result_text = match self.check_result {
            Some(true) => "Accepted!",
            Some(false) => "Not Accepted :(",
            None => "Error"
        };

        let popup = container(
            container(
                iced::widget::column![
                    row![
                        iced::widget::text(result_text)
                            .size(18)
                            .color(text_color),
                        horizontal_space(),
                        button("×")
                            .on_press(Message::CloseCheckResultPopup)
                            .style(|_theme: &iced::Theme, status| {
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
                            .padding([2, 6]),
                    ]
                    .align_y(Alignment::Center)
                ]
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

        popup.into()
    }

    pub fn view(&self) -> Element<Message> {
        let menu_bar = self.create_menu_bar();

        let main_content = container(hover(
            self.state_machine.view(&self.states, &self.transitions, self.initial_state, &self.final_states)
                .map(Message::Canvas),
            if self.states.is_empty() && self.transitions.is_empty() {
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
                background: Some(iced::Color::WHITE.into()),
                border: iced::Border {
                    color: iced::Color::from_rgba(0.3, 0.3, 0.3, 1.0),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .padding(20);

        let content_with_menu = column![
            menu_bar,
            main_content
        ]
        .spacing(0);
        
        let mut final_content = if self.operations_menu_open {
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
                    top: 32.0,  // Height of menu bar + small gap
                    left: 141.0, // Approximate position of Operations button
                    right: 0.0,
                    bottom: 0.0,
                })
            ].into()
        } else {
            content_with_menu.into()
        };

        if self.editing_state.is_some() || self.editing_transition.is_some() {
            let menu_background_color = iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0); 
            let text_color = iced::Color::WHITE;
            let border_color = iced::Color::from_rgba(0.4, 0.4, 0.4, 1.0);

            let edit_dialog = container(
                container(
                    iced::widget::column![
                        iced::widget::text(if self.editing_state.is_some() { "Edit State:" } else { "Edit Transition:" })
                            .size(17)
                            .color(text_color),
                        iced::widget::text_input("Enter label...", &self.edit_text)
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
            
            final_content = iced::widget::stack![final_content, edit_dialog].into();
        }

        if self.check_input_dialog_open {
            let check_input_dialog = self.create_check_input_dialog();
            final_content = iced::widget::stack![final_content, check_input_dialog].into();
        }

        if self.check_result_popup_open {
            let check_result_popup = self.create_check_result_popup();
            final_content = iced::widget::stack![final_content, check_result_popup].into();
        }

        final_content
    }
}
