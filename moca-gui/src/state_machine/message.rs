use iced::{Point, Vector};

use super::node::StateNode;
use super::transition::Transition;

#[derive(Debug, Clone)]
pub enum CanvasMessage {
    AddState(StateNode),
    AddTransition(Transition),
    MoveState { state_id: usize, new_position: Point },
    StateClicked(usize),
    StateDoubleClicked(usize),
    TransitionDoubleClicked(usize),
    TransitionClicked(usize),
    /* New viewport scroll offset (world position of the canvas top-left),
     * already clamped by the canvas program. */
    Scrolled(Vector),
    RequestTransitionLabel {
        from_state_id: usize,
        to_state_id: usize,
        from_point: Point,
        to_point: Point,
    },
}
