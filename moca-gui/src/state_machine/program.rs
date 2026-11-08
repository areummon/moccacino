use iced::keyboard::{self, Modifiers};
use iced::time::Instant;
use iced::widget::canvas::{self, Event, Frame, Geometry, LineCap, LineDash, Path, Stroke, Text};
use iced::{alignment, mouse, Color, Point, Rectangle, Renderer, Size, Theme, Vector};
use indexmap::IndexSet;
use std::time::Duration;

use super::canvas::StateMachine;
use super::edge::{EdgeGeometry, LABEL_FONT_SIZE};
use super::gesture::Gesture;
use super::hit_testing::Hit;
use super::message::CanvasMessage;
use super::node::{NodeLook, StateNode};
use super::state::{clamp_scroll, content_bounds};
use super::tool::EditorTool;
use super::util::VectorExt;
use crate::theme::{self, Palette};

const DOUBLE_CLICK: Duration = Duration::from_millis(300);

#[derive(Debug, Default)]
pub(crate) struct Interaction {
    gesture: Option<Gesture>,
    hover: Option<Hit>,
    modifiers: Modifiers,
}

fn changed(a: Vector, b: Vector) -> bool {
    (a.x - b.x).abs() > f32::EPSILON || (a.y - b.y).abs() > f32::EPSILON
}

type Outcome = (canvas::event::Status, Option<CanvasMessage>);

const IGNORED: Outcome = (canvas::event::Status::Ignored, None);

fn captured(message: Option<CanvasMessage>) -> Outcome {
    (canvas::event::Status::Captured, message)
}

