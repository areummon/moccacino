//! Layered, colored icons for the four machine families that morph when hot.
//!
//! Each icon is drawn on a 31×29 design grid (y from -2) as a mid-tone back
//! piece, white content and light front pieces, all with a 1-unit outside
//! stroke, a drop shadow and a top glow; every colour derives from the
//! family colour. `p` runs from 0 (rest) to 1 (hot) over 460 ms.

use std::cell::Cell;
use std::f32::consts::PI;

use iced::advanced::graphics::geometry::Renderer as _;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::event::{self, Event};
use iced::time::Instant;
use iced::widget::canvas::{self, gradient, Frame, LineCap, LineJoin, Path, Stroke};
use iced::{mouse, window, Color, Element, Length, Point, Rectangle, Renderer, Size, Theme, Vector};

use super::icons::arc_into;
use super::motion::ease_out;
use crate::family::Family;
use crate::theme;

const GRID_W: f32 = 31.0;
const GRID_H: f32 = 29.0;
const GRID_TOP: f32 = -2.0;
const DURATION: f32 = 0.46;

/// `hot` holds the icon in its morphed pose (e.g. the selected card); hovering
/// the icon itself does too.
pub(crate) fn feature_icon<'a, Message: 'a>(family: Family, size: f32, hot: bool) -> Element<'a, Message> {
    Element::new(FeatureIcon { family, size, hot })
}

struct FeatureIcon {
    family: Family,
    size: f32,
    hot: bool,
}

struct Motion {
    value: f32,
    from: f32,
    target: f32,
    start: Option<Instant>,
    hovered: bool,
    cache: canvas::Cache,
    drawn: Cell<Option<(u32, Family, bool)>>,
}

impl Motion {
    /// Restarts from the value on screen, so a reversal mid-flight never jumps.
    fn retarget(&mut self, target: f32, now: Instant) {
        if target != self.target {
            self.from = self.value;
            self.target = target;
            self.start = Some(now);
        }
    }

    fn advance(&mut self, now: Instant) -> bool {
        let Some(start) = self.start else { return false };
        let t = (now.saturating_duration_since(start).as_secs_f32() / DURATION).min(1.0);
        self.value = self.from + (self.target - self.from) * ease_out(t);
        if t >= 1.0 {
            self.start = None;
        }
        self.start.is_some()
    }
}

impl<Message> Widget<Message, Theme, Renderer> for FeatureIcon {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Motion>()
    }

    fn state(&self) -> tree::State {
        let value = if self.hot { 1.0 } else { 0.0 };
        tree::State::new(Motion {
            value,
            from: value,
            target: value,
            start: None,
            hovered: false,
            cache: canvas::Cache::new(),
            drawn: Cell::new(None),
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.size), Length::Fixed(self.size))
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::atomic(limits, self.size, self.size)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let motion = tree.state.downcast_mut::<Motion>();
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorLeft) => {
                let hovered = cursor.is_over(layout.bounds());
                if hovered != motion.hovered {
                    motion.hovered = hovered;
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                }
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                motion.retarget(if self.hot || motion.hovered { 1.0 } else { 0.0 }, now);
                if motion.advance(now) {
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                }
            }
            _ => {}
        }
        event::Status::Ignored
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let motion = tree.state.downcast_ref::<Motion>();
        let key = (motion.value.to_bits(), self.family, theme.extended_palette().is_dark);
        if motion.drawn.replace(Some(key)) != Some(key) {
            motion.cache.clear();
        }
        let geometry = motion.cache.draw(renderer, bounds.size(), |frame| {
            let scale = bounds.width / GRID_W;
            frame.translate(Vector::new(0.0, (bounds.height - GRID_H * scale) / 2.0 - GRID_TOP * scale));
            frame.scale(scale);
            let ink = Ink::of(theme::family_color(theme::palette(theme), self.family));
            paint(frame, self.family, &ink, motion.value);
        });
        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| renderer.draw_geometry(geometry));
    }
}

/// The icon colours, derived from one main colour the way every stop of the
/// reference set (#5393FF main) relates to its reference.
struct Ink {
    back: [Color; 2],
    front: [Color; 2],
    stroke: Color,
    deep: Color,
    shadow: Color,
}

impl Ink {
    /// The family colour plays the back piece (#93BBFF), so the main colour is
    /// solved backwards from it; the icon then sits in the app's family pills.
    fn of(family: Color) -> Self {
        let solve = |c: f32| ((c - BACK_AMT) / (1.0 - BACK_AMT)).clamp(0.0, 1.0);
        let main = Color::from_rgb(solve(family.r), solve(family.g), solve(family.b));
        let tone = |source| derive(main, source);
        Ink {
            back: [tone(0x93BBFF), tone(0x81B0FF)],
            front: [tone(0xCCDFFF), tone(0xB2CEFF)],
            stroke: main,
            deep: tone(0x2F6FE0),
            shadow: Color { a: 1.0, ..tone(0x8AB6FF) },
        }
    }
}

