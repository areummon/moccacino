use iced::widget::canvas::{self, Event, Frame, Geometry, Path, Stroke, Text};
use iced::{alignment, mouse, Point, Rectangle, Renderer, Theme, Vector};
use std::collections::HashSet;

use super::message::CanvasMessage;
use super::node::StateNode;
use super::pending::PendingTransition;
use super::state::State;
use super::util::VectorExt;

pub(crate) struct StateMachine<'a> {
    pub(crate) state: &'a State,
    pub(crate) states: &'a [StateNode],
    pub(crate) transitions: &'a std::collections::HashMap<(usize, usize), indexmap::IndexSet<String>>,
    pub(crate) initial_state: Option<usize>,
    pub(crate) final_states: &'a HashSet<usize>,
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
