use iced::time::Instant;
use iced::{Point, Vector};

#[derive(Debug, Clone, Copy)]
pub(crate) enum Gesture {
    Connecting {
        from_state_id: usize,
        from_point: Point,
    },
    Dragging {
        state_id: usize,
        offset: Vector,
        moved: bool,
    },
    Panning {
        origin_scroll: Vector,
        cursor_start: Point,
    },
    ClickTracking {
        at: Instant,
        state_id: usize,
    },
}
