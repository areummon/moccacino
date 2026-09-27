/* Small vector icons drawn on a canvas. The bundled fonts lack most UI
 * symbols (play, pause, sun, moon...), so icons are drawn from simple
 * paths in a 24×24 design grid and scaled to the requested size. Colors
 * come from the active palette at draw time, so icons follow the theme. */

use std::cell::Cell;

use iced::mouse;
use iced::widget::canvas::{self, path, Frame, Geometry, LineCap, LineJoin, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::gui::theme::{self, Family, Palette};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Icon {
    Logo,
    Play,
    Pause,
    Step,
    Reset,
    Sun,
    Moon,
    ChevronDown,
    ChevronUp,
    Plus,
    Minus,
    Close,
    Select,
    State,
    Transition,
    Delete,
    Fit,
    Check,
    Warn,
    Keyboard,
    Folder,
    Family(Family),
}

/* Which palette color an icon is painted with. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ink {
    Text,
    Dim,
    Faint,
    Accent,
    OnAccent,
    Success,
    Danger,
}

impl Ink {
    fn color(self, p: &Palette) -> Color {
        match self {
            Ink::Text => p.text,
            Ink::Dim => p.text_dim,
            Ink::Faint => p.text_faint,
            Ink::Accent => p.accent_strong,
            Ink::OnAccent => p.accent_text,
            Ink::Success => p.success,
            Ink::Danger => p.danger,
        }
    }
}

struct IconProgram {
    icon: Icon,
    ink: Ink,
}

pub(crate) fn icon<'a, Message: 'a>(icon: Icon, size: f32, ink: Ink) -> Element<'a, Message> {
    canvas::Canvas::new(IconProgram { icon, ink })
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .into()
}

/* Per-icon tessellation cache. Widget state is kept by tree position, so a
 * slot can switch icons (Play → Pause, Moon → Sun) or inks: the cache is
 * keyed on what it drew and rebuilt when that changes (size changes are
 * handled by the cache itself). */
#[derive(Default)]
pub(crate) struct IconCache {
    cache: canvas::Cache,
    key: Cell<Option<(Icon, Ink, bool)>>,
}

impl<Message> canvas::Program<Message> for IconProgram {
    type State = IconCache;

    fn draw(
        &self,
        state: &IconCache,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let p = theme::palette(theme);
        let key = (self.icon, self.ink, theme.extended_palette().is_dark);
        if state.key.replace(Some(key)) != Some(key) {
            state.cache.clear();
        }
        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let side = bounds.width.min(bounds.height);
            frame.translate(iced::Vector::new(
                (bounds.width - side) / 2.0,
                (bounds.height - side) / 2.0,
            ));
            frame.scale(side / 24.0);
            paint(frame, self.icon, self.ink.color(p), p);
        });
        vec![geometry]
    }
}

fn pen(color: Color, width: f32) -> Stroke<'static> {
    Stroke::default()
        .with_color(color)
        .with_width(width)
        .with_line_cap(LineCap::Round)
        .with_line_join(LineJoin::Round)
}

fn polyline(points: &[(f32, f32)]) -> Path {
    Path::new(|b| {
        if let Some(&(x, y)) = points.first() {
            b.move_to(Point::new(x, y));
        }
        for &(x, y) in &points[1..] {
            b.line_to(Point::new(x, y));
        }
    })
}

fn polygon(points: &[(f32, f32)]) -> Path {
    Path::new(|b| {
        if let Some(&(x, y)) = points.first() {
            b.move_to(Point::new(x, y));
        }
        for &(x, y) in &points[1..] {
            b.line_to(Point::new(x, y));
        }
        b.close();
    })
}

/* Arc approximated by a polyline, appended to an open builder. Angles are
 * in radians, y pointing down. */
fn arc_into(b: &mut path::Builder, cx: f32, cy: f32, r: f32, start: f32, end: f32, first: bool) {
    const SEGMENTS: usize = 24;
    for i in 0..=SEGMENTS {
        let t = start + (end - start) * i as f32 / SEGMENTS as f32;
        let point = Point::new(cx + r * t.cos(), cy + r * t.sin());
        if i == 0 && first {
            b.move_to(point);
        } else {
            b.line_to(point);
        }
    }
}

