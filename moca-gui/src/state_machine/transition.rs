use iced::Point;

/* One labeled transition as handed to the LaTeX exporter and the label
 * dialog; the canvas itself draws from the tab's grouped transition map
 * through `edge::EdgeGeometry`. */
#[derive(Debug, Clone)]
pub struct Transition {
    pub from_state_id: usize,
    pub to_state_id: usize,
    pub from_point: Point,
    pub to_point: Point,
    pub label: String,
}
