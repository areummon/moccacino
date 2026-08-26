use iced::{Point, Vector};

use super::node::StateNode;
use super::program::StateMachine;
use super::util::VectorExt;

impl StateMachine<'_> {
    pub(crate) fn find_transition_at_point(&self, point: Point) -> Option<usize> {
        for (index, (key, labels)) in self.transitions.iter().enumerate() {
            if let (Some(from_state), Some(to_state)) = (
                self.states.iter().find(|s| s.id == key.0),
                self.states.iter().find(|s| s.id == key.1)
            ) {
                if key.0 == key.1 {
                    let center = from_state.position;
                    let control = Point::new(center.x, center.y - from_state.radius * 3.8);
                    // Calculate loop curve points for label positioning (same as drawing)
                    let r = from_state.radius;
                    let theta = std::f32::consts::PI / 4.0;
                    let start = Point::new(
                        (center.x - r * theta.cos()) + 4.0,
                        (center.y - r * theta.sin()) + 4.0,
                    );
                    let end = Point::new(
                        (center.x + r * theta.cos()) - 4.0,
                        (center.y - r * theta.sin()) - 4.0,
                    );

                    // Calculate label position (same as drawing)
                    let t = 0.5;
                    let one_minus_t = 1.0 - t;
                    let midpoint = Point::new(
                        one_minus_t * one_minus_t * start.x + 2.0 * one_minus_t * t * control.x + t * t * end.x,
                        one_minus_t * one_minus_t * start.y + 2.0 * one_minus_t * t * control.y + t * t * end.y,
                    );
                    let label_pos = Point::new(midpoint.x, midpoint.y - 10.0);

                    let label_count = labels.len().max(1);
                    let label_height = (label_count as f32) * 18.0 + 16.0;
                    let width = 80.0;

                    // Extended rectangular area: covers both the label stack above and the loop curve area below
                    let rect_left = label_pos.x - width / 2.0;
                    let rect_right = label_pos.x + width / 2.0;
                    let rect_top = label_pos.y - label_height; // Start from top of label stack
                    let rect_bottom = label_pos.y + 60.0; // Extend down to cover the loop curve area

                    if point.x >= rect_left && point.x <= rect_right && point.y >= rect_top && point.y <= rect_bottom {
                        return Some(index);
                    }
                } else {
                    // Calculate label position and stack direction
                    let has_reverse = self.transitions.contains_key(&(key.1, key.0));
                    let from_state = from_state;
                    let to_state = to_state;
                    let label_count = labels.len().max(1);
                    let height = (label_count as f32) * 18.0 + 16.0;
                    let width = 80.0;
                    let (label_pos, stack_vec) = if has_reverse {
                        // Curved: use the same logic as drawing
                        let center_to_center = to_state.position - from_state.position;
                        let distance = center_to_center.length();
                        let (node_a_pos, node_b_pos) = if key.0 < key.1 {
                            (from_state.position, to_state.position)
                        } else {
                            (to_state.position, from_state.position)
                        };
                        let consistent_direction = node_b_pos - node_a_pos;
                        let consistent_perpendicular = Vector::new(-consistent_direction.y, consistent_direction.x).unit();
                        let curve_side_multiplier = if key.0 < key.1 { 1.0 } else { -1.0 };
                        let curve_offset = distance * 0.4;
                        let midpoint = Point::new(
                            (from_state.position.x + to_state.position.x) / 2.0,
                            (from_state.position.y + to_state.position.y) / 2.0,
                        );
                        let control_point = midpoint + consistent_perpendicular * curve_offset * curve_side_multiplier;
                        let start_direction = (control_point - from_state.position).unit();
                        let end_direction = (to_state.position - control_point).unit();
                        let start_point = from_state.position + start_direction * from_state.radius;
                        let end_point = to_state.position - end_direction * to_state.radius;
                        let t = 0.5;
                        let one_minus_t = 1.0 - t;
                        let curve_midpoint = Point::new(
                            one_minus_t * one_minus_t * start_point.x + 2.0 * one_minus_t * t * control_point.x + t * t * end_point.x,
                            one_minus_t * one_minus_t * start_point.y + 2.0 * one_minus_t * t * control_point.y + t * t * end_point.y,
                        );
                        let label_offset = consistent_perpendicular * (25.0 * curve_side_multiplier);
                        (curve_midpoint + label_offset, label_offset.unit())
                    } else {
                        // Straight: use -perpendicular_vec as in drawing
                        let direction = to_state.position - from_state.position;
                        let midpoint = Point::new(
                            (from_state.position.x + to_state.position.x) / 2.0,
                            (from_state.position.y + to_state.position.y) / 2.0,
                        );
                        let perpendicular_vec = Vector::new(-direction.y, direction.x).unit() * 15.0;
                        (midpoint - perpendicular_vec, (-perpendicular_vec).unit())
                    };
                    // Rectangle starts at the top of the label stack and extends downward in the stack direction
                    let rect_top_left = label_pos - stack_vec * 0.0 - Vector::new(width / 2.0, 0.0);
                    // Check if point is inside the rectangle (project point onto stack direction)
                    let rel = point - rect_top_left;
                    let stack_proj = rel.x * stack_vec.x + rel.y * stack_vec.y;
                    let ortho_vec = Vector::new(-stack_vec.y, stack_vec.x);
                    let ortho_proj = rel.x * ortho_vec.x + rel.y * ortho_vec.y;
                    if stack_proj >= 0.0 && stack_proj <= height && ortho_proj >= 0.0 && ortho_proj <= width {
                        return Some(index);
                    }

                    // For curved transitions, also check distance to the curve itself
                    if has_reverse {
                        let center_to_center = to_state.position - from_state.position;
                        let distance = center_to_center.length();
                        let (node_a_pos, node_b_pos) = if key.0 < key.1 {
                            (from_state.position, to_state.position)
                        } else {
                            (to_state.position, from_state.position)
                        };
                        let consistent_direction = node_b_pos - node_a_pos;
                        let consistent_perpendicular = Vector::new(-consistent_direction.y, consistent_direction.x).unit();
                        let curve_side_multiplier = if key.0 < key.1 { 1.0 } else { -1.0 };
                        let curve_offset = distance * 0.4;
                        let midpoint = Point::new(
                            (from_state.position.x + to_state.position.x) / 2.0,
                            (from_state.position.y + to_state.position.y) / 2.0,
                        );
                        let control_point = midpoint + consistent_perpendicular * curve_offset * curve_side_multiplier;
                        let start_direction = (control_point - from_state.position).unit();
                        let end_direction = (to_state.position - control_point).unit();
                        let start_point = from_state.position + start_direction * from_state.radius;
                        let end_point = to_state.position - end_direction * to_state.radius;

                        // Check distance to the curve at multiple points
                        for t in [0.2, 0.4, 0.6, 0.8] {
                            let one_minus_t = 1.0 - t;
                            let curve_point = Point::new(
                                one_minus_t * one_minus_t * start_point.x + 2.0 * one_minus_t * t * control_point.x + t * t * end_point.x,
                                one_minus_t * one_minus_t * start_point.y + 2.0 * one_minus_t * t * control_point.y + t * t * end_point.y,
                            );
                            let curve_distance = (point - curve_point).length();
                            if curve_distance <= 25.0 {
                                return Some(index);
                            }
                        }
                    } else {
                        // For straight transitions, allow clicking near the line (legacy behavior)
                        let direction = to_state.position - from_state.position;
                        let direction_unit = direction.unit();
                        let start_point = from_state.position + direction_unit * from_state.radius;
                        let end_point = to_state.position - direction_unit * to_state.radius;
                        let line_midpoint = Point::new((start_point.x + end_point.x) / 2.0, (start_point.y + end_point.y) / 2.0);
                        let line_distance = (point - line_midpoint).length();
                        if line_distance <= 20.0 {
                            return Some(index);
                        }
                    }
                }
            }
        }
        None
    }

    #[allow(dead_code)]
    pub(crate) fn calculate_transition_label_position(&self, transition: &(usize, usize), from_state: &StateNode, to_state: &StateNode) -> Point {
        let has_reverse = self.transitions.iter().any(|other_trans|
            other_trans.0.0 == transition.1 &&
            other_trans.0.1 == transition.0
        );

        if has_reverse {
            let center_to_center = to_state.position - from_state.position;
            let distance = center_to_center.length();
            let (node_a_pos, node_b_pos) = if transition.0 < transition.1 {
                (from_state.position, to_state.position)
            } else {
                (to_state.position, from_state.position)
            };
            let consistent_direction = node_b_pos - node_a_pos;
            let consistent_perpendicular = Vector::new(-consistent_direction.y, consistent_direction.x).unit();
            let curve_side_multiplier = if transition.0 < transition.1 { 1.0 } else { -1.0 };
            let curve_offset = distance * 0.4;
            let midpoint = Point::new(
                (from_state.position.x + to_state.position.x) / 2.0,
                (from_state.position.y + to_state.position.y) / 2.0,
            );
            let control_point = midpoint + consistent_perpendicular * curve_offset * curve_side_multiplier;
            let label_position = self.calculate_curve_midpoint(from_state.position, control_point, to_state.position);
            let label_offset = consistent_perpendicular * (25.0 * curve_side_multiplier);
            label_position + label_offset
        } else {
            let direction = to_state.position - from_state.position;
            let midpoint = Point::new(
                (from_state.position.x + to_state.position.x) / 2.0,
                (from_state.position.y + to_state.position.y) / 2.0,
            );
            let perpendicular_vec = Vector::new(-direction.y, direction.x).unit() * 15.0;
            midpoint + perpendicular_vec
        }
    }

    #[allow(dead_code)]
    fn calculate_curve_midpoint(&self, start: Point, control: Point, end: Point) -> Point {
        let t = 0.5;
        let one_minus_t = 1.0 - t;
        Point::new(
            one_minus_t * one_minus_t * start.x + 2.0 * one_minus_t * t * control.x + t * t * end.x,
            one_minus_t * one_minus_t * start.y + 2.0 * one_minus_t * t * control.y + t * t * end.y,
        )
    }
}