fn paint(frame: &mut Frame, icon: Icon, ink: Color, p: &Palette) {
    let stroke = pen(ink, 2.0);
    match icon {
        Icon::Logo => {
            // Two states joined by a transition: the app's mark.
            frame.fill(&Path::circle(Point::new(6.5, 12.0), 4.5), p.family_finite);
            frame.fill(&Path::circle(Point::new(17.5, 12.0), 5.0), p.accent);
            frame.stroke(&Path::circle(Point::new(17.5, 12.0), 3.0), pen(p.accent_text, 1.3));
            frame.stroke(&polyline(&[(11.2, 12.0), (12.0, 12.0)]), pen(ink, 1.6));
        }
        Icon::Play => {
            let shape = polygon(&[(8.0, 5.5), (18.5, 12.0), (8.0, 18.5)]);
            frame.fill(&shape, ink);
            frame.stroke(&shape, pen(ink, 1.5));
        }
        Icon::Pause => {
            for x in [7.0, 13.5] {
                frame.fill(
                    &Path::rounded_rectangle(Point::new(x, 5.5), Size::new(3.5, 13.0), 1.2.into()),
                    ink,
                );
            }
        }
        Icon::Step => {
            let shape = polygon(&[(6.0, 6.0), (14.5, 12.0), (6.0, 18.0)]);
            frame.fill(&shape, ink);
            frame.stroke(&shape, pen(ink, 1.5));
            frame.fill(
                &Path::rounded_rectangle(Point::new(15.8, 6.0), Size::new(2.8, 12.0), 1.0.into()),
                ink,
            );
        }
        Icon::Reset => {
            let arc = Path::new(|b| arc_into(b, 12.0, 12.5, 6.5, -1.1, 4.0, true));
            frame.stroke(&arc, stroke);
            // Arrow head at the arc start, pointing clockwise.
            let (sx, sy) = (12.0 + 6.5 * (-1.1f32).cos(), 12.5 + 6.5 * (-1.1f32).sin());
            let head = polygon(&[(sx + 3.6, sy - 1.6), (sx - 0.6, sy - 3.4), (sx + 0.6, sy + 1.4)]);
            frame.fill(&head, ink);
            frame.stroke(&head, pen(ink, 1.2));
        }
        Icon::Sun => {
            frame.stroke(&Path::circle(Point::new(12.0, 12.0), 4.0), stroke);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::FRAC_PI_4;
                let (c, s) = (a.cos(), a.sin());
                frame.stroke(
                    &polyline(&[(12.0 + 7.0 * c, 12.0 + 7.0 * s), (12.0 + 9.0 * c, 12.0 + 9.0 * s)]),
                    stroke,
                );
            }
        }
        Icon::Moon => {
            // Crescent: outer arc of the moon disc, inner arc of the
            // shadow disc, meeting at their two intersection points.
            let moon = Path::new(|b| {
                arc_into(b, 12.0, 12.0, 8.0, 0.2393, 4.4728, true);
                arc_into(b, 16.0, 8.0, 7.0, 3.7104, 1.0027, false);
                b.close();
            });
            frame.fill(&moon, ink);
        }
        Icon::ChevronDown => frame.stroke(&polyline(&[(7.0, 10.0), (12.0, 15.0), (17.0, 10.0)]), stroke),
        Icon::ChevronUp => frame.stroke(&polyline(&[(7.0, 14.5), (12.0, 9.5), (17.0, 14.5)]), stroke),
        Icon::Plus => {
            frame.stroke(&polyline(&[(12.0, 5.5), (12.0, 18.5)]), stroke);
            frame.stroke(&polyline(&[(5.5, 12.0), (18.5, 12.0)]), stroke);
        }
        Icon::Minus => frame.stroke(&polyline(&[(5.5, 12.0), (18.5, 12.0)]), stroke),
        Icon::Close => {
            frame.stroke(&polyline(&[(7.0, 7.0), (17.0, 17.0)]), stroke);
            frame.stroke(&polyline(&[(17.0, 7.0), (7.0, 17.0)]), stroke);
        }
        Icon::Select => {
            let pointer = polygon(&[
                (7.0, 4.0),
                (7.0, 18.5),
                (10.6, 15.0),
                (13.2, 20.2),
                (15.6, 19.0),
                (13.0, 13.9),
                (18.0, 13.6),
            ]);
            frame.fill(&pointer, ink);
            frame.stroke(&pointer, pen(ink, 1.2));
        }
        Icon::State => {
            frame.stroke(&Path::circle(Point::new(12.0, 12.0), 7.0), stroke);
            frame.fill(&Path::circle(Point::new(12.0, 12.0), 2.2), ink);
        }
        Icon::Transition => {
            frame.fill(&Path::circle(Point::new(5.0, 17.0), 2.4), ink);
            let curve = Path::new(|b| {
                b.move_to(Point::new(6.8, 15.2));
                b.quadratic_curve_to(Point::new(9.0, 6.0), Point::new(17.5, 7.0));
            });
            frame.stroke(&curve, stroke);
            let head = polygon(&[(20.0, 7.2), (15.6, 4.2), (16.0, 10.0)]);
            frame.fill(&head, ink);
            frame.stroke(&head, pen(ink, 1.2));
        }
        Icon::Delete => {
            frame.stroke(&polyline(&[(4.5, 7.0), (19.5, 7.0)]), stroke);
            frame.stroke(&polyline(&[(9.5, 7.0), (9.5, 4.8), (14.5, 4.8), (14.5, 7.0)]), stroke);
            frame.stroke(&polyline(&[(6.5, 7.0), (7.6, 19.5), (16.4, 19.5), (17.5, 7.0)]), stroke);
            frame.stroke(&polyline(&[(10.3, 10.5), (10.3, 16.0)]), pen(ink, 1.6));
            frame.stroke(&polyline(&[(13.7, 10.5), (13.7, 16.0)]), pen(ink, 1.6));
        }
        Icon::Fit => {
            for &(x, y, dx, dy) in &[
                (5.0, 5.0, 1.0, 1.0),
                (19.0, 5.0, -1.0, 1.0),
                (5.0, 19.0, 1.0, -1.0),
                (19.0, 19.0, -1.0, -1.0),
            ] {
                frame.stroke(
                    &polyline(&[(x, y + 5.0 * dy), (x, y), (x + 5.0 * dx, y)]),
                    stroke,
                );
            }
            frame.fill(&Path::circle(Point::new(12.0, 12.0), 2.2), ink);
        }
        Icon::Check => frame.stroke(&polyline(&[(5.5, 12.5), (10.0, 17.0), (18.5, 7.5)]), pen(ink, 2.4)),
        Icon::Warn => {
            frame.stroke(&polygon(&[(12.0, 4.0), (21.0, 19.5), (3.0, 19.5)]), stroke);
            frame.stroke(&polyline(&[(12.0, 9.5), (12.0, 14.0)]), stroke);
            frame.fill(&Path::circle(Point::new(12.0, 16.8), 1.2), ink);
        }
        Icon::Keyboard => {
            frame.stroke(
                &Path::rounded_rectangle(Point::new(3.0, 6.5), Size::new(18.0, 11.0), 2.5.into()),
                stroke,
            );
            for (x, y) in [(7.0, 10.0), (10.3, 10.0), (13.7, 10.0), (17.0, 10.0)] {
                frame.fill(&Path::circle(Point::new(x, y), 0.9), ink);
            }
            frame.stroke(&polyline(&[(8.0, 14.0), (16.0, 14.0)]), pen(ink, 1.6));
        }
        Icon::Folder => {
            frame.stroke(
                &polygon(&[(3.5, 7.0), (3.5, 18.5), (20.5, 18.5), (20.5, 8.5), (11.5, 8.5), (9.5, 6.0), (4.5, 6.0)]),
                stroke,
            );
        }
        Icon::Family(family) => paint_family(frame, family, ink),
    }
}

