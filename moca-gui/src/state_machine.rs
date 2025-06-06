use iced::{alignment, mouse};
use iced::widget::canvas::{
    self, Canvas, Event, Frame, Geometry, Path, Stroke, Text
};
use iced::{Element, Fill, Point, Rectangle, Renderer, Theme, Vector};
use std::collections::HashSet;

pub trait VectorExt {
    fn length(&self) -> f32;
    fn unit(&self) -> Self;
}

impl VectorExt for iced::Vector {
    fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn unit(&self) -> Self {
        let length = self.length();
        if length > 1e-6 {
            iced::Vector::new(self.x / length, self.y / length)
        } else {
            iced::Vector::new(0.0, 0.0)
        }
    }
}

#[derive(Debug, Clone)]
pub enum CanvasMessage {
    AddState(StateNode),
    AddTransition(Transition),
    MoveState { state_id: usize, new_position: Point },
    StateClicked(usize),
    StateDoubleClicked(usize),
    TransitionDoubleClicked(usize),
    TransitionClicked(usize),
}

pub struct State {
    cache: canvas::Cache,
    ctrl_pressed: bool,
    shift_pressed: bool,
    alt_pressed: bool,
    deletion_mode: bool,
    pub next_id: usize, // Make next_id public for external modification
}

impl Default for State {
    fn default() -> Self {
        Self {
            cache: canvas::Cache::default(),
            ctrl_pressed: false,
            shift_pressed: false,
            alt_pressed: false,
            deletion_mode: false,
            next_id: 0,
        }
    }
}

impl State {
    pub fn view<'a>(
        &'a self,
        states: &'a [StateNode],
        transitions: &'a [Transition],
        initial_state: Option<usize>,
        final_states: &'a HashSet<usize>
    ) -> Element<'a, CanvasMessage> {
        Canvas::new(StateMachine {
            state: self,
            states,
            transitions,
            initial_state,
            final_states,
        })
        .width(Fill)
        .height(Fill)
        .into()
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear();
    }

    pub fn set_ctrl_pressed(&mut self, pressed: bool) {
        self.ctrl_pressed = pressed;
    }

    pub fn set_shift_pressed(&mut self, pressed: bool) {
        self.shift_pressed = pressed;
    }

    pub fn set_alt_pressed(&mut self, pressed: bool) {
        self.alt_pressed = pressed;
    }

    pub fn is_ctrl_pressed(&self) -> bool {
        self.ctrl_pressed
    }

    pub fn is_shift_pressed(&self) -> bool {
        self.shift_pressed
    }

    pub fn is_alt_pressed(&self) -> bool {
        self.alt_pressed
    }

    pub fn set_deletion_mode(&mut self, enabled: bool) {
        self.deletion_mode = enabled;
        self.cache.clear(); 
    }

    pub fn is_deletion_mode(&self) -> bool {
        self.deletion_mode
    }

    // This method is now less about "getting" and more about providing the current counter value
    // The actual increment happens in gui.rs after a state is formally added.
    pub fn get_current_next_id(&self) -> usize {
        self.next_id
    }

    pub fn reset_id_counter(&mut self) {
        self.next_id = 0;
    }

    pub fn check_input(&self, _input: &str) -> bool {
        // For now, return false as a placeholder
        // This will be implemented properly when we integrate with the finite automata logic
        false
    }
}

struct StateMachine<'a> {
    state: &'a State,
    states: &'a [StateNode],
    transitions: &'a [Transition],
    initial_state: Option<usize>,
    final_states: &'a HashSet<usize>,
}

