use iced::widget::canvas::{Frame, Path, Stroke, Text};
use iced::{alignment, Color, Point, Vector};

use crate::gui::theme::{self, Palette};

#[derive(Debug, Clone)]
pub struct StateNode {
    pub id: usize,
    pub position: Point,
    pub radius: f32,
    pub label: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct NodeLook {
    pub(crate) initial: bool,
    pub(crate) accepting: bool,
    pub(crate) active: bool,
}

impl StateNode {
    pub fn new(id: usize, position: Point, radius: f32, label: String) -> Self {
        StateNode { id, position, radius, label }
    }

    pub fn new_with_temp_id(position: Point, radius: f32, label: String) -> Self {
        StateNode { id: 0, position, radius, label }
    }

    pub(crate) fn draw(&self, frame: &mut Frame, p: &Palette, tint: Color, look: NodeLook) {
        let center = self.position;
        let r = self.radius;

        if look.active {
            frame.fill(&Path::circle(center, r + 11.0), theme::alpha(p.accent, 0.14));
            frame.fill(&Path::circle(center, r + 6.0), theme::alpha(p.accent, 0.26));
        }

        frame.fill(&Path::circle(center + Vector::new(0.0, 2.5), r + 0.5), p.shadow);
        let fill = if look.active {
            theme::mix(p.node_fill, p.accent, 0.35)
        } else {
            theme::mix(p.node_fill, tint, 0.28)
        };
        frame.fill(&Path::circle(center, r), fill);
        frame.stroke(
            &Path::circle(center, r),
            Stroke::default()
                .with_width(if look.active { 2.5 } else { 2.0 })
                .with_color(if look.active { p.accent_strong } else { p.node_stroke }),
        );

        if look.accepting {
            frame.stroke(
                &Path::circle(center, r - 5.0),
                Stroke::default()
                    .with_width(1.5)
                    .with_color(if look.active { p.accent_strong } else { p.node_stroke }),
            );
        }

        if look.initial {
            let tip = Point::new(center.x - r - 3.0, center.y);
            let pointer = Path::new(|b| {
                b.move_to(tip);
                b.line_to(Point::new(tip.x - 16.0, center.y - 9.0));
                b.line_to(Point::new(tip.x - 12.0, center.y));
                b.line_to(Point::new(tip.x - 16.0, center.y + 9.0));
                b.close();
            });
            frame.fill(&pointer, p.accent);
            frame.stroke(
                &pointer,
                Stroke::default()
                    .with_width(1.2)
                    .with_color(p.accent_strong)
                    .with_line_join(iced::widget::canvas::LineJoin::Round),
            );
        }

        frame.fill_text(Text {
            content: self.label.to_string(),
            position: center,
            color: p.text,
            size: 14.0.into(),
            font: theme::SEMIBOLD,
            shaping: crate::gui::widgets::shaping_for(&self.label),
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
            ..Text::default()
        });
    }
}