/* Family glyphs for the startup cards and dock headers. */
fn paint_family(frame: &mut Frame, family: Family, ink: Color) {
    let stroke = pen(ink, 1.8);
    match family {
        Family::Finite => {
            frame.stroke(&Path::circle(Point::new(6.0, 13.0), 3.5), stroke);
            frame.stroke(&Path::circle(Point::new(18.0, 13.0), 3.5), stroke);
            frame.stroke(&Path::circle(Point::new(18.0, 13.0), 1.8), pen(ink, 1.2));
            frame.stroke(&polyline(&[(9.8, 13.0), (13.6, 13.0)]), stroke);
            frame.fill(&polygon(&[(14.4, 13.0), (12.2, 11.4), (12.2, 14.6)]), ink);
        }
        Family::Pushdown => {
            for (i, y) in [6.0, 10.8, 15.6].iter().enumerate() {
                let width = 12.0 - i as f32 * 0.0;
                frame.stroke(
                    &Path::rounded_rectangle(Point::new(12.0 - width / 2.0, *y), Size::new(width, 3.6), 1.2.into()),
                    stroke,
                );
            }
            frame.stroke(&polyline(&[(20.0, 4.5), (20.0, 9.0)]), stroke);
            frame.fill(&polygon(&[(20.0, 3.0), (18.3, 5.6), (21.7, 5.6)]), ink);
        }
        Family::Turing => {
            for x in [3.0, 9.0, 15.0] {
                frame.stroke(
                    &Path::rounded_rectangle(Point::new(x, 10.0), Size::new(6.0, 7.0), 1.0.into()),
                    stroke,
                );
            }
            frame.fill(&polygon(&[(12.0, 8.2), (9.8, 4.8), (14.2, 4.8)]), ink);
        }
        Family::Grammar => {
            frame.stroke(&polyline(&[(4.0, 7.0), (8.0, 7.0)]), stroke);
            frame.stroke(&polyline(&[(10.0, 7.0), (13.0, 7.0)]), stroke);
            frame.fill(&polygon(&[(14.5, 7.0), (12.5, 5.4), (12.5, 8.6)]), ink);
            frame.stroke(&polyline(&[(16.0, 7.0), (20.0, 7.0)]), stroke);
            frame.stroke(&polyline(&[(4.0, 12.0), (20.0, 12.0)]), pen(ink, 1.4));
            frame.stroke(&polyline(&[(4.0, 17.0), (14.0, 17.0)]), pen(ink, 1.4));
        }
    }
}
