use iced::widget::canvas::{Frame, Geometry, Path, Stroke};
use iced::{mouse, Point, Rectangle, Renderer, Theme, Vector};

use super::node::StateNode;

#[derive(Debug, Clone, Copy)]
pub(crate) enum PendingTransition {
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
    pub(crate) fn draw(
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
