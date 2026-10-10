use iced::{Point, Vector};

#[derive(Debug, Clone, Copy)]
pub(crate) enum PendingTransition {
    Start {
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
        last_click_time: iced::time::Instant,
        last_clicked_state: Option<usize>,
        last_clicked_transition: Option<(usize, usize)>,
    },
}