impl StateMachine<'_> {
    fn find_transition_at_point(&self, point: Point) -> Option<usize> {
        for (index, transition) in self.transitions.iter().enumerate() {
            if let (Some(from_state), Some(to_state)) = (
                self.states.iter().find(|s| s.id == transition.from_state_id),
                self.states.iter().find(|s| s.id == transition.to_state_id)
            ) {
                // Check if point is near the transition label
                let label_pos = self.calculate_transition_label_position(transition, from_state, to_state);
                let distance = (point - label_pos).length();

                // Consider a hit if within 20 pixels of the label center
                if distance <= 20.0 {
                    return Some(index);
                }
            }
        }
        None
    }

    fn calculate_transition_label_position(&self, transition: &Transition, from_state: &StateNode, to_state: &StateNode) -> Point {
        let has_reverse = self.transitions.iter().any(|other_trans|
            other_trans.from_state_id == transition.to_state_id &&
            other_trans.to_state_id == transition.from_state_id
        );

        if has_reverse {
            let center_to_center = to_state.position - from_state.position;
            let distance = center_to_center.length();
            let (node_a_pos, node_b_pos) = if transition.from_state_id < transition.to_state_id {
                (from_state.position, to_state.position)
            } else {
                (to_state.position, from_state.position)
            };
            let consistent_direction = node_b_pos - node_a_pos;
            let consistent_perpendicular = Vector::new(-consistent_direction.y, consistent_direction.x).unit();
            let curve_side_multiplier = if transition.from_state_id < transition.to_state_id { 1.0 } else { -1.0 };
            let curve_offset = distance * 0.4;
            let midpoint = Point::new(
                (from_state.position.x + to_state.position.x) / 2.0,
                (from_state.position.y + to_state.position.y) / 2.0,
            );
            let control_point = midpoint + consistent_perpendicular * curve_offset * curve_side_multiplier;
            let label_position = self.calculate_curve_midpoint(from_state.position, control_point, to_state.position);
            let label_offset = consistent_perpendicular * (25.0 * curve_side_multiplier);
            label_position + label_offset
        } else {
            let direction = to_state.position - from_state.position;
            let midpoint = Point::new(
                (from_state.position.x + to_state.position.x) / 2.0,
                (from_state.position.y + to_state.position.y) / 2.0,
            );
            let perpendicular_vec = Vector::new(-direction.y, direction.x).unit() * 15.0;
            midpoint + perpendicular_vec
        }
    }

    fn calculate_curve_midpoint(&self, start: Point, control: Point, end: Point) -> Point {
        let t = 0.5;
        let one_minus_t = 1.0 - t;
        Point::new(
            one_minus_t * one_minus_t * start.x + 2.0 * one_minus_t * t * control.x + t * t * end.x,
            one_minus_t * one_minus_t * start.y + 2.0 * one_minus_t * t * control.y + t * t * end.y,
        )
    }
}

