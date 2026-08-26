use iced::widget::canvas::{self, Frame, Path, Stroke, Text};
use iced::{alignment, Point, Theme};
use std::collections::HashSet;

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

    pub(crate) fn draw_all(
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