impl StateMachine<'_> {
    fn wheel(&self, interaction: &Interaction, delta: mouse::ScrollDelta, anchor: Point, viewport: Size) -> Outcome {
        let scroll = self.state.scroll();
        let zoom = self.state.zoom();
        let (dx, dy) = match delta {
            mouse::ScrollDelta::Lines { x, y } => (x * 40.0, y * 40.0),
            mouse::ScrollDelta::Pixels { x, y } => (x, y),
        };
        if interaction.modifiers.command() {
            let factor = 1.0015f32.powf(dy.clamp(-240.0, 240.0));
            let (zoom, scroll) = self.state.zoom_target(factor, anchor, self.states, viewport);
            return captured(Some(CanvasMessage::Zoomed { zoom, scroll }));
        }
        let (dx, dy) = if interaction.modifiers.shift() && dx == 0.0 { (-dy, 0.0) } else { (dx, dy) };
        let new_scroll = clamp_scroll(
            Vector::new(scroll.x + dx / zoom, scroll.y - dy / zoom),
            self.states,
            viewport,
            zoom,
        );
        captured(changed(new_scroll, scroll).then_some(CanvasMessage::Scrolled(new_scroll)))
    }

    fn press(&self, interaction: &mut Interaction, screen: Point) -> Outcome {
        let world = self.state.to_world(screen);
        let hit = self.hit_test(world);
        let previous = interaction.gesture.take();
        match self.state.active_tool() {
            EditorTool::Arrow => self.press_arrow(interaction, hit, previous, world, screen),
            EditorTool::State => captured(hit.is_none().then_some(CanvasMessage::AddState(world))),
            EditorTool::Transition => self.press_transition(interaction, hit, previous),
            EditorTool::Delete => {
                interaction.hover = None;
                captured(match hit {
                    Some(Hit::State(id)) => Some(CanvasMessage::DeleteState(id)),
                    Some(Hit::Transition(pair)) => Some(CanvasMessage::DeleteTransition(pair)),
                    None => None,
                })
            }
        }
    }

    fn clicked_node(&self, hit: Option<Hit>) -> Option<&StateNode> {
        match hit {
            Some(Hit::State(id)) => self.states.iter().find(|node| node.id == id),
            _ => None,
        }
    }

    fn press_arrow(
        &self,
        interaction: &mut Interaction,
        hit: Option<Hit>,
        previous: Option<Gesture>,
        world: Point,
        screen: Point,
    ) -> Outcome {
        match hit {
            Some(Hit::State(id)) => {
                let now = Instant::now();
                let double = matches!(
                    previous,
                    Some(Gesture::ClickTracking { at, state_id })
                        if state_id == id && now.duration_since(at) < DOUBLE_CLICK
                );
                if double {
                    interaction.gesture = Some(Gesture::ClickTracking { at: now, state_id: id });
                    return captured(Some(CanvasMessage::RenameState(id)));
                }
                if interaction.modifiers.shift() {
                    return captured(Some(CanvasMessage::ToggleFinal(id)));
                }
                if interaction.modifiers.alt() {
                    return captured(Some(CanvasMessage::MakeInitial(id)));
                }
                if let Some(node) = self.clicked_node(hit) {
                    interaction.gesture = Some(Gesture::Dragging {
                        state_id: id,
                        offset: world - node.position,
                        moved: false,
                    });
                }
                captured(None)
            }
            Some(Hit::Transition(pair)) => captured(Some(CanvasMessage::EditTransition(pair))),
            None => {
                interaction.gesture = Some(Gesture::Panning { origin_scroll: self.state.scroll(), cursor_start: screen });
                captured(None)
            }
        }
    }

    fn press_transition(&self, interaction: &mut Interaction, hit: Option<Hit>, previous: Option<Gesture>) -> Outcome {
        match (previous, self.clicked_node(hit)) {
            (Some(Gesture::Connecting { from_state_id, .. }), Some(to_node)) => {
                captured(Some(CanvasMessage::ConnectStates { from: from_state_id, to: to_node.id }))
            }
            (Some(Gesture::Connecting { .. }), None) => captured(None),
            (_, node) => {
                interaction.gesture = node.map(|node| Gesture::Connecting {
                    from_state_id: node.id,
                    from_point: node.position,
                });
                captured(None)
            }
        }
    }

    fn cursor_moved(&self, interaction: &mut Interaction, screen: Option<Point>, viewport: Size) -> Outcome {
        if (viewport.width - self.known_viewport.width).abs() > 0.5
            || (viewport.height - self.known_viewport.height).abs() > 0.5
        {
            return (canvas::event::Status::Ignored, Some(CanvasMessage::Viewport(viewport)));
        }
        let Some(screen) = screen else {
            interaction.hover = None;
            return IGNORED;
        };
        let world = self.state.to_world(screen);
        let scroll = self.state.scroll();
        let zoom = self.state.zoom();
        let healed = clamp_scroll(scroll, self.states, viewport, zoom);
        if changed(healed, scroll) && !matches!(interaction.gesture, Some(Gesture::Dragging { .. })) {
            return captured(Some(CanvasMessage::Scrolled(healed)));
        }
        match interaction.gesture {
            Some(Gesture::Dragging { state_id, offset, .. }) => {
                interaction.gesture = Some(Gesture::Dragging { state_id, offset, moved: true });
                interaction.hover = Some(Hit::State(state_id));
                captured(Some(CanvasMessage::MoveState { state_id, new_position: world - offset }))
            }
            Some(Gesture::Panning { origin_scroll, cursor_start }) => {
                let drag = screen - cursor_start;
                let new_scroll = clamp_scroll(
                    Vector::new(origin_scroll.x - drag.x / zoom, origin_scroll.y - drag.y / zoom),
                    self.states,
                    viewport,
                    zoom,
                );
                captured(changed(new_scroll, scroll).then_some(CanvasMessage::Scrolled(new_scroll)))
            }
            Some(Gesture::Connecting { .. }) => {
                interaction.hover = self.hit_test(world).filter(|hit| matches!(hit, Hit::State(_)));
                captured(None)
            }
            _ => {
                interaction.hover = match self.state.active_tool() {
                    EditorTool::State => None,
                    _ => self.hit_test(world),
                };
                IGNORED
            }
        }
    }

    fn release(interaction: &mut Interaction) -> Outcome {
        match interaction.gesture {
            Some(Gesture::Dragging { state_id, moved, .. }) => {
                interaction.gesture = (!moved).then(|| Gesture::ClickTracking { at: Instant::now(), state_id });
                captured(None)
            }
            Some(Gesture::Panning { .. }) => {
                interaction.gesture = None;
                captured(None)
            }
            _ => IGNORED,
        }
    }
}

