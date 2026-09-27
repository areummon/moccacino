use iced::{Point, Vector};

/* In-progress mouse interaction of the canvas, kept in the widget state. */
#[derive(Debug, Clone, Copy)]
pub(crate) enum PendingTransition {
    /* Transition tool: source picked, waiting for the target click. */
    Start {
        from_state_id: usize,
        from_point: Point,
    },
    Dragging {
        state_id: usize,
        /* World offset from the state center to the grab point. */
        offset: Vector,
        /* Whether a MoveState already fired; release-without-move turns
         * the drag into click tracking so a second press can detect a
         * double-click rename. */
        moved: bool,
    },
    /* Arrow-tool viewport pan: scroll anchor plus the press position
     * (viewport coordinates). */
    Panning {
        origin_scroll: Vector,
        cursor_start: Point,
    },
    ClickTracking {
        last_click_time: std::time::Instant,
        last_clicked_state: Option<usize>,
        last_clicked_transition: Option<(usize, usize)>,
    },
}
