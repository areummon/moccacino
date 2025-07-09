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
    RequestTransitionLabel {
        from_state_id: usize,
        to_state_id: usize,
        from_point: Point,
        to_point: Point,
    },
}

pub struct State {
    cache: canvas::Cache,
    ctrl_pressed: bool,
    shift_pressed: bool,
    alt_pressed: bool,
    deletion_mode: bool,
    pub next_id: usize, 
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
        transitions: &'a std::collections::HashMap<(usize, usize), indexmap::IndexSet<String>>,
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

    pub fn get_current_next_id(&self) -> usize {
        self.next_id
    }

    pub fn reset_id_counter(&mut self) {
        self.next_id = 0;
    }

    pub fn check_input(&self, _input: &str) -> bool {
        false
    }
}

struct StateMachine<'a> {
    state: &'a State,
    states: &'a [StateNode],
    transitions: &'a std::collections::HashMap<(usize, usize), indexmap::IndexSet<String>>,
    initial_state: Option<usize>,
    final_states: &'a HashSet<usize>,
}

impl StateMachine<'_> {
    fn find_transition_at_point(&self, point: Point) -> Option<usize> {
        for (index, (key, labels)) in self.transitions.iter().enumerate() {
            if let (Some(from_state), Some(to_state)) = (
                self.states.iter().find(|s| s.id == key.0),
                self.states.iter().find(|s| s.id == key.1)
            ) {
                if key.0 == key.1 {
                    let center = from_state.position;
                    let control = Point::new(center.x, center.y - from_state.radius * 3.8);
                    // Calculate loop curve points for label positioning (same as drawing)
                    let r = from_state.radius;
                    let theta = std::f32::consts::PI / 4.0;
                    let start = Point::new(
                        (center.x - r * theta.cos()) + 4.0,
                        (center.y - r * theta.sin()) + 4.0,
                    );
                    let end = Point::new(
                        (center.x + r * theta.cos()) - 4.0,
                        (center.y - r * theta.sin()) - 4.0,
                    );
                    
                    // Calculate label position (same as drawing)
                    let t = 0.5;
                    let one_minus_t = 1.0 - t;
                    let midpoint = Point::new(
                        one_minus_t * one_minus_t * start.x + 2.0 * one_minus_t * t * control.x + t * t * end.x,
                        one_minus_t * one_minus_t * start.y + 2.0 * one_minus_t * t * control.y + t * t * end.y,
                    );
                    let label_pos = Point::new(midpoint.x, midpoint.y - 10.0);
                    
                    let label_count = labels.len().max(1);
                    let label_height = (label_count as f32) * 18.0 + 16.0;
                    let width = 80.0;
                    
                    // Extended rectangular area: covers both the label stack above and the loop curve area below
                    let rect_left = label_pos.x - width / 2.0;
                    let rect_right = label_pos.x + width / 2.0;
                    let rect_top = label_pos.y - label_height; // Start from top of label stack
                    let rect_bottom = label_pos.y + 60.0; // Extend down to cover the loop curve area
                    
                    if point.x >= rect_left && point.x <= rect_right && point.y >= rect_top && point.y <= rect_bottom {
                        return Some(index);
                    }
                } else {
                    // Calculate label position and stack direction
                    let has_reverse = self.transitions.contains_key(&(key.1, key.0));
                    let from_state = from_state;
                    let to_state = to_state;
                    let label_count = labels.len().max(1);
                    let height = (label_count as f32) * 18.0 + 16.0;
                    let width = 80.0;
                    let (label_pos, stack_vec) = if has_reverse {
                        // Curved: use the same logic as drawing
                        let center_to_center = to_state.position - from_state.position;
                        let distance = center_to_center.length();
                        let (node_a_pos, node_b_pos) = if key.0 < key.1 {
                            (from_state.position, to_state.position)
                        } else {
                            (to_state.position, from_state.position)
                        };
                        let consistent_direction = node_b_pos - node_a_pos;
                        let consistent_perpendicular = Vector::new(-consistent_direction.y, consistent_direction.x).unit();
                        let curve_side_multiplier = if key.0 < key.1 { 1.0 } else { -1.0 };
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
                        let t = 0.5;
                        let one_minus_t = 1.0 - t;
                        let curve_midpoint = Point::new(
                            one_minus_t * one_minus_t * start_point.x + 2.0 * one_minus_t * t * control_point.x + t * t * end_point.x,
                            one_minus_t * one_minus_t * start_point.y + 2.0 * one_minus_t * t * control_point.y + t * t * end_point.y,
                        );
                        let label_offset = consistent_perpendicular * (25.0 * curve_side_multiplier);
                        (curve_midpoint + label_offset, label_offset.unit())
                    } else {
                        // Straight: use -perpendicular_vec as in drawing
                        let direction = to_state.position - from_state.position;
                        let midpoint = Point::new(
                            (from_state.position.x + to_state.position.x) / 2.0,
                            (from_state.position.y + to_state.position.y) / 2.0,
                        );
                        let perpendicular_vec = Vector::new(-direction.y, direction.x).unit() * 15.0;
                        (midpoint - perpendicular_vec, (-perpendicular_vec).unit())
                    };
                    // Rectangle starts at the top of the label stack and extends downward in the stack direction
                    let rect_top_left = label_pos - stack_vec * 0.0 - Vector::new(width / 2.0, 0.0);
                    let rect_bottom_right = label_pos + stack_vec * height + Vector::new(width / 2.0, 0.0);
                    // Check if point is inside the rectangle (project point onto stack direction)
                    let rel = point - rect_top_left;
                    let stack_proj = rel.x * stack_vec.x + rel.y * stack_vec.y;
                    let ortho_vec = Vector::new(-stack_vec.y, stack_vec.x);
                    let ortho_proj = rel.x * ortho_vec.x + rel.y * ortho_vec.y;
                    if stack_proj >= 0.0 && stack_proj <= height && ortho_proj >= 0.0 && ortho_proj <= width {
                        return Some(index);
                    }
                    // Also allow clicking near the line (legacy behavior)
                    let direction = to_state.position - from_state.position;
                    let direction_unit = direction.unit();
                    let start_point = from_state.position + direction_unit * from_state.radius;
                    let end_point = to_state.position - direction_unit * to_state.radius;
                    let line_midpoint = Point::new((start_point.x + end_point.x) / 2.0, (start_point.y + end_point.y) / 2.0);
                    let line_distance = (point - line_midpoint).length();
                    if line_distance <= 20.0 {
                        return Some(index);
                    }
                }
            }
        }
        None
    }

    fn calculate_transition_label_position(&self, transition: &(usize, usize), from_state: &StateNode, to_state: &StateNode) -> Point {
        let has_reverse = self.transitions.iter().any(|other_trans|
            other_trans.0.0 == transition.1 &&
            other_trans.0.1 == transition.0
        );

        if has_reverse {
            let center_to_center = to_state.position - from_state.position;
            let distance = center_to_center.length();
            let (node_a_pos, node_b_pos) = if transition.0 < transition.1 {
                (from_state.position, to_state.position)
            } else {
                (to_state.position, from_state.position)
            };
            let consistent_direction = node_b_pos - node_a_pos;
            let consistent_perpendicular = Vector::new(-consistent_direction.y, consistent_direction.x).unit();
            let curve_side_multiplier = if transition.0 < transition.1 { 1.0 } else { -1.0 };
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
                                return (canvas::event::Status::Captured, Some(CanvasMessage::TransitionClicked(transition_index)));
                            } else {
                                let label = format!("q{}", self.state.get_current_next_id()); 
                                let state_node = StateNode::new_with_temp_id(cursor_pos, 30.0, Box::leak(label.into_boxed_str()));
                                (canvas::event::Status::Captured, Some(CanvasMessage::AddState(state_node)))
                            }
                        }
                        Some(PendingTransition::Start { from_state_id, from_point }) => {
                            if let Some(to_node) = clicked_node {
                                *state = None;
                                // Instead of adding the transition here, request a label from the GUI
                                return (canvas::event::Status::Captured, Some(CanvasMessage::RequestTransitionLabel {
                                    from_state_id,
                                    to_state_id: to_node.id,
                                    from_point,
                                    to_point: to_node.position,
                                }));
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
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let content = self.state.cache.draw(renderer, bounds.size(), |frame| {
            frame.fill(
                &Path::rectangle(Point::ORIGIN, frame.size()),
                iced::Color::from_rgb(0.1, 0.1, 0.1), 
            );

            for ((from_id, to_id), labels) in self.transitions.iter() {
                let from_state = self.states.iter().find(|s| s.id == *from_id);
                let to_state = self.states.iter().find(|s| s.id == *to_id);
                if let (Some(from_state), Some(to_state)) = (from_state, to_state) {
                    if from_id == to_id {
                        // Draw self-loop
                        let center = from_state.position;
                        let r = from_state.radius;
                        let theta = std::f32::consts::PI / 4.0;
                        let start = iced::Point::new(
                            (center.x - r * theta.cos()) + 4.0,
                            (center.y - r * theta.sin()) + 4.0,
                        );
                        let end = iced::Point::new(
                            (center.x + r * theta.cos()) - 4.0,
                            (center.y - r * theta.sin()) - 4.0,
                        );
                        let control = iced::Point::new(center.x, center.y - r * 3.8);
                        let mut path_builder = canvas::path::Builder::new();
                        path_builder.move_to(start);
                        path_builder.quadratic_curve_to(control, end);
                        let curve_path = path_builder.build();
                        frame.stroke(
                            &curve_path,
                            Stroke::default()
                                .with_width(2.0)
                                .with_color(iced::Color::WHITE),
                        );
                        // Draw arrowhead for loop
                        let t = 0.05;
                        let one_minus_t = 1.0 - t;
                        let arrow_pos = iced::Point::new(
                            one_minus_t * one_minus_t * start.x + 2.0 * one_minus_t * t * control.x + t * t * end.x,
                            one_minus_t * one_minus_t * start.y + 2.0 * one_minus_t * t * control.y + t * t * end.y,
                        );
                        let tangent = iced::Vector::new(
                            2.0 * (one_minus_t * (control.x - start.x) + t * (end.x - control.x)),
                            2.0 * (one_minus_t * (control.y - start.y) + t * (end.y - control.y)),
                        ).unit();
                        let arrow_dir = tangent;
                        let arrow_length = 12.0;
                        let arrow_angle = std::f32::consts::PI / 6.0;
                        let cos_angle = arrow_angle.cos();
                        let sin_angle = arrow_angle.sin();
                        let left = iced::Point::new(
                            arrow_pos.x + arrow_length * (arrow_dir.x * cos_angle - arrow_dir.y * sin_angle),
                            arrow_pos.y + arrow_length * (arrow_dir.x * sin_angle + arrow_dir.y * cos_angle),
                        );
                        let right = iced::Point::new(
                            arrow_pos.x + arrow_length * (arrow_dir.x * cos_angle + arrow_dir.y * sin_angle),
                            arrow_pos.y + arrow_length * (-arrow_dir.x * sin_angle + arrow_dir.y * cos_angle),
                        );
                        let mut arrow_path = canvas::path::Builder::new();
                        arrow_path.move_to(arrow_pos);
                        arrow_path.line_to(left);
                        arrow_path.move_to(arrow_pos);
                        arrow_path.line_to(right);
                        let arrow_path = arrow_path.build();
                        frame.stroke(
                            &arrow_path,
                            Stroke::default()
                                .with_width(2.0)
                                .with_color(iced::Color::WHITE),
                        );
                        // Draw stacked labels above the loop
                        // Place label at midpoint of the loop curve (t=0.5), with a small offset above
                        let t = 0.5;
                        let one_minus_t = 1.0 - t;
                        let midpoint = iced::Point::new(
                            one_minus_t * one_minus_t * start.x + 2.0 * one_minus_t * t * control.x + t * t * end.x,
                            one_minus_t * one_minus_t * start.y + 2.0 * one_minus_t * t * control.y + t * t * end.y,
                        );
                        let label_pos = iced::Point::new(midpoint.x, midpoint.y - 10.0);
                        let mut y_offset = 0.0;
                        for label in labels {
                            frame.fill_text(Text {
                                content: label.to_string(),
                                position: label_pos - Vector::new(0.0, y_offset),
                                color: iced::Color::WHITE,
                                size: 14.0.into(),
                                horizontal_alignment: alignment::Horizontal::Center,
                                vertical_alignment: alignment::Vertical::Center,
                                ..Text::default()
                            });
                            y_offset += 18.0;
                        }
                    } else {
                        // Check for reverse transition
                        let has_reverse = self.transitions.contains_key(&(*to_id, *from_id));
                        if has_reverse {
                            // Draw a curved line for both directions
                            let center_to_center = to_state.position - from_state.position;
                            let distance = center_to_center.length();
                            let (node_a_pos, node_b_pos) = if from_id < to_id {
                                (from_state.position, to_state.position)
                            } else {
                                (to_state.position, from_state.position)
                            };
                            let consistent_direction = node_b_pos - node_a_pos;
                            let consistent_perpendicular = Vector::new(-consistent_direction.y, consistent_direction.x).unit();
                            let curve_side_multiplier = if from_id < to_id { 1.0 } else { -1.0 };
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
                                    .with_color(iced::Color::WHITE),
                            );
                            // Draw the arrowhead
                            let tip = end_point;
                            let arrow_length = 12.0;
                            let arrow_angle = std::f32::consts::PI / 6.0;
                            let cos_angle = arrow_angle.cos();
                            let sin_angle = arrow_angle.sin();
                            let reverse_dir = end_direction * -arrow_length;
                            let left = Point::new(
                                tip.x + reverse_dir.x * cos_angle - reverse_dir.y * sin_angle,
                                tip.y + reverse_dir.x * sin_angle + reverse_dir.y * cos_angle,
                            );
                            let right = Point::new(
                                tip.x + reverse_dir.x * cos_angle + reverse_dir.y * sin_angle,
                                tip.y - reverse_dir.x * sin_angle + reverse_dir.y * cos_angle,
                            );
                            let mut arrow_path = canvas::path::Builder::new();
                            arrow_path.move_to(tip);
                            arrow_path.line_to(left);
                            arrow_path.move_to(tip);
                            arrow_path.line_to(right);
                            let arrow_path = arrow_path.build();
                            frame.stroke(
                                &arrow_path,
                                Stroke::default()
                                    .with_width(2.0)
                                    .with_color(iced::Color::WHITE),
                            );
                            // Draw stacked labels above the curve
                            let label_position = {
                                // Midpoint of the curve
                                let t = 0.5;
                                let one_minus_t = 1.0 - t;
                                Point::new(
                                    one_minus_t * one_minus_t * start_point.x + 2.0 * one_minus_t * t * control_point.x + t * t * end_point.x,
                                    one_minus_t * one_minus_t * start_point.y + 2.0 * one_minus_t * t * control_point.y + t * t * end_point.y,
                                )
                            };
                            let label_offset = consistent_perpendicular * (25.0 * curve_side_multiplier);
                            let mut y_offset = 0.0;
                            for label in labels {
                                frame.fill_text(Text {
                                    content: label.to_string(),
                                    position: label_position + label_offset - Vector::new(0.0, y_offset),
                                    color: iced::Color::WHITE,
                                    size: 14.0.into(),
                                    horizontal_alignment: alignment::Horizontal::Center,
                                    vertical_alignment: alignment::Vertical::Center,
                                    ..Text::default()
                                });
                                y_offset += 18.0;
                            }
                        } else {
                            // Draw the arrow (straight line)
                            let direction = to_state.position - from_state.position;
                            let direction_unit = direction.unit();
                            let start_point = from_state.position + direction_unit * from_state.radius;
                            let end_point = to_state.position - direction_unit * to_state.radius;
                            frame.stroke(
                                &Path::line(start_point, end_point),
                                Stroke::default()
                                    .with_width(1.5)
                                    .with_color(iced::Color::WHITE),
                            );
                            // Draw the arrowhead
                            let arrow_length = 12.0;
                            let arrow_angle = std::f32::consts::PI / 6.0;
                            let cos_angle = arrow_angle.cos();
                            let sin_angle = arrow_angle.sin();
                            let reverse_dir = direction_unit * -arrow_length;
                            let tip = end_point;
                            let left = Point::new(
                                tip.x + reverse_dir.x * cos_angle - reverse_dir.y * sin_angle,
                                tip.y + reverse_dir.x * sin_angle + reverse_dir.y * cos_angle,
                            );
                            let right = Point::new(
                                tip.x + reverse_dir.x * cos_angle + reverse_dir.y * sin_angle,
                                tip.y - reverse_dir.x * sin_angle + reverse_dir.y * cos_angle,
                            );
                            let mut arrow_path = canvas::path::Builder::new();
                            arrow_path.move_to(tip);
                            arrow_path.line_to(left);
                            arrow_path.move_to(tip);
                            arrow_path.line_to(right);
                            let arrow_path = arrow_path.build();
                            frame.stroke(
                                &arrow_path,
                                Stroke::default()
                                    .with_width(2.0)
                                    .with_color(iced::Color::WHITE),
                            );
                            // Draw stacked labels above the line
                            let midpoint = Point::new(
                                (start_point.x + end_point.x) / 2.0,
                                (start_point.y + end_point.y) / 2.0,
                            );
                            let perpendicular_vec = Vector::new(-direction.y, direction.x).unit() * 15.0;
                            let mut y_offset = 0.0;
                            for label in labels {
                                frame.fill_text(Text {
                                    content: label.to_string(),
                                    position: midpoint - perpendicular_vec - Vector::new(0.0, y_offset),
                                    color: iced::Color::WHITE,
                                    size: 14.0.into(),
                                    horizontal_alignment: alignment::Horizontal::Center,
                                    vertical_alignment: alignment::Vertical::Center,
                                    ..Text::default()
                                });
                                y_offset += 18.0;
                            }
                        }
                    }
                }
            }

            StateNode::draw_all(self.states, frame, _theme, self.initial_state, self.final_states);
        });

        let mut geometries = vec![content];

        if self.state.is_deletion_mode() {
            if let Some(cursor_position) = cursor.position_in(bounds) {
                let mut cursor_frame = Frame::new(renderer, bounds.size());
                let x_size = 10.0; 
                let line_width = 2.0;

                cursor_frame.stroke(
                    &Path::line(
                        Point::new(cursor_position.x - x_size, cursor_position.y - x_size),
                        Point::new(cursor_position.x + x_size, cursor_position.y + x_size),
                    ),
                    Stroke::default()
                        .with_width(line_width)
                        .with_color(iced::Color::from_rgb(1.0, 0.0, 0.0)), 
                );

                cursor_frame.stroke(
                    &Path::line(
                        Point::new(cursor_position.x + x_size, cursor_position.y - x_size),
                        Point::new(cursor_position.x - x_size, cursor_position.y + x_size),
                    ),
                    Stroke::default()
                        .with_width(line_width)
                        .with_color(iced::Color::from_rgb(1.0, 0.0, 0.0)), 
                );
                geometries.push(cursor_frame.into_geometry());
            }
        }

        if let Some(pending) = state {
            geometries.push(pending.draw(renderer, _theme, bounds, cursor, self.states));
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
        StateNode { id: 0, position, radius, label } 
    }

    fn draw(&self, frame: &mut Frame, _theme: &Theme, is_initial: bool, is_final: bool) {
        frame.fill(
            &Path::circle(self.position, self.radius),
            iced::Color::from_rgb(0.2, 0.7, 0.4), 
        );

        frame.stroke(
            &Path::circle(self.position, self.radius),
            Stroke::default()
                .with_width(2.0)
                .with_color(iced::Color::WHITE), 
        );

        if is_final {
            let inner_radius = self.radius - 5.0;
            frame.stroke(
                &Path::circle(self.position, inner_radius),
                Stroke::default()
                    .with_width(1.5)
                    .with_color(iced::Color::WHITE), 
            );
        }

        if is_initial {
            let arrow_size = 20.0; 
            let arrow_height = 20.0; 

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

            frame.stroke(
                &triangle_path,
                Stroke::default()
                    .with_width(2.0)
                    .with_color(iced::Color::WHITE),
            );
        }

        frame.fill_text(Text {
            content: self.label.to_string(),
            position: self.position,
            color: iced::Color::WHITE, 
            size: 14.0.into(),
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
            ..Text::default()
        });
    }

    fn draw_all(
        nodes: &[StateNode],
        frame: &mut Frame,
        _theme: &Theme,
        initial_state: Option<usize>,
        final_states: &HashSet<usize>
    ) {
        for node in nodes {
            let is_initial = initial_state == Some(node.id);
            let is_final = final_states.contains(&node.id);
            node.draw(frame, _theme, is_initial, is_final);
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
    fn draw_all(transitions: &[Transition], frame: &mut Frame, _theme: &Theme, states: &[StateNode]) {
        for transition in transitions.iter() {
            let has_reverse = transitions.iter().any(|other_trans|
                other_trans.from_state_id == transition.to_state_id &&
                other_trans.to_state_id == transition.from_state_id
            );

            transition.draw(frame, _theme, states, has_reverse);
        }
    }

    fn draw(&self, frame: &mut Frame, _theme: &Theme, states: &[StateNode], has_reverse: bool) {
        let from_state = states.iter().find(|s| s.id == self.from_state_id);
        let to_state = states.iter().find(|s| s.id == self.to_state_id);

        if let (Some(from_state), Some(to_state)) = (from_state, to_state) {
            if self.from_state_id == self.to_state_id {
                let center = from_state.position;
                let r = from_state.radius;
                let theta = std::f32::consts::PI / 4.0; 
                let start = iced::Point::new(
                    (center.x - r * theta.cos()) + 4.0,
                    (center.y - r * theta.sin()) + 4.0,
                );
                let end = iced::Point::new(
                    (center.x + r * theta.cos()) - 4.0,
                    (center.y - r * theta.sin()) - 4.0,
                );
                let control = iced::Point::new(center.x, center.y - r * 3.8);

                let mut path_builder = canvas::path::Builder::new();
                path_builder.move_to(start);
                path_builder.quadratic_curve_to(control, end);
                let curve_path = path_builder.build();
                frame.stroke(
                    &curve_path,
                    Stroke::default()
                        .with_width(2.0)
                        .with_color(iced::Color::WHITE),
                );

                let t = 0.05;
                let one_minus_t = 1.0 - t;
                let arrow_pos = iced::Point::new(
                    one_minus_t * one_minus_t * start.x + 2.0 * one_minus_t * t * control.x + t * t * end.x,
                    one_minus_t * one_minus_t * start.y + 2.0 * one_minus_t * t * control.y + t * t * end.y,
                );
                let tangent = iced::Vector::new(
                    2.0 * (one_minus_t * (control.x - start.x) + t * (end.x - control.x)),
                    2.0 * (one_minus_t * (control.y - start.y) + t * (end.y - control.y)),
                ).unit();
                let arrow_dir = tangent;
                let arrow_length = 12.0;
                let arrow_angle = std::f32::consts::PI / 6.0; 
                let cos_angle = arrow_angle.cos();
                let sin_angle = arrow_angle.sin();
                let left = iced::Point::new(
                    arrow_pos.x + arrow_length * (arrow_dir.x * cos_angle - arrow_dir.y * sin_angle),
                    arrow_pos.y + arrow_length * (arrow_dir.x * sin_angle + arrow_dir.y * cos_angle),
                );
                let right = iced::Point::new(
                    arrow_pos.x + arrow_length * (arrow_dir.x * cos_angle + arrow_dir.y * sin_angle),
                    arrow_pos.y + arrow_length * (-arrow_dir.x * sin_angle + arrow_dir.y * cos_angle),
                );
                let mut arrow_path = canvas::path::Builder::new();
                arrow_path.move_to(arrow_pos);
                arrow_path.line_to(left);
                arrow_path.move_to(arrow_pos);
                arrow_path.line_to(right);
                let arrow_path = arrow_path.build();
                frame.stroke(
                    &arrow_path,
                    Stroke::default()
                        .with_width(2.0)
                        .with_color(iced::Color::WHITE),
                );

                let label_pos = iced::Point::new(control.x, control.y + 30.0);
                frame.fill_text(Text {
                    content: self.label.to_string(),
                    position: label_pos,
                    color: iced::Color::WHITE,
                    size: 14.0.into(),
                    horizontal_alignment: alignment::Horizontal::Center,
                    vertical_alignment: alignment::Vertical::Center,
                    ..Text::default()
                });
            } else if has_reverse {
                self.draw_curved_transition(frame, _theme, from_state, to_state);
            } else {
                self.draw_straight_transition(frame, _theme, from_state, to_state);
            }
        }
    }

    fn draw_straight_transition(&self, frame: &mut Frame, _theme: &Theme, from_state: &StateNode, to_state: &StateNode) {
        let direction = to_state.position - from_state.position;
        let direction_unit = direction.unit();

        let start_point = from_state.position + direction_unit * from_state.radius;
        let end_point = to_state.position - direction_unit * to_state.radius;

        frame.stroke(
            &Path::line(start_point, end_point),
            Stroke::default()
                .with_width(1.5)
                .with_color(iced::Color::WHITE), 
        );

        self.draw_arrowhead(frame, _theme, end_point, direction_unit);

        let midpoint = Point::new(
            (start_point.x + end_point.x) / 2.0,
            (start_point.y + end_point.y) / 2.0,
        );

        let perpendicular_vec = Vector::new(-direction.y, direction.x).unit() * 15.0;

        frame.fill_text(Text {
            content: self.label.to_string(),
            position: midpoint + perpendicular_vec,
            color: iced::Color::WHITE, 
            size: 14.0.into(),
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
            ..Text::default()
        });
    }

    fn draw_curved_transition(&self, frame: &mut Frame, _theme: &Theme, from_state: &StateNode, to_state: &StateNode) {
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
                .with_color(iced::Color::WHITE), 
        );

        self.draw_arrowhead(frame, _theme, end_point, end_direction);

        let label_position = self.calculate_curve_midpoint(start_point, control_point, end_point);
        let label_offset = consistent_perpendicular * (25.0 * curve_side_multiplier);

        frame.fill_text(Text {
            content: self.label.to_string(),
            position: label_position + label_offset,
            color: iced::Color::WHITE, 
            size: 14.0.into(),
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
        let arrow_angle = std::f32::consts::PI / 6.0; 

        let cos_angle = arrow_angle.cos();
        let sin_angle = arrow_angle.sin();

        let reverse_dir = direction * -arrow_length;

        let left = Point::new(
            tip.x + reverse_dir.x * cos_angle - reverse_dir.y * sin_angle,
            tip.y + reverse_dir.x * sin_angle + reverse_dir.y * cos_angle,
        );

        let right = Point::new(
            tip.x + reverse_dir.x * cos_angle + reverse_dir.y * sin_angle,
            tip.y - reverse_dir.x * sin_angle + reverse_dir.y * cos_angle,
        );

        let mut arrow_path = canvas::path::Builder::new();
        arrow_path.move_to(tip);
        arrow_path.line_to(left);
        arrow_path.move_to(tip);
        arrow_path.line_to(right);
        let arrow_path = arrow_path.build();

        frame.stroke(
            &arrow_path,
            Stroke::default()
                .with_width(2.0)
                .with_color(iced::Color::WHITE),
        );
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
        _theme: &Theme,
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
                            .with_color(iced::Color::WHITE),
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
                                .with_color(iced::Color::WHITE),
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

