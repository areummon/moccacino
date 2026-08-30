use iced::widget::canvas::{self, Frame, Path, Stroke, Text};
use iced::{alignment, Point, Theme, Vector};

use super::node::StateNode;
use super::util::VectorExt;
use crate::gui::theme;

#[derive(Debug, Clone)]
pub struct Transition {
    pub from_state_id: usize,
    pub to_state_id: usize,
    pub from_point: Point,
    pub to_point: Point,
    pub label: String,
}

#[allow(dead_code)]
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
                        .with_color(theme::CREAM),
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
                        .with_color(theme::CREAM),
                );

                let label_pos = iced::Point::new(control.x, control.y + 30.0);
                frame.fill_text(Text {
                    content: self.label.to_string(),
                    position: label_pos,
                    color: theme::CREAM,
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
                .with_color(theme::CREAM),
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
            color: theme::CREAM,
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
                .with_color(theme::CREAM),
        );

        self.draw_arrowhead(frame, _theme, end_point, end_direction);

        let label_position = self.calculate_curve_midpoint(start_point, control_point, end_point);
        let label_offset = consistent_perpendicular * (25.0 * curve_side_multiplier);

        frame.fill_text(Text {
            content: self.label.to_string(),
            position: label_position + label_offset,
            color: theme::CREAM,
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
                .with_color(theme::CREAM),
        );
    }
}
