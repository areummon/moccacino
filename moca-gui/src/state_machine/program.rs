use iced::widget::canvas::{self, Event, Frame, Geometry, Path, Stroke, Text};
use iced::{alignment, mouse, Point, Rectangle, Renderer, Size, Theme, Vector};
use std::collections::HashSet;

use super::message::CanvasMessage;
use super::node::StateNode;
use super::pending::PendingTransition;
use super::state::State;
use super::tool::EditorTool;
use super::util::VectorExt;
use crate::gui::theme;

pub(crate) struct StateMachine<'a> {
    pub(crate) state: &'a State,
    pub(crate) states: &'a [StateNode],
    pub(crate) transitions: &'a std::collections::HashMap<(usize, usize), indexmap::IndexSet<String>>,
    pub(crate) initial_state: Option<usize>,
    pub(crate) final_states: &'a HashSet<usize>,
}

impl StateMachine<'_> {
    /* Bounding box of all drawn content in world coordinates: every
     * state plus a margin covering self-loops and labels. */
    pub(crate) fn content_extent(&self) -> (f32, f32) {
        const MARGIN: f32 = 90.0;
        let mut max_x: f32 = 1.0;
        let mut max_y: f32 = 1.0;
        for state in self.states {
            max_x = max_x.max(state.position.x + state.radius + MARGIN);
            max_y = max_y.max(state.position.y + state.radius + MARGIN);
        }
        (max_x, max_y)
    }

    /* How far the viewport may scroll on each axis; zero when the content
     * fits inside the visible area. */
    fn scroll_extent(&self, bounds: Rectangle) -> (f32, f32) {
        let (content_w, content_h) = self.content_extent();
        ((content_w - bounds.width).max(0.0), (content_h - bounds.height).max(0.0))
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
        let scroll = self.state.scroll();
        // Hit-testing happens in world coordinates: the stored state
        // positions are world positions, the cursor is viewport-relative.
        let world_position = cursor_position.map(|pos| pos + scroll);

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if let Some(_) = cursor_position {
                    let (dx, dy) = match delta {
                        mouse::ScrollDelta::Lines { x, y } => (x * 40.0, y * 40.0),
                        mouse::ScrollDelta::Pixels { x, y } => (x, y),
                    };
                    let (max_x, max_y) = self.scroll_extent(bounds);
                    let new_scroll = Vector::new(
                        (scroll.x + dx).clamp(0.0, max_x),
                        (scroll.y - dy).clamp(0.0, max_y),
                    );
                    if (new_scroll.x - scroll.x).abs() > f32::EPSILON
                        || (new_scroll.y - scroll.y).abs() > f32::EPSILON
                    {
                        (canvas::event::Status::Captured, Some(CanvasMessage::Scrolled(new_scroll)))
                    } else {
                        (canvas::event::Status::Captured, None)
                    }
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(cursor_pos) = world_position else {
                    return (canvas::event::Status::Ignored, None);
                };

                // Shared hit-testing: the active tool decides what a press
                // on each target means.
                let clicked_node = self.states.iter().find(|node| {
                    (cursor_pos - node.position).length() <= node.radius
                });
                let clicked_transition_index = self.find_transition_at_point(cursor_pos);

                match self.state.active_tool() {
                    EditorTool::Arrow => {
                        let now = std::time::Instant::now();
                        let double_click_threshold = std::time::Duration::from_millis(300);
                        let (last_click_time, last_clicked_state, last_clicked_transition) =
                            match state {
                                Some(PendingTransition::ClickTracking {
                                    last_click_time,
                                    last_clicked_state,
                                    last_clicked_transition,
                                }) => (*last_click_time, *last_clicked_state, *last_clicked_transition),
                                _ => (
                                    now - double_click_threshold - std::time::Duration::from_millis(1),
                                    None,
                                    None,
                                ),
                            };
                        let _ = state.take();

                        if let Some(node) = clicked_node {
                            if now.duration_since(last_click_time) < double_click_threshold
                                && last_clicked_state == Some(node.id)
                            {
                                *state = Some(PendingTransition::ClickTracking {
                                    last_click_time: now,
                                    last_clicked_state: Some(node.id),
                                    last_clicked_transition: None,
                                });
                                return (
                                    canvas::event::Status::Captured,
                                    Some(CanvasMessage::StateDoubleClicked(node.id)),
                                );
                            }
                            // Modifier clicks never become drags, and they
                            // leave no click tracking so a fast modifier
                            // double-tap cannot trigger a rename.
                            if self.state.is_shift_pressed() || self.state.is_alt_pressed() {
                                return (
                                    canvas::event::Status::Captured,
                                    Some(CanvasMessage::StateClicked(node.id)),
                                );
                            }
                            *state = Some(PendingTransition::Dragging {
                                state_id: node.id,
                                offset: cursor_pos - node.position,
                                moved: false,
                            });
                            return (canvas::event::Status::Captured, None);
                        }
                        if let Some(transition_index) = clicked_transition_index {
                            if now.duration_since(last_click_time) < double_click_threshold
                                && last_clicked_transition == Some(transition_index)
                            {
                                *state = Some(PendingTransition::ClickTracking {
                                    last_click_time: now,
                                    last_clicked_state: None,
                                    last_clicked_transition: Some(transition_index),
                                });
                                return (
                                    canvas::event::Status::Captured,
                                    Some(CanvasMessage::TransitionDoubleClicked(transition_index)),
                                );
                            }
                            *state = Some(PendingTransition::ClickTracking {
                                last_click_time: now,
                                last_clicked_state: None,
                                last_clicked_transition: Some(transition_index),
                            });
                            return (
                                canvas::event::Status::Captured,
                                Some(CanvasMessage::TransitionClicked(transition_index)),
                            );
                        }
                        // Empty space: begin panning the viewport.
                        *state = Some(PendingTransition::Panning {
                            origin_scroll: scroll,
                            cursor_start: cursor_pos,
                        });
                        (canvas::event::Status::Captured, None)
                    }
                    EditorTool::State => {
                        *state = None;
                        if clicked_node.is_none() && clicked_transition_index.is_none() {
                            let label = format!("q{}", self.state.get_current_next_id());
                            let state_node = StateNode::new_with_temp_id(
                                cursor_pos,
                                30.0,
                                label,
                            );
                            (canvas::event::Status::Captured, Some(CanvasMessage::AddState(state_node)))
                        } else {
                            (canvas::event::Status::Captured, None)
                        }
                    }
                    EditorTool::Transition => {
                        match state.take() {
                            Some(PendingTransition::Start { from_state_id, from_point }) => {
                                if let Some(to_node) = clicked_node {
                                    // Instead of adding the transition here,
                                    // request a label from the GUI.
                                    (
                                        canvas::event::Status::Captured,
                                        Some(CanvasMessage::RequestTransitionLabel {
                                            from_state_id,
                                            to_state_id: to_node.id,
                                            from_point,
                                            to_point: to_node.position,
                                        }),
                                    )
                                } else {
                                    // Clicking anything but a state cancels.
                                    (canvas::event::Status::Captured, None)
                                }
                            }
                            _ => {
                                if let Some(node) = clicked_node {
                                    *state = Some(PendingTransition::Start {
                                        from_state_id: node.id,
                                        from_point: node.position,
                                    });
                                }
                                (canvas::event::Status::Captured, None)
                            }
                        }
                    }
                    EditorTool::Delete => {
                        *state = None;
                        if let Some(node) = clicked_node {
                            (canvas::event::Status::Captured, Some(CanvasMessage::StateClicked(node.id)))
                        } else if let Some(transition_index) = clicked_transition_index {
                            (
                                canvas::event::Status::Captured,
                                Some(CanvasMessage::TransitionClicked(transition_index)),
                            )
                        } else {
                            (canvas::event::Status::Captured, None)
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(cursor_pos) = cursor_position {
                    // Self-heal after content shrinks (state deleted or
                    // moved): clamp the stored scroll back into range.
                    let (max_x, max_y) = self.scroll_extent(bounds);
                    if scroll.x > max_x || scroll.y > max_y {
                        return (
                            canvas::event::Status::Captured,
                            Some(CanvasMessage::Scrolled(Vector::new(
                                scroll.x.min(max_x),
                                scroll.y.min(max_y),
                            ))),
                        );
                    }
                    match state {
                        Some(PendingTransition::Start { .. }) => {
                            (canvas::event::Status::Captured, None)
                        }
                        Some(PendingTransition::Dragging { state_id, offset, .. }) => {
                            let (state_id, offset) = (*state_id, *offset);
                            let new_position = cursor_pos + scroll - offset;
                            *state = Some(PendingTransition::Dragging {
                                state_id,
                                offset,
                                moved: true,
                            });
                            (
                                canvas::event::Status::Captured,
                                Some(CanvasMessage::MoveState {
                                    state_id,
                                    new_position,
                                }),
                            )
                        }
                        Some(PendingTransition::Panning { origin_scroll, cursor_start }) => {
                            let drag = cursor_pos - *cursor_start;
                            let new_scroll = Vector::new(
                                (origin_scroll.x - drag.x).clamp(0.0, max_x),
                                (origin_scroll.y - drag.y).clamp(0.0, max_y),
                            );
                            if (new_scroll.x - scroll.x).abs() > f32::EPSILON
                                || (new_scroll.y - scroll.y).abs() > f32::EPSILON
                            {
                                (
                                    canvas::event::Status::Captured,
                                    Some(CanvasMessage::Scrolled(new_scroll)),
                                )
                            } else {
                                (canvas::event::Status::Captured, None)
                            }
                        }
                        _ => (canvas::event::Status::Ignored, None),
                    }
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                match state {
                    // A press-release without movement counts as a click:
                    // keep it tracked so the next press can detect a
                    // double-click rename. Real drags clear the tracking.
                    Some(PendingTransition::Dragging { state_id, moved, .. }) => {
                        let (state_id, moved) = (*state_id, *moved);
                        *state = if moved {
                            None
                        } else {
                            Some(PendingTransition::ClickTracking {
                                last_click_time: std::time::Instant::now(),
                                last_clicked_state: Some(state_id),
                                last_clicked_transition: None,
                            })
                        };
                        (canvas::event::Status::Captured, None)
                    }
                    Some(PendingTransition::Panning { .. }) => {
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
        if !cursor.is_over(bounds) {
            return mouse::Interaction::default();
        }
        match self.state.active_tool() {
            // The delete overlay draws its own cross; keep the system
            // cursor hidden beneath it.
            EditorTool::Delete => mouse::Interaction::None,
            EditorTool::State | EditorTool::Transition => mouse::Interaction::Crosshair,
            EditorTool::Arrow => match state {
                // Special cursors only while an interaction is in
                // progress; idle hovering keeps the normal cursor.
                Some(PendingTransition::Dragging { .. }) => mouse::Interaction::Grabbing,
                Some(PendingTransition::Panning { .. }) => mouse::Interaction::Move,
                _ => mouse::Interaction::Idle,
            },
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
        let scroll = self.state.scroll();
        let content = self.state.cache.draw(renderer, bounds.size(), |frame| {
            frame.fill(
                &Path::rectangle(Point::ORIGIN, frame.size()),
                theme::BG,
            );

            // World content is drawn shifted by the scroll offset; the
            // background fill above stays viewport-anchored.
            frame.translate(-scroll);

            // Per-transition state lookups come from this map instead of an
            // O(states) search per transition.
            let node_by_id: std::collections::HashMap<usize, &StateNode> =
                self.states.iter().map(|node| (node.id, node)).collect();

            for ((from_id, to_id), labels) in self.transitions.iter() {
                let from_state = node_by_id.get(from_id).copied();
                let to_state = node_by_id.get(to_id).copied();
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
                                .with_color(theme::CREAM),
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
                                .with_color(theme::CREAM),
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
                                color: theme::CREAM,
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
                                    .with_color(theme::CREAM),
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
                                    .with_color(theme::CREAM),
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
                                    color: theme::CREAM,
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
                                    .with_color(theme::CREAM),
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
                                    .with_color(theme::CREAM),
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
                                    color: theme::CREAM,
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
                        .with_color(theme::REJECT),
                );

                cursor_frame.stroke(
                    &Path::line(
                        Point::new(cursor_position.x + x_size, cursor_position.y - x_size),
                        Point::new(cursor_position.x - x_size, cursor_position.y + x_size),
                    ),
                    Stroke::default()
                        .with_width(line_width)
                        .with_color(theme::REJECT),
                );
                geometries.push(cursor_frame.into_geometry());
            }
        }

        if let Some(pending) = state {
            geometries.push(pending.draw(renderer, _theme, bounds, cursor, self.states, scroll));
        }

        // Thin scrollbar indicators whenever the content escapes the
        // visible area; the wheel pans and these track its position.
        let (max_x, max_y) = self.scroll_extent(bounds);
        if max_x > 0.0 || max_y > 0.0 {
            let mut bars_frame = Frame::new(renderer, bounds.size());
            let view = bounds.size();
            const BAR_THICKNESS: f32 = 4.0;
            const BAR_MARGIN: f32 = 3.0;
            if max_y > 0.0 {
                let content_height = view.height + max_y;
                let thumb_height = (view.height * view.height / content_height).max(24.0);
                let travel = view.height - thumb_height - 2.0 * BAR_MARGIN;
                let thumb_y = BAR_MARGIN + scroll.y / max_y * travel;
                bars_frame.fill(
                    &Path::rounded_rectangle(
                        Point::new(view.width - BAR_THICKNESS - BAR_MARGIN, thumb_y),
                        Size::new(BAR_THICKNESS, thumb_height),
                        2.0.into(),
                    ),
                    theme::BORDER,
                );
            }
            if max_x > 0.0 {
                let content_width = view.width + max_x;
                let thumb_width = (view.width * view.width / content_width).max(24.0);
                let travel = view.width - thumb_width - 2.0 * BAR_MARGIN;
                let thumb_x = BAR_MARGIN + scroll.x / max_x * travel;
                bars_frame.fill(
                    &Path::rounded_rectangle(
                        Point::new(thumb_x, view.height - BAR_THICKNESS - BAR_MARGIN),
                        Size::new(thumb_width, BAR_THICKNESS),
                        2.0.into(),
                    ),
                    theme::BORDER,
                );
            }
            geometries.push(bars_frame.into_geometry());
        }
        geometries
    }
}
