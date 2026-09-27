use iced::{Point, Size, Vector};

use super::node::StateNode;
use super::transition::Transition;

#[derive(Debug, Clone)]
pub enum CanvasMessage {
    AddState(StateNode),
    AddTransition(Transition),
    MoveState { state_id: usize, new_position: Point },
    StateClicked(usize),
    StateDoubleClicked(usize),
    TransitionDoubleClicked((usize, usize)),
    TransitionClicked((usize, usize)),
    /* New viewport scroll offset (world position of the canvas top-left),
     * already clamped by the canvas program. */
    Scrolled(Vector),
    /* Ctrl+wheel zoom around the cursor: new zoom plus the scroll that
     * keeps the point under the cursor fixed. */
    Zoomed { zoom: f32, scroll: Vector },
    /* The canvas reports its on-screen size whenever it changes, so zoom
     * buttons and fit-to-content can work without an event of their own. */
    Viewport(Size),
    RequestTransitionLabel {
        from_state_id: usize,
        to_state_id: usize,
        from_point: Point,
        to_point: Point,
    },
}
