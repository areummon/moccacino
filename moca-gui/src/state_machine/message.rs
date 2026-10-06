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
    Scrolled(Vector),
    Zoomed { zoom: f32, scroll: Vector },
    Viewport(Size),
    RequestTransitionLabel {
        from_state_id: usize,
        to_state_id: usize,
        from_point: Point,
        to_point: Point,
    },
}