impl canvas::Program<CanvasMessage> for StateMachine<'_> {
    type State = Option<PendingTransition>;

    fn update(
        &self, 
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        let cursor_position = cursor.position_in(bounds);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(cursor_pos) = cursor_position {
                    let now = std::time::Instant::now();
                    let double_click_threshold = std::time::Duration::from_millis(300);
                    
                    let clicked_node = self.states.iter().find(|node| {
                        (cursor_pos - node.position).length() <= node.radius
                    });
                    let clicked_transition_index = self.find_transition_at_point(cursor_pos);

                    let (last_click_time, last_clicked_state, last_clicked_transition) = match state {
                        Some(PendingTransition::ClickTracking { 
                            last_click_time, 
                            last_clicked_state, 
                            last_clicked_transition 
                        }) => (*last_click_time, *last_clicked_state, *last_clicked_transition),
                        _ => (std::time::Instant::now() - double_click_threshold - std::time::Duration::from_millis(1), None, None),
                    };

                    // Take ownership of the current state to avoid borrow conflicts
                    let current_state = state.take();

                    match current_state {
                        None | Some(PendingTransition::ClickTracking { .. }) => {
                            if let Some(node) = clicked_node {
                                if self.state.is_deletion_mode() { 
                                    return (canvas::event::Status::Captured, Some(CanvasMessage::StateClicked(node.id)));
                                }

                                if now.duration_since(last_click_time) < double_click_threshold
                                    && last_clicked_state == Some(node.id) {
                                    *state = Some(PendingTransition::ClickTracking {
                                        last_click_time: now,
                                        last_clicked_state: Some(node.id),
                                        last_clicked_transition: None,
                                    });
                                    return (canvas::event::Status::Captured, Some(CanvasMessage::StateDoubleClicked(node.id)));
                                }

                                *state = Some(PendingTransition::ClickTracking {
                                    last_click_time: now,
                                    last_clicked_state: Some(node.id),
                                    last_clicked_transition: None,
                                });

                                if self.state.is_shift_pressed() {
                                    return (canvas::event::Status::Captured, Some(CanvasMessage::StateClicked(node.id)));
                                } else if self.state.is_alt_pressed() {
                                    return (canvas::event::Status::Captured, Some(CanvasMessage::StateClicked(node.id)));
                                } else if self.state.is_ctrl_pressed() {
                                    let offset = cursor_pos - node.position;
                                    *state = Some(PendingTransition::Dragging {
                                        state_id: node.id,
                                        offset
                                    });
                                    return (canvas::event::Status::Captured, None);
                                } else {
                                    *state = Some(PendingTransition::Start {
                                        from_state_id: node.id,
                                        from_point: node.position
                                    });
                                    return (canvas::event::Status::Captured, None);
                                }
                            } else if let Some(transition_index) = clicked_transition_index {
                                if self.state.is_deletion_mode() { 
                                    return (canvas::event::Status::Captured, Some(CanvasMessage::TransitionClicked(transition_index)));
                                }

                                if now.duration_since(last_click_time) < double_click_threshold
                                    && last_clicked_transition == Some(transition_index) {
                                    *state = Some(PendingTransition::ClickTracking {
                                        last_click_time: now,
                                        last_clicked_transition: Some(transition_index),
                                        last_clicked_state: None,
                                    });
                                    return (canvas::event::Status::Captured, Some(CanvasMessage::TransitionDoubleClicked(transition_index)));
                                }

                                *state = Some(PendingTransition::ClickTracking {
                                    last_click_time: now,
                                    last_clicked_transition: Some(transition_index),
                                    last_clicked_state: None,
                                });
                                return (canvas::event::Status::Captured, None);
                            } else {
                                // Dynamically assign a label using the current next_id from State
                                let label = format!("q{}", self.state.get_current_next_id()); 
                                let state_node = StateNode::new_with_temp_id(cursor_pos, 30.0, Box::leak(label.into_boxed_str()));
                                (canvas::event::Status::Captured, Some(CanvasMessage::AddState(state_node)))
                            }
                        }
                        Some(PendingTransition::Start { from_state_id, from_point }) => {
                            if let Some(to_node) = clicked_node {
                                if from_state_id != to_node.id {
                                    *state = None;
                                    let transition = Transition {
                                        from_state_id,
                                        to_state_id: to_node.id,
                                        from_point,
                                        to_point: to_node.position,
                                        label: Box::leak("ε".to_string().into_boxed_str()),
                                    };
                                    (canvas::event::Status::Captured, Some(CanvasMessage::AddTransition(transition)))
                                } else {
                                    *state = None;
                                    (canvas::event::Status::Captured, None)
                                }
                            } else {
                                *state = None;
                                (canvas::event::Status::Captured, None)
                            }
                        }
                        Some(PendingTransition::Dragging { .. }) => {
                            *state = None;
                            (canvas::event::Status::Captured, None)
                        }
                    }
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(cursor_pos) = cursor_position {
                    match state {
                        Some(PendingTransition::Start { .. }) => {
                            (canvas::event::Status::Captured, None)
                        }
                        Some(PendingTransition::Dragging { state_id, offset }) => {
                            let new_position = cursor_pos - *offset;
                            (canvas::event::Status::Captured, Some(CanvasMessage::MoveState {
                                state_id: *state_id,
                                new_position
                            }))
                        }
                        _ => (canvas::event::Status::Ignored, None),
                    }
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                match state {
                    Some(PendingTransition::Dragging { .. }) => {
                        *state = None;
                        (canvas::event::Status::Captured, None)
                    }
                    _ => (canvas::event::Status::Ignored, None),
                }
            }
            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            if self.state.is_deletion_mode() {
                return mouse::Interaction::None; 
            }

            let cursor_position = cursor.position_in(bounds);
            if let Some(pos) = cursor_position {
                match state {
                    Some(PendingTransition::Dragging { .. }) => {
                        return mouse::Interaction::Grabbing;
                    }
                    _ => {
                        if self.states.iter().any(|node| (pos - node.position).length() <= node.radius) {
                            if self.state.is_ctrl_pressed() {
                                return mouse::Interaction::Grab;
                            } else if self.state.is_shift_pressed() || self.state.is_alt_pressed() {
                                return mouse::Interaction::Pointer;
                            } else {
                                return mouse::Interaction::Crosshair;
                            }
                        }
                    }
                }
            }
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let content = self.state.cache.draw(renderer, bounds.size(), |frame| {
            Transition::draw_all(self.transitions, frame, theme, self.states);

            StateNode::draw_all(self.states, frame, theme, self.initial_state, self.final_states);

            frame.stroke(
                &Path::rectangle(Point::ORIGIN, frame.size()),
                Stroke::default()
                    .with_width(2.0)
                    .with_color(theme.palette().text),
            );
        });

        let mut geometries = vec![content];

        // Draw deletion cursor if in deletion mode
        if self.state.is_deletion_mode() {
            if let Some(cursor_position) = cursor.position_in(bounds) {
                let mut cursor_frame = Frame::new(renderer, bounds.size());
                let x_size = 10.0; // Size of the 'X' arms
                let line_width = 2.0;

                // Draw first line of the X
                cursor_frame.stroke(
                    &Path::line(
                        Point::new(cursor_position.x - x_size, cursor_position.y - x_size),
                        Point::new(cursor_position.x + x_size, cursor_position.y + x_size),
                    ),
                    Stroke::default()
                        .with_width(line_width)
                        .with_color(iced::Color::from_rgb(1.0, 0.0, 0.0)), // Red color
                );

                // Draw second line of the X
                cursor_frame.stroke(
                    &Path::line(
                        Point::new(cursor_position.x + x_size, cursor_position.y - x_size),
                        Point::new(cursor_position.x - x_size, cursor_position.y + x_size),
                    ),
                    Stroke::default()
                        .with_width(line_width)
                        .with_color(iced::Color::from_rgb(1.0, 0.0, 0.0)), // Red color
                );
                geometries.push(cursor_frame.into_geometry());
            }
        }

        if let Some(pending) = state {
            geometries.push(pending.draw(renderer, theme, bounds, cursor, self.states));
        }
        geometries
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StateNode {
    pub id: usize,
    pub position: Point,
    pub radius: f32,
    pub label: &'static str,
}

impl StateNode {
    pub fn new(id: usize, position: Point, radius: f32, label: &'static str) -> Self {
        StateNode { id, position, radius, label }
    }

    pub fn new_with_temp_id(position: Point, radius: f32, label: &'static str) -> Self {
        // When created temporarily in the canvas, its ID is 0 and label is a placeholder.
        // The real ID and label based on next_id will be assigned in App::update.
        StateNode { id: 0, position, radius, label } 
    }

    fn draw(&self, frame: &mut Frame, theme: &Theme, is_initial: bool, is_final: bool) {
        // Draw main circle
        frame.fill(
            &Path::circle(self.position, self.radius),
            theme.palette().background,
        );

        frame.stroke(
            &Path::circle(self.position, self.radius),
            Stroke::default()
                .with_width(2.0)
                .with_color(theme.palette().text),
        );

        if is_final {
            let inner_radius = self.radius - 5.0;
            frame.stroke(
                &Path::circle(self.position, inner_radius),
                Stroke::default()
                    .with_width(1.5)
                    .with_color(theme.palette().text),
            );
        }

        if is_initial {
            let arrow_size = 12.0;
            let arrow_height = 12.0;

            let triangle_start_x = self.position.x - self.radius - arrow_size;
            let triangle_y = self.position.y;

            let tip = Point::new(self.position.x - self.radius, triangle_y);
            let base_top = Point::new(triangle_start_x, triangle_y - arrow_height / 2.0);
            let base_bottom = Point::new(triangle_start_x, triangle_y + arrow_height / 2.0);

            let mut path_builder = canvas::path::Builder::new();
            path_builder.move_to(tip);
            path_builder.line_to(base_top);
            path_builder.line_to(base_bottom);
            path_builder.close();
            let triangle_path = path_builder.build();

            frame.fill(&triangle_path, iced::Color::BLACK);
        }

        frame.fill_text(Text {
            content: self.label.to_string(),
            position: self.position,
            color: theme.palette().text,
            size: 14.0.into(),
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
            ..Text::default()
        });
    }

    fn draw_all(
        nodes: &[StateNode],
        frame: &mut Frame,
        theme: &Theme,
        initial_state: Option<usize>,
        final_states: &HashSet<usize>
    ) {
        for node in nodes {
            let is_initial = initial_state == Some(node.id);
            let is_final = final_states.contains(&node.id);
            node.draw(frame, theme, is_initial, is_final);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Transition {
    pub from_state_id: usize,
    pub to_state_id: usize,
    pub from_point: Point,
    pub to_point: Point,
    pub label: &'static str,
}

impl Transition {
    fn draw_all(transitions: &[Transition], frame: &mut Frame, theme: &Theme, states: &[StateNode]) {
        for transition in transitions.iter() {
            let has_reverse = transitions.iter().any(|other_trans|
                other_trans.from_state_id == transition.to_state_id &&
                other_trans.to_state_id == transition.from_state_id
            );

            transition.draw(frame, theme, states, has_reverse);
        }
    }

    fn draw(&self, frame: &mut Frame, theme: &Theme, states: &[StateNode], has_reverse: bool) {
        let from_state = states.iter().find(|s| s.id == self.from_state_id);
        let to_state = states.iter().find(|s| s.id == self.to_state_id);

        if let (Some(from_state), Some(to_state)) = (from_state, to_state) {
            if has_reverse {
                self.draw_curved_transition(frame, theme, from_state, to_state);
            } else {
                self.draw_straight_transition(frame, theme, from_state, to_state);
            }
        }
    }

    fn draw_straight_transition(&self, frame: &mut Frame, theme: &Theme, from_state: &StateNode, to_state: &StateNode) {
        let direction = to_state.position - from_state.position;
        let direction_unit = direction.unit();

        let start_point = from_state.position + direction_unit * from_state.radius;
        let end_point = to_state.position - direction_unit * to_state.radius;

        frame.stroke(
            &Path::line(start_point, end_point),
            Stroke::default()
                .with_width(1.5)
                .with_color(theme.palette().text),
        );

        self.draw_arrowhead(frame, theme, end_point, direction_unit);

        let midpoint = Point::new(
            (start_point.x + end_point.x) / 2.0,
            (start_point.y + end_point.y) / 2.0,
        );

        let perpendicular_vec = Vector::new(-direction.y, direction.x).unit() * 15.0;

        frame.fill_text(Text {
            content: self.label.to_string(),
            position: midpoint + perpendicular_vec,
            color: theme.palette().text,
            size: 12.0.into(),
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
            ..Text::default()
        });
    }

    fn draw_curved_transition(&self, frame: &mut Frame, theme: &Theme, from_state: &StateNode, to_state: &StateNode) {
        let center_to_center = to_state.position - from_state.position;
        let distance = center_to_center.length();

        let (node_a_pos, node_b_pos) = if self.from_state_id < self.to_state_id {
            (from_state.position, to_state.position)
        } else {
            (to_state.position, from_state.position)
        };
        let consistent_direction = node_b_pos - node_a_pos;
        let consistent_perpendicular = Vector::new(-consistent_direction.y, consistent_direction.x).unit();

        let curve_side_multiplier = if self.from_state_id < self.to_state_id { 1.0 } else { -1.0 };

        let curve_offset = distance * 0.4;

        let midpoint = Point::new(
            (from_state.position.x + to_state.position.x) / 2.0,
            (from_state.position.y + to_state.position.y) / 2.0,
        );
        let control_point = midpoint + consistent_perpendicular * curve_offset * curve_side_multiplier;

        let start_direction = (control_point - from_state.position).unit();
        let end_direction = (to_state.position - control_point).unit();

        let start_point = from_state.position + start_direction * from_state.radius;
        let end_point = to_state.position - end_direction * to_state.radius;

        let mut path_builder = canvas::path::Builder::new();
        path_builder.move_to(start_point);
        path_builder.quadratic_curve_to(control_point, end_point);
        let curve_path = path_builder.build();

        frame.stroke(
            &curve_path,
            Stroke::default()
                .with_width(1.5)
                .with_color(theme.palette().text),
        );

        self.draw_arrowhead(frame, theme, end_point, end_direction);

        let label_position = self.calculate_curve_midpoint(start_point, control_point, end_point);
        let label_offset = consistent_perpendicular * (25.0 * curve_side_multiplier);

        frame.fill_text(Text {
            content: self.label.to_string(),
            position: label_position + label_offset,
            color: theme.palette().text,
            size: 12.0.into(),
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
            ..Text::default()
        });
    }

    fn calculate_curve_midpoint(&self, start: Point, control: Point, end: Point) -> Point {
        // B(t) = (1-t)²P₀ + 2(1-t)tP₁ + t²P₂
        let t = 0.5;
        let one_minus_t = 1.0 - t;

        Point::new(
            one_minus_t * one_minus_t * start.x + 2.0 * one_minus_t * t * control.x + t * t * end.x,
            one_minus_t * one_minus_t * start.y + 2.0 * one_minus_t * t * control.y + t * t * end.y,
        )
    }

    fn draw_arrowhead(&self, frame: &mut Frame, _theme: &Theme, tip: Point, direction: Vector) {
        let arrow_length = 12.0;
        let arrow_angle = std::f32::consts::PI / 6.0; // 30 degrees

        let cos_angle = arrow_angle.cos();
        let sin_angle = arrow_angle.sin();

        let reverse_dir = direction * -arrow_length;

        let wing1 = Point::new(
            tip.x + reverse_dir.x * cos_angle - reverse_dir.y * sin_angle,
            tip.y + reverse_dir.x * sin_angle + reverse_dir.y * cos_angle,
        );

        let wing2 = Point::new(
            tip.x + reverse_dir.x * cos_angle + reverse_dir.y * sin_angle,
            tip.y - reverse_dir.x * sin_angle + reverse_dir.y * cos_angle,
        );

        let mut path_builder = canvas::path::Builder::new();
        path_builder.move_to(tip);
        path_builder.line_to(wing1);
        path_builder.line_to(wing2);
        path_builder.close();
        let triangle_path = path_builder.build();

        frame.fill(&triangle_path, iced::Color::BLACK);
    }
}

#[derive(Debug, Clone, Copy)]
enum PendingTransition {
    Start {
        from_state_id: usize,
        from_point: Point,
    },
    Dragging {
        state_id: usize,
        offset: Vector,
    },
    ClickTracking {
        last_click_time: std::time::Instant,
        last_clicked_state: Option<usize>,
        last_clicked_transition: Option<usize>,
    },
}

impl PendingTransition {
    fn draw(
        &self,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        states: &[StateNode],
    ) -> Geometry {
        let mut frame = Frame::new(renderer, bounds.size());

        if let Some(cursor_position) = cursor.position_in(bounds) {
            match *self {
                PendingTransition::Start { from_point, .. } => {
                    let line = Path::line(from_point, cursor_position);
                    frame.stroke(
                        &line,
                        Stroke::default()
                            .with_width(2.0)
                            .with_color(theme.palette().text),
                    );
                }
                PendingTransition::Dragging { state_id, offset } => {
                    let drag_position = cursor_position - offset;
                    let node = states.iter().find(|s| s.id == state_id);

                    if let Some(node) = node {
                        frame.fill(
                            &Path::circle(drag_position, node.radius),
                            iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                        );
                        frame.stroke(
                            &Path::circle(drag_position, node.radius),
                            Stroke::default()
                                .with_width(2.0)
                                .with_color(theme.palette().text),
                        );
                    }
                }
                PendingTransition::ClickTracking { .. } => {
                }
            }
        }
        frame.into_geometry()
    }
}

