use std::collections::HashSet;

use iced::{Point, Size, Vector};

use super::hit_testing::Hit;

#[derive(Debug, Clone)]
pub enum CanvasMessage {
    AddState(Point),
    MoveState { state_id: usize, new_position: Point },
    DeleteState(usize),
    DeleteTransition((usize, usize)),
    ToggleFinal(usize),
    MakeInitial(usize),
    RenameState(usize),
    EditTransition((usize, usize)),
    ConnectStates { from: usize, to: usize },
    Select(HashSet<usize>),
    /// A right-click on `hit`, at a point relative to the canvas.
    ContextMenu { hit: Hit, at: Point },
    Scrolled(Vector),
    Zoomed { zoom: f32, scroll: Vector },
    Viewport(Size),
}