/// How far #93BBFF sits from #5393FF towards white.
const BACK_AMT: f32 = (0x93 - 0x53) as f32 / (255 - 0x53) as f32;

fn derive(main: Color, source: u32) -> Color {
    const REFERENCE: [f32; 3] = [83.0, 147.0, 255.0];
    let channel = |shift: u32| ((source >> shift) & 0xff) as f32;
    let amt = |value: f32, reference: f32| (value - reference) / (255.0 - reference);
    let red = amt(channel(16), REFERENCE[0]);
    let shift = |m: f32, a: f32| if a >= 0.0 { m + (1.0 - m) * a } else { m * (1.0 + a) };
    Color::from_rgb(
        shift(main.r, red),
        shift(main.g, amt(channel(8), REFERENCE[1])),
        // The reference blue is saturated, so blue follows red's offset.
        shift(main.b, red),
    )
}

fn pen(color: Color, width: f32) -> Stroke<'static> {
    Stroke::default()
        .with_color(color)
        .with_width(width)
        .with_line_cap(LineCap::Round)
        .with_line_join(LineJoin::Round)
}

fn rounded(x: f32, y: f32, w: f32, h: f32, r: f32) -> Path {
    Path::rounded_rectangle(Point::new(x, y), Size::new(w, h), r.into())
}

fn line(from: (f32, f32), to: (f32, f32)) -> Path {
    Path::line(Point::new(from.0, from.1), Point::new(to.0, to.1))
}

fn alpha(color: Color, a: f32) -> Color {
    Color { a, ..color }
}

const WHITE: Color = Color::WHITE;

enum Tint {
    Back,
    Front,
}

/// One coloured layer: shadow, outside stroke (drawn 2 wide, its inner half
/// hidden by the fill), vertical gradient from `top` to `bottom`, top glow.
fn piece(frame: &mut Frame, ink: &Ink, shape: &Path, outline: Option<&Path>, (top, bottom): (f32, f32), tint: Tint, glow: Option<Path>) {
    frame.with_save(|f| {
        f.translate(Vector::new(0.0, 2.0));
        f.stroke(shape, pen(alpha(ink.shadow, 0.04), 4.0));
        f.stroke(shape, pen(alpha(ink.shadow, 0.06), 2.0));
        f.fill(shape, alpha(ink.shadow, 0.14));
    });
    match outline {
        Some(outline) => frame.stroke(outline, pen(ink.stroke, 2.0).with_line_cap(LineCap::Butt)),
        None => frame.stroke(shape, pen(ink.stroke, 2.0)),
    }
    let [from, to] = match tint {
        Tint::Back => ink.back,
        Tint::Front => ink.front,
    };
    frame.fill(
        shape,
        gradient::Linear::new(Point::new(0.0, top), Point::new(0.0, bottom))
            .add_stop(0.0, from)
            .add_stop(1.0, to),
    );
    if let Some(glow) = glow {
        frame.stroke(&glow, pen(alpha(WHITE, 0.5), 0.7));
    }
}

/// The highlight along a flat top edge, inset by half its width.
fn top_glow(x0: f32, x1: f32, y: f32) -> Path {
    line((x0, y + 0.35), (x1, y + 0.35))
}

fn circle_glow(cx: f32, cy: f32, r: f32) -> Path {
    Path::new(|b| arc_into(b, cx, cy, r - 0.35, 1.2 * PI, 1.8 * PI, true))
}

fn paint(frame: &mut Frame, family: Family, ink: &Ink, p: f32) {
    match family {
        Family::Finite => finite(frame, ink, p),
        Family::Pushdown => pushdown(frame, ink, p),
        Family::Turing => turing(frame, ink, p),
        Family::Grammar => grammar(frame, ink, p),
    }
}

/// A state with its start arrow; hot, the arrow pushes in and the state
/// becomes accepting (its inner ring draws in).
fn finite(frame: &mut Frame, ink: &Ink, p: f32) {
    let (cx, cy, r) = (19.0, 12.5, 8.0);
    let state = Path::circle(Point::new(cx, cy), r);
    piece(frame, ink, &state, None, (cy - r, cy + r), Tint::Back, Some(circle_glow(cx, cy, r)));
    frame.fill(&Path::circle(Point::new(cx, cy), 5.0), WHITE);
    if p > 0.0 {
        frame.stroke(&Path::circle(Point::new(cx, cy), 3.0 * p), pen(ink.deep, 1.6 * p));
    }

    // Rest: 2 units clear of the outside stroke; hot: touching it.
    let tip = 8.0 + 2.0 * p;
    frame.stroke(&line((tip - 5.0, cy), (tip - 2.8, cy)), pen(ink.stroke, 2.2));
    frame.fill(
        &Path::new(|b| {
            b.move_to(Point::new(tip, cy));
            b.line_to(Point::new(tip - 3.2, cy - 2.4));
            b.line_to(Point::new(tip - 3.2, cy + 2.4));
            b.close();
        }),
        ink.stroke,
    );
}

