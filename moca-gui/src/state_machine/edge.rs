/* Geometry of one drawn transition (all labels between an ordered pair of
 * states): the shaft curve, the arrowhead and the label pills. Drawing,
 * hit-testing and hover highlighting all derive from this single source,
 * so what you see is exactly what you can click. Coordinates are world
 * coordinates. */

use iced::widget::canvas::{Path, path};
use iced::{Point, Rectangle, Size, Vector};

use super::node::StateNode;
use super::util::VectorExt;

pub(crate) const LABEL_FONT_SIZE: f32 = 13.0;
pub(crate) const LABEL_HEIGHT: f32 = 20.0;
const LABEL_GAP: f32 = 3.0;
/* JetBrains Mono advances 0.6em per glyph, so label widths are exact. */
const LABEL_CHAR_WIDTH: f32 = LABEL_FONT_SIZE * 0.6;
const LABEL_PADDING_X: f32 = 8.0;
/* Distance between the shaft and the nearest edge of the label block. */
const LABEL_CLEARANCE: f32 = 5.0;
const ARROW_LENGTH: f32 = 11.0;
const ARROW_HALF_WIDTH: f32 = 4.8;
/* How close (world units) a click must land to the shaft to hit it. */
const SHAFT_HIT_DISTANCE: f32 = 7.0;

#[derive(Debug, Clone, Copy)]
enum Shaft {
    Line { start: Point, end: Point },
    Quad { start: Point, control: Point, end: Point },
    Cubic { start: Point, c1: Point, c2: Point, end: Point },
}

