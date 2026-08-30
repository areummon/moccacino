use iced::widget::canvas::{Frame, Geometry, Path, Stroke};
use iced::{mouse, Point, Rectangle, Renderer, Theme, Vector};

use super::node::StateNode;
use crate::gui::theme;

#[derive(Debug, Clone, Copy)]
pub(crate) enum PendingTransition {
    Start {
        from_state_id: usize,
        from_point: Point,
    },
    Dragging {
        state_id: usize,
        offset: Vector,
        /* Whether a MoveState already fired; release-without-move turns
         * the drag into click tracking so a second press can detect a
         * double-click rename. */
        moved: bool,
    },
    /* Arrow-tool viewport pan: scroll anchor plus the press position. */
    Panning {
        origin_scroll: Vector,
        cursor_start: Point,
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
        scroll: Vector,
    ) -> Geometry {
        let mut frame = Frame::new(renderer, bounds.size());

        if let Some(cursor_position) = cursor.position_in(bounds) {
            match *self {
                PendingTransition::Start { from_point, .. } => {
                    // The stored anchor is a world position; convert it to
                    // viewport coordinates for drawing.
                    let start = from_point - scroll;
                    let line = Path::line(start, cursor_position);
                    frame.stroke(
                        &line,
                        Stroke::default()
                            .with_width(2.0)
                            .with_color(theme::CREAM),
                    );
                }
                PendingTransition::Dragging { state_id, offset, .. } => {
                    let drag_position = cursor_position - offset;
                    let node = states.iter().find(|s| s.id == state_id);

                    if let Some(node) = node {
                        frame.fill(
                            &Path::circle(drag_position, node.radius),
                            iced::Color { a: 0.5, ..theme::TEAL },
                        );
                        frame.stroke(
                            &Path::circle(drag_position, node.radius),
                            Stroke::default()
                                .with_width(2.0)
                                .with_color(theme::CREAM),
                        );
                    }
                }
                // Panning has no overlay of its own; the world content
                // moves through the emitted scroll updates.
                PendingTransition::Panning { .. } => {}
                PendingTransition::ClickTracking { .. } => {
                }
            }
        }
        frame.into_geometry()
    }
}