impl canvas::Program<CanvasMessage> for StateMachine<'_> {
    type State = Interaction;

    fn update(
        &self,
        interaction: &mut Interaction,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Outcome {
        if let Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) = event {
            interaction.modifiers = modifiers;
            return IGNORED;
        }
        if !self.interactive {
            interaction.hover = None;
            return IGNORED;
        }
        let screen = cursor.position_in(bounds);
        let viewport = bounds.size();
        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => match screen {
                Some(anchor) => self.wheel(interaction, delta, anchor, viewport),
                None => IGNORED,
            },
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => match screen {
                Some(screen) => self.press(interaction, screen),
                None => IGNORED,
            },
            Event::Mouse(mouse::Event::CursorMoved { .. }) => self.cursor_moved(interaction, screen, viewport),
            Event::Mouse(mouse::Event::CursorLeft) => {
                interaction.hover = None;
                IGNORED
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => Self::release(interaction),
            _ => IGNORED,
        }
    }

    fn mouse_interaction(
        &self,
        interaction: &Interaction,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if !self.interactive || !cursor.is_over(bounds) {
            return mouse::Interaction::default();
        }
        match (self.state.active_tool(), interaction.gesture, interaction.hover) {
            (_, Some(Gesture::Dragging { .. }), _) | (_, Some(Gesture::Panning { .. }), _) => {
                mouse::Interaction::Grabbing
            }
            (EditorTool::Delete, _, Some(_)) => mouse::Interaction::Pointer,
            (EditorTool::Delete, _, None) => mouse::Interaction::Crosshair,
            (EditorTool::State, _, _) => mouse::Interaction::Crosshair,
            (EditorTool::Transition, _, Some(Hit::State(_))) => mouse::Interaction::Pointer,
            (EditorTool::Transition, _, _) => mouse::Interaction::Crosshair,
            (EditorTool::Arrow, _, Some(Hit::State(_))) => mouse::Interaction::Grab,
            (EditorTool::Arrow, _, Some(Hit::Transition(_))) => mouse::Interaction::Pointer,
            (EditorTool::Arrow, _, None) => mouse::Interaction::Idle,
        }
    }

    fn draw(
        &self,
        interaction: &Interaction,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let p = theme::palette(theme);
        let scroll = self.state.scroll();
        let zoom = self.state.zoom();

        let mut overlay = Frame::new(renderer, bounds.size());
        let deleting = self.state.is_deletion_mode();
        let highlight = if deleting { p.danger } else { p.accent };
        overlay.with_save(|frame| {
            frame.scale(zoom);
            frame.translate(-scroll);
            let node_by_id = self.node_by_id();
            match interaction.hover.filter(|_| self.interactive) {
                Some(Hit::State(id)) => {
                    if let Some(node) = node_by_id.get(&id) {
                        frame.stroke(
                            &Path::circle(node.position, StateNode::RADIUS + 5.0),
                            Stroke::default().with_width(2.5).with_color(theme::alpha(highlight, 0.75)),
                        );
                    }
                }
                Some(Hit::Transition((from, to))) => {
                    if let (Some(geometry), Some(labels)) =
                        (self.edge_geometry(&node_by_id, from, to), self.transitions.get(&(from, to)))
                    {
                        let fill = if deleting { p.danger_soft } else { p.accent_soft };
                        let look = EdgeLook { width: 3.0, line: highlight, pill_stroke: highlight, pill_fill: fill };
                        draw_edge(frame, p, &geometry, labels, look);
                    }
                }
                None => {}
            }

            if let (Some(Gesture::Connecting { from_point, .. }), Some(cursor_position)) =
                (interaction.gesture, cursor.position_in(bounds))
            {
                let target = self.state.to_world(cursor_position);
                let start = from_point + (target - from_point).unit() * StateNode::RADIUS;
                frame.stroke(
                    &Path::line(start, target),
                    Stroke {
                        line_dash: LineDash { segments: &[7.0, 5.0], offset: 0 },
                        ..Stroke::default()
                            .with_width(2.0)
                            .with_color(p.accent_strong)
                            .with_line_cap(LineCap::Round)
                    },
                );
                frame.fill(&Path::circle(target, 3.5), p.accent_strong);
            }
        });

        if deleting && self.interactive {
            if let Some(position) = cursor.position_in(bounds) {
                let center = position + Vector::new(14.0, 14.0);
                overlay.fill(&Path::circle(center, 8.0), p.danger);
                let arm = 3.2;
                for (a, b) in [
                    (Vector::new(-arm, -arm), Vector::new(arm, arm)),
                    (Vector::new(arm, -arm), Vector::new(-arm, arm)),
                ] {
                    overlay.stroke(
                        &Path::line(center + a, center + b),
                        Stroke::default().with_width(1.8).with_color(p.on_danger).with_line_cap(LineCap::Round),
                    );
                }
            }
        }

        self.draw_scrollbars(&mut overlay, p, bounds.size());
        vec![overlay.into_geometry()]
    }
}