/// An open stack; hot, a block drops in on top.
fn pushdown(frame: &mut Frame, ink: &Ink, p: f32) {
    let (x0, x1, y0, y1, r) = (7.0, 24.0, 6.6, 24.0, 2.2);
    let walls = |b: &mut iced::widget::canvas::path::Builder| {
        b.move_to(Point::new(x0, y0));
        b.line_to(Point::new(x0, y1 - r));
        arc_into(b, x0 + r, y1 - r, r, PI, PI / 2.0, false);
        b.line_to(Point::new(x1 - r, y1));
        arc_into(b, x1 - r, y1 - r, r, PI / 2.0, 0.0, false);
        b.line_to(Point::new(x1, y0));
    };
    let tube = Path::new(|b| {
        walls(b);
        b.close();
    });
    let open = Path::new(walls);
    piece(frame, ink, &tube, Some(&open), (y0, y1), Tint::Back, None);

    let (bx, bw, bh) = (10.0, 11.0, 3.4);
    for y in [17.6, 14.2, 0.5 + 10.3 * p] {
        let block = rounded(bx, y, bw, bh, 1.6);
        piece(frame, ink, &block, None, (y, y + bh), Tint::Front, Some(top_glow(bx + 1.6, bx + bw - 1.6, y)));
    }
}

/// A tape under its read/write head; hot, the head steps right, then writes.
fn turing(frame: &mut Frame, ink: &Ink, p: f32) {
    let (tx, ty, tw, th) = (4.0, 12.0, 23.0, 9.5);
    let tape = rounded(tx, ty, tw, th, 2.2);
    piece(frame, ink, &tape, None, (ty, ty + th), Tint::Back, Some(top_glow(tx + 2.2, tx + tw - 2.2, ty)));

    let written = ((p - 0.45) / 0.55).clamp(0.0, 1.0);
    for (cx, mark) in [(8.5, 0.0), (15.5, 1.0), (22.5, written)] {
        frame.fill(&rounded(cx - 2.5, 14.0, 5.0, 5.5, 1.2), WHITE);
        if mark > 0.0 {
            frame.fill(&Path::circle(Point::new(cx, 16.75), 1.4 * mark), ink.deep);
        }
    }

    // The tip's outside stroke meets the tape's.
    let hx = 15.5 + 7.0 * p;
    let (l, r, top, shoulder, tip) = (hx - 3.6, hx + 3.6, 2.4, 6.6, 10.0);
    let head = Path::new(|b| {
        b.move_to(Point::new(l, top + 1.8));
        b.quadratic_curve_to(Point::new(l, top), Point::new(l + 1.8, top));
        b.line_to(Point::new(r - 1.8, top));
        b.quadratic_curve_to(Point::new(r, top), Point::new(r, top + 1.8));
        b.line_to(Point::new(r, shoulder));
        b.line_to(Point::new(hx, tip));
        b.line_to(Point::new(l, shoulder));
        b.close();
    });
    piece(frame, ink, &head, None, (top, tip), Tint::Front, Some(top_glow(l + 1.8, r - 1.8, top)));
}

/// A derivation tree; hot, the variable expands into two children.
fn grammar(frame: &mut Frame, ink: &Ink, p: f32) {
    let spread = 6.5 * p;
    let (root_y, root_h, child_y, child_w, child_h) = (2.2, 7.8, 16.0, 7.6, 6.8);
    let children = [15.5 - spread, 15.5 + spread];
    for cx in children {
        frame.stroke(&line((15.5, root_y + root_h), (cx, child_y)), pen(ink.stroke, 1.6));
    }
    let root = rounded(11.0, root_y, 9.0, root_h, 2.2);
    piece(frame, ink, &root, None, (root_y, root_y + root_h), Tint::Back, Some(top_glow(13.2, 17.8, root_y)));
    for cx in children {
        let x = cx - child_w / 2.0;
        let child = rounded(x, child_y, child_w, child_h, 1.8);
        piece(frame, ink, &child, None, (child_y, child_y + child_h), Tint::Front, Some(top_glow(x + 1.8, x + child_w - 1.8, child_y)));
    }
}
