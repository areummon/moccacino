use iced::Point;

#[derive(Debug, Clone)]
pub struct Transition {
    pub from_state_id: usize,
    pub to_state_id: usize,
    pub from_point: Point,
    pub to_point: Point,
    pub label: String,
}