impl StateMachine<'_> {
    /// The edges `(from, to)` that pass `include`, in world coordinates.
    pub(crate) fn draw_edges(&self, frame: &mut Frame, p: &Palette, include: impl Fn(usize, usize) -> bool) {
        let node_by_id = self.node_by_id();
        for (&(from, to), labels) in self.transitions.iter() {
            if !include(from, to) {
                continue;
            }
            if let Some(geometry) = self.edge_geometry(&node_by_id, from, to) {
                let look = EdgeLook { width: 1.75, line: p.edge, pill_stroke: p.border_soft, pill_fill: p.label_bg };
                draw_edge(frame, p, &geometry, labels, look);
            }
        }
    }

    /// The states that pass `include`, in world coordinates.
    pub(crate) fn draw_nodes(&self, frame: &mut Frame, p: &Palette, include: impl Fn(usize) -> bool) {
        let tint = theme::family_color(p, self.family);
        for node in self.states.iter().filter(|node| include(node.id)) {
            node.draw(
                frame,
                p,
                tint,
                NodeLook {
                    initial: self.initial_state == Some(node.id),
                    accepting: self.final_states.contains(&node.id),
                    active: self.active_states.contains(&node.id),
                },
            );
        }
    }

    /// World rectangle covering everything `draw_edges` and `draw_nodes` paint.
    pub(crate) fn content_extent(&self) -> Rectangle {
        // Reaches past the circle to the active glow, drop shadow and initial pointer.
        const NODE_REACH: f32 = StateNode::RADIUS + 24.0;
        const STROKE_ALLOWANCE: f32 = 3.0;
        let node_by_id = self.node_by_id();
        let nodes = self.states.iter().map(|node| {
            Rectangle::new(node.position - Vector::new(NODE_REACH, NODE_REACH), Size::new(2.0 * NODE_REACH, 2.0 * NODE_REACH))
        });
        let edges = self.transitions.iter().filter_map(|(&(from, to), labels)| {
            Some(self.edge_geometry(&node_by_id, from, to)?.extent(labels).expand(STROKE_ALLOWANCE))
        });
        nodes.chain(edges).reduce(|a, b| a.union(&b)).unwrap_or(Rectangle::new(Point::ORIGIN, Size::new(1.0, 1.0)))
    }

    fn draw_scrollbars(&self, frame: &mut Frame, p: &Palette, view: Size) {
        const THICKNESS: f32 = 5.0;
        const MARGIN: f32 = 4.0;
        const MIN_THUMB: f32 = 28.0;
        if self.states.is_empty() {
            return;
        }
        let content = content_bounds(self.states);
        let visible = self.state.visible_world(view);
        let color = theme::alpha(p.text_faint, 0.55);

        let span = |content_start: f32, content_len: f32, view_start: f32, view_len: f32| {
            let start = content_start.min(view_start);
            let end = (content_start + content_len).max(view_start + view_len);
            (start, end - start)
        };
        let (y_start, y_len) = span(content.y, content.height, visible.y, visible.height);
        if y_len > visible.height + 1.0 {
            let track = view.height - 2.0 * MARGIN;
            let thumb = (track * visible.height / y_len).max(MIN_THUMB);
            let y = MARGIN + (visible.y - y_start) / (y_len - visible.height) * (track - thumb);
            frame.fill(
                &Path::rounded_rectangle(
                    Point::new(view.width - THICKNESS - MARGIN, y),
                    Size::new(THICKNESS, thumb),
                    (THICKNESS / 2.0).into(),
                ),
                color,
            );
        }
        let (x_start, x_len) = span(content.x, content.width, visible.x, visible.width);
        if x_len > visible.width + 1.0 {
            let track = view.width - 2.0 * MARGIN;
            let thumb = (track * visible.width / x_len).max(MIN_THUMB);
            let x = MARGIN + (visible.x - x_start) / (x_len - visible.width) * (track - thumb);
            frame.fill(
                &Path::rounded_rectangle(
                    Point::new(x, view.height - THICKNESS - MARGIN),
                    Size::new(thumb, THICKNESS),
                    (THICKNESS / 2.0).into(),
                ),
                color,
            );
        }
    }
}

struct EdgeLook {
    width: f32,
    line: Color,
    pill_stroke: Color,
    pill_fill: Color,
}

fn draw_edge(frame: &mut Frame, p: &Palette, geometry: &EdgeGeometry, labels: &IndexSet<String>, look: EdgeLook) {
    frame.stroke(
        &geometry.shaft_path(),
        Stroke::default().with_width(look.width).with_color(look.line).with_line_cap(LineCap::Round),
    );
    frame.fill(&geometry.arrowhead_path(), look.line);
    for (rect, label) in geometry.label_rects(labels) {
        let pill = Path::rounded_rectangle(rect.position(), rect.size(), (rect.height / 2.0).into());
        frame.fill(&pill, look.pill_fill);
        frame.stroke(&pill, Stroke::default().with_width(1.0).with_color(look.pill_stroke));
        frame.fill_text(Text {
            content: label.to_string(),
            position: rect.center(),
            color: p.text,
            size: LABEL_FONT_SIZE.into(),
            font: theme::MONO,
            shaping: theme::shaping_for(label),
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
            ..Text::default()
        });
    }
}