impl Shaft {
    fn point_at(&self, t: f32) -> Point {
        let u = 1.0 - t;
        match *self {
            Shaft::Line { start, end } => start + (end - start) * t,
            Shaft::Quad { start, control, end } => Point::new(
                u * u * start.x + 2.0 * u * t * control.x + t * t * end.x,
                u * u * start.y + 2.0 * u * t * control.y + t * t * end.y,
            ),
            Shaft::Cubic { start, c1, c2, end } => Point::new(
                u * u * u * start.x + 3.0 * u * u * t * c1.x + 3.0 * u * t * t * c2.x + t * t * t * end.x,
                u * u * u * start.y + 3.0 * u * u * t * c1.y + 3.0 * u * t * t * c2.y + t * t * t * end.y,
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct EdgeGeometry {
    shaft: Shaft,
    /* Arrow tip (on the target's boundary) and the unit travel direction
     * arriving there. */
    tip: Point,
    direction: Vector,
    /* Point on the shaft the label block hangs off, and the unit normal
     * pointing to the side the labels sit on. */
    label_anchor: Point,
    label_normal: Vector,
}

impl EdgeGeometry {
    /* `has_reverse` bends both directions of a two-way pair apart so
     * neither hides the other; `loop_direction` is the unit vector a
     * self-loop points along (away from the state's other edges). */
    pub(crate) fn new(from: &StateNode, to: &StateNode, has_reverse: bool, loop_direction: Vector) -> Self {
        if from.id == to.id {
            return Self::self_loop(from, loop_direction);
        }
        let delta = to.position - from.position;
        if has_reverse {
            // Both directions bend to the same absolute side of their own
            // travel direction, so the pair separates into two arcs.
            let perpendicular = Vector::new(-delta.y, delta.x).unit();
            let midpoint = from.position + delta * 0.5;
            let control = midpoint + perpendicular * (delta.length() * 0.22).max(24.0);
            let start = from.position + (control - from.position).unit() * from.radius;
            let tip = to.position - (to.position - control).unit() * to.radius;
            let shaft = Shaft::Quad { start, control, end: tip };
            EdgeGeometry {
                shaft,
                tip,
                direction: (tip - control).unit(),
                label_anchor: shaft.point_at(0.5),
                label_normal: perpendicular,
            }
        } else {
            let unit = delta.unit();
            let start = from.position + unit * from.radius;
            let tip = to.position - unit * to.radius;
            EdgeGeometry {
                shaft: Shaft::Line { start, end: tip },
                tip,
                direction: unit,
                label_anchor: start + (tip - start) * 0.5,
                label_normal: Vector::new(unit.y, -unit.x),
            }
        }
    }

    /* A teardrop pointing along `direction`: it leaves the state a little
     * clockwise of that direction and returns a little counterclockwise
     * (for the default upward loop: out at the upper right, back in at the
     * upper left). */
    fn self_loop(node: &StateNode, direction: Vector) -> Self {
        const SPREAD: f32 = 0.52;
        const LEAN: f32 = 0.42;
        let center = node.position;
        let r = node.radius;
        let axis = direction.y.atan2(direction.x);
        let polar = |angle: f32| Vector::new(angle.cos(), angle.sin());
        let start = center + polar(axis + SPREAD) * r;
        let tip = center + polar(axis - SPREAD) * r;
        let c1 = start + polar(axis + LEAN) * (r * 2.2);
        let c2 = tip + polar(axis - LEAN) * (r * 2.2);
        let shaft = Shaft::Cubic { start, c1, c2, end: tip };
        EdgeGeometry {
            shaft,
            tip,
            direction: (tip - c2).unit(),
            label_anchor: shaft.point_at(0.5),
            label_normal: polar(axis),
        }
    }

    /* The shaft, stopping just short of the tip so the stroke never pokes
     * through the filled arrowhead. */
    pub(crate) fn shaft_path(&self) -> Path {
        let end = self.tip - self.direction * (ARROW_LENGTH * 0.6);
        let mut builder = path::Builder::new();
        match self.shaft {
            Shaft::Line { start, .. } => {
                builder.move_to(start);
                builder.line_to(end);
            }
            Shaft::Quad { start, control, .. } => {
                builder.move_to(start);
                builder.quadratic_curve_to(control, end);
            }
            Shaft::Cubic { start, c1, c2, .. } => {
                builder.move_to(start);
                builder.bezier_curve_to(c1, c2, end);
            }
        }
        builder.build()
    }

    pub(crate) fn arrowhead_path(&self) -> Path {
        let back = self.tip - self.direction * ARROW_LENGTH;
        let side = Vector::new(-self.direction.y, self.direction.x) * ARROW_HALF_WIDTH;
        Path::new(|b| {
            b.move_to(self.tip);
            b.line_to(back + side);
            b.line_to(back - side);
            b.close();
        })
    }

    /* One rectangle per label, stacked in label order, placed beside the
     * shaft on the normal side. */
    pub(crate) fn label_rects<'l>(&self, labels: impl IntoIterator<Item = &'l String>) -> Vec<(Rectangle, &'l str)> {
        let labels: Vec<&str> = labels.into_iter().map(String::as_str).collect();
        if labels.is_empty() {
            return Vec::new();
        }
        let widths: Vec<f32> = labels.iter().map(|label| label_width(label)).collect();
        let block_width = widths.iter().cloned().fold(0.0, f32::max);
        let block_height =
            labels.len() as f32 * LABEL_HEIGHT + (labels.len() - 1) as f32 * LABEL_GAP;
        // Push the block out along the normal until its nearest side
        // clears the shaft.
        let n = self.label_normal;
        let reach = n.x.abs() * block_width / 2.0 + n.y.abs() * block_height / 2.0;
        let center = self.label_anchor + n * (LABEL_CLEARANCE + reach);
        let mut top = center.y - block_height / 2.0;
        labels
            .into_iter()
            .zip(widths)
            .map(|(label, width)| {
                let rect = Rectangle::new(
                    Point::new(center.x - width / 2.0, top),
                    Size::new(width, LABEL_HEIGHT),
                );
                top += LABEL_HEIGHT + LABEL_GAP;
                (rect, label)
            })
            .collect()
    }

    /* Whether a world point hits this edge: on a label pill, or close
     * enough to the shaft. */
    pub(crate) fn hit<'l>(&self, point: Point, labels: impl IntoIterator<Item = &'l String>) -> bool {
        if self
            .label_rects(labels)
            .iter()
            .any(|(rect, _)| rect.expand(2.0).contains(point))
        {
            return true;
        }
        self.distance_to_shaft(point) <= SHAFT_HIT_DISTANCE
    }

    fn distance_to_shaft(&self, point: Point) -> f32 {
        const SAMPLES: usize = 24;
        let mut best = f32::MAX;
        let mut previous = self.shaft.point_at(0.0);
        for i in 1..=SAMPLES {
            let current = self.shaft.point_at(i as f32 / SAMPLES as f32);
            best = best.min(distance_to_segment(point, previous, current));
            previous = current;
        }
        best
    }
}

pub(crate) fn label_width(label: &str) -> f32 {
    (label.chars().count() as f32 * LABEL_CHAR_WIDTH + 2.0 * LABEL_PADDING_X).max(LABEL_HEIGHT)
}

fn distance_to_segment(point: Point, a: Point, b: Point) -> f32 {
    let ab = b - a;
    let length_squared = ab.x * ab.x + ab.y * ab.y;
    if length_squared <= f32::EPSILON {
        return (point - a).length();
    }
    let ap = point - a;
    let t = ((ap.x * ab.x + ap.y * ab.y) / length_squared).clamp(0.0, 1.0);
    (point - (a + ab * t)).length()
}
