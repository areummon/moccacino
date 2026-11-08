use iced::{Point, Size, Vector};

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
    Scrolled(Vector),
    Zoomed { zoom: f32, scroll: Vector },
    Viewport(Size),
}
