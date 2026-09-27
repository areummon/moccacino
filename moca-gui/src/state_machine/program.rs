use iced::widget::canvas::{self, Event, Frame, Geometry, LineCap, LineDash, Path, Stroke, Text};
use iced::{alignment, mouse, Point, Rectangle, Renderer, Size, Theme, Vector};
use std::collections::HashSet;

use super::hit_testing::Hit;
use super::message::CanvasMessage;
use super::node::{NodeLook, StateNode};
use super::pending::PendingTransition;
use super::state::{clamp_scroll, content_bounds, zoom_target, State};
use super::tool::EditorTool;
use super::edge::LABEL_FONT_SIZE;
use super::util::VectorExt;
use crate::gui::theme::{self, Family, Palette};

const GRID_SPACING: f32 = 28.0;
const DOUBLE_CLICK: std::time::Duration = std::time::Duration::from_millis(300);

pub(crate) struct StateMachine<'a> {
    pub(crate) state: &'a State,
    pub(crate) states: &'a [StateNode],
    pub(crate) transitions: &'a std::collections::HashMap<(usize, usize), indexmap::IndexSet<String>>,
    pub(crate) initial_state: Option<usize>,
    pub(crate) final_states: &'a HashSet<usize>,
    pub(crate) active_states: &'a HashSet<usize>,
    pub(crate) family: Family,
    pub(crate) interactive: bool,
    pub(crate) known_viewport: Size,
}

/* Widget-local interaction state: the gesture in progress plus whatever
 * the cursor currently hovers (drawn in an uncached overlay, so hovering
 * never invalidates the cached drawing). */
#[derive(Debug, Default)]
pub(crate) struct Interaction {
    pending: Option<PendingTransition>,
    hover: Option<Hit>,
}

impl StateMachine<'_> {
    fn zoom(&self) -> f32 {
        self.state.zoom()
    }

    /* Viewport → world. */
    fn to_world(&self, point: Point) -> Point {
        let scroll = self.state.scroll();
        let zoom = self.zoom();
        Point::new(point.x / zoom + scroll.x, point.y / zoom + scroll.y)
    }

    fn changed(a: Vector, b: Vector) -> bool {
        (a.x - b.x).abs() > f32::EPSILON || (a.y - b.y).abs() > f32::EPSILON
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
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        if !self.interactive {
            interaction.hover = None;
            return (canvas::event::Status::Ignored, None);
        }
        let cursor_position = cursor.position_in(bounds);
        let scroll = self.state.scroll();
        let zoom = self.zoom();
        let viewport = bounds.size();
        // Hit-testing happens in world coordinates: the stored state
        // positions are world positions, the cursor is viewport-relative.
        let world_position = cursor_position.map(|pos| self.to_world(pos));

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let Some(anchor) = cursor_position else {
                    return (canvas::event::Status::Ignored, None);
                };
                let (dx, dy) = match delta {
                    mouse::ScrollDelta::Lines { x, y } => (x * 40.0, y * 40.0),
                    mouse::ScrollDelta::Pixels { x, y } => (x, y),
                };
                if self.state.is_ctrl_pressed() {
                    // Ctrl+wheel zooms around the cursor.
                    let factor = 1.0015f32.powf(dy.clamp(-240.0, 240.0));
                    let (new_zoom, new_scroll) =
                        zoom_target(zoom, scroll, factor, anchor, self.states, viewport);
                    return (
                        canvas::event::Status::Captured,
                        Some(CanvasMessage::Zoomed { zoom: new_zoom, scroll: new_scroll }),
                    );
                }
                // Shift+wheel pans sideways on plain mouse wheels.
                let (dx, dy) = if self.state.is_shift_pressed() && dx == 0.0 { (-dy, 0.0) } else { (dx, dy) };
                let new_scroll = clamp_scroll(
                    Vector::new(scroll.x + dx / zoom, scroll.y - dy / zoom),
                    self.states,
                    viewport,
                    zoom,
                );
                if Self::changed(new_scroll, scroll) {
                    (canvas::event::Status::Captured, Some(CanvasMessage::Scrolled(new_scroll)))
                } else {
                    (canvas::event::Status::Captured, None)
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let (Some(cursor_pos), Some(screen_pos)) = (world_position, cursor_position) else {
                    return (canvas::event::Status::Ignored, None);
                };

                // Shared hit-testing: the active tool decides what a press
                // on each target means.
                let hit = self.hit_test(cursor_pos);
                let clicked_node = match hit {
                    Some(Hit::State(id)) => self.states.iter().find(|node| node.id == id),
                    _ => None,
                };
                let clicked_transition = match hit {
                    Some(Hit::Transition(pair)) => Some(pair),
                    _ => None,
                };

                match self.state.active_tool() {
                    EditorTool::Arrow => {
                        let now = std::time::Instant::now();
                        let (last_click_time, last_clicked_state, last_clicked_transition) =
                            match interaction.pending {
                                Some(PendingTransition::ClickTracking {
                                    last_click_time,
                                    last_clicked_state,
                                    last_clicked_transition,
                                }) => (last_click_time, last_clicked_state, last_clicked_transition),
                                _ => (now - DOUBLE_CLICK - std::time::Duration::from_millis(1), None, None),
                            };
                        interaction.pending = None;

                        if let Some(node) = clicked_node {
                            if now.duration_since(last_click_time) < DOUBLE_CLICK
                                && last_clicked_state == Some(node.id)
                            {
                                interaction.pending = Some(PendingTransition::ClickTracking {
                                    last_click_time: now,
                                    last_clicked_state: Some(node.id),
                                    last_clicked_transition: None,
                                });
                                return (
                                    canvas::event::Status::Captured,
                                    Some(CanvasMessage::StateDoubleClicked(node.id)),
                                );
                            }
                            // Modifier clicks never become drags, and they
                            // leave no click tracking so a fast modifier
                            // double-tap cannot trigger a rename.
                            if self.state.is_shift_pressed() || self.state.is_alt_pressed() {
                                return (
                                    canvas::event::Status::Captured,
                                    Some(CanvasMessage::StateClicked(node.id)),
                                );
                            }
                            interaction.pending = Some(PendingTransition::Dragging {
                                state_id: node.id,
                                offset: cursor_pos - node.position,
                                moved: false,
                            });
                            return (canvas::event::Status::Captured, None);
                        }
                        if let Some(pair) = clicked_transition {
                            let double = now.duration_since(last_click_time) < DOUBLE_CLICK
                                && last_clicked_transition == Some(pair);
                            interaction.pending = Some(PendingTransition::ClickTracking {
                                last_click_time: now,
                                last_clicked_state: None,
                                last_clicked_transition: Some(pair),
                            });
                            let message = if double {
                                CanvasMessage::TransitionDoubleClicked(pair)
                            } else {
                                CanvasMessage::TransitionClicked(pair)
                            };
                            return (canvas::event::Status::Captured, Some(message));
                        }
                        // Empty space: begin panning the viewport.
                        interaction.pending = Some(PendingTransition::Panning {
                            origin_scroll: scroll,
                            cursor_start: screen_pos,
                        });
                        (canvas::event::Status::Captured, None)
                    }
                    EditorTool::State => {
                        interaction.pending = None;
                        if hit.is_none() {
                            let label = format!("q{}", self.state.get_current_next_id());
                            let state_node = StateNode::new_with_temp_id(cursor_pos, 30.0, label);
                            (canvas::event::Status::Captured, Some(CanvasMessage::AddState(state_node)))
                        } else {
                            (canvas::event::Status::Captured, None)
                        }
                    }
                    EditorTool::Transition => match interaction.pending.take() {
                        Some(PendingTransition::Start { from_state_id, from_point }) => {
                            if let Some(to_node) = clicked_node {
                                // Instead of adding the transition here,
                                // request a label from the GUI.
                                (
                                    canvas::event::Status::Captured,
                                    Some(CanvasMessage::RequestTransitionLabel {
                                        from_state_id,
                                        to_state_id: to_node.id,
                                        from_point,
                                        to_point: to_node.position,
                                    }),
                                )
                            } else {
                                // Clicking anything but a state cancels.
                                (canvas::event::Status::Captured, None)
                            }
                        }
                        _ => {
                            if let Some(node) = clicked_node {
                                interaction.pending = Some(PendingTransition::Start {
                                    from_state_id: node.id,
                                    from_point: node.position,
                                });
                            }
                            (canvas::event::Status::Captured, None)
                        }
                    },
                    EditorTool::Delete => {
                        interaction.pending = None;
                        interaction.hover = None;
                        match hit {
                            Some(Hit::State(id)) => {
                                (canvas::event::Status::Captured, Some(CanvasMessage::StateClicked(id)))
                            }
                            Some(Hit::Transition(pair)) => {
                                (canvas::event::Status::Captured, Some(CanvasMessage::TransitionClicked(pair)))
                            }
                            None => (canvas::event::Status::Captured, None),
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // Keep the app's idea of the viewport size current (zoom
                // buttons and fit-to-content need it).
                if (viewport.width - self.known_viewport.width).abs() > 0.5
                    || (viewport.height - self.known_viewport.height).abs() > 0.5
                {
                    return (canvas::event::Status::Ignored, Some(CanvasMessage::Viewport(viewport)));
                }
                let (Some(cursor_pos), Some(screen_pos)) = (world_position, cursor_position) else {
                    interaction.hover = None;
                    return (canvas::event::Status::Ignored, None);
                };
                // Self-heal after content shrinks (state deleted or
                // moved): clamp the stored scroll back into range.
                let healed = clamp_scroll(scroll, self.states, viewport, zoom);
                if Self::changed(healed, scroll)
                    && !matches!(interaction.pending, Some(PendingTransition::Dragging { .. }))
                {
                    return (canvas::event::Status::Captured, Some(CanvasMessage::Scrolled(healed)));
                }
                match interaction.pending {
                    Some(PendingTransition::Dragging { state_id, offset, .. }) => {
                        interaction.pending = Some(PendingTransition::Dragging { state_id, offset, moved: true });
                        interaction.hover = Some(Hit::State(state_id));
                        (
                            canvas::event::Status::Captured,
                            Some(CanvasMessage::MoveState { state_id, new_position: cursor_pos - offset }),
                        )
                    }
                    Some(PendingTransition::Panning { origin_scroll, cursor_start }) => {
                        let drag = screen_pos - cursor_start;
                        let new_scroll = clamp_scroll(
                            Vector::new(origin_scroll.x - drag.x / zoom, origin_scroll.y - drag.y / zoom),
                            self.states,
                            viewport,
                            zoom,
                        );
                        if Self::changed(new_scroll, scroll) {
                            (canvas::event::Status::Captured, Some(CanvasMessage::Scrolled(new_scroll)))
                        } else {
                            (canvas::event::Status::Captured, None)
                        }
                    }
                    Some(PendingTransition::Start { .. }) => {
                        interaction.hover = self.hit_test(cursor_pos).filter(|hit| matches!(hit, Hit::State(_)));
                        (canvas::event::Status::Captured, None)
                    }
                    _ => {
                        interaction.hover = match self.state.active_tool() {
                            // Creating states only cares about empty space.
                            EditorTool::State => None,
                            _ => self.hit_test(cursor_pos),
                        };
                        (canvas::event::Status::Ignored, None)
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorLeft) => {
                interaction.hover = None;
                (canvas::event::Status::Ignored, None)
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => match interaction.pending {
                // A press-release without movement counts as a click: keep
                // it tracked so the next press can detect a double-click
                // rename. Real drags clear the tracking.
                Some(PendingTransition::Dragging { state_id, moved, .. }) => {
                    interaction.pending = if moved {
                        None
                    } else {
                        Some(PendingTransition::ClickTracking {
                            last_click_time: std::time::Instant::now(),
                            last_clicked_state: Some(state_id),
                            last_clicked_transition: None,
                        })
                    };
                    (canvas::event::Status::Captured, None)
                }
                Some(PendingTransition::Panning { .. }) => {
                    interaction.pending = None;
                    (canvas::event::Status::Captured, None)
                }
                _ => (canvas::event::Status::Ignored, None),
            },
            _ => (canvas::event::Status::Ignored, None),
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
        match (self.state.active_tool(), interaction.pending, interaction.hover) {
            (_, Some(PendingTransition::Dragging { .. }), _)
            | (_, Some(PendingTransition::Panning { .. }), _) => mouse::Interaction::Grabbing,
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
        let zoom = self.zoom();
        let tint = self.family.color(p);

        // Background and dot grid depend only on the view, so they live in
        // their own cache and survive drags and edits.
        let grid = self.state.grid_cache.draw(renderer, bounds.size(), |frame| {
            frame.fill(&Path::rectangle(Point::ORIGIN, frame.size()), p.canvas_bg);
            frame.scale(zoom);
            frame.translate(-scroll);
            self.draw_grid(frame, p, bounds.size());
        });

        let content = self.state.cache.draw(renderer, bounds.size(), |frame| {
            // World content is drawn scaled and shifted.
            frame.scale(zoom);
            frame.translate(-scroll);

            let node_by_id = self.node_by_id();
            for (&(from, to), labels) in self.transitions.iter() {
                let Some(geometry) = self.edge_geometry(&node_by_id, from, to) else { continue };
                frame.stroke(
                    &geometry.shaft_path(),
                    Stroke::default()
                        .with_width(1.75)
                        .with_color(p.edge)
                        .with_line_cap(LineCap::Round),
                );
                frame.fill(&geometry.arrowhead_path(), p.edge);
                for (rect, label) in geometry.label_rects(labels) {
                    draw_label_pill(frame, p, rect, label, p.border_soft, p.label_bg);
                }
            }

            for node in self.states {
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
        });

        let mut geometries = vec![grid, content];

        // Uncached overlay: hover highlights, the pending transition line
        // and the delete badge follow the cursor every frame.
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
                            &Path::circle(node.position, node.radius + 5.0),
                            Stroke::default()
                                .with_width(2.5)
                                .with_color(theme::alpha(highlight, 0.75)),
                        );
                    }
                }
                Some(Hit::Transition((from, to))) => {
                    if let (Some(geometry), Some(labels)) =
                        (self.edge_geometry(&node_by_id, from, to), self.transitions.get(&(from, to)))
                    {
                        frame.stroke(
                            &geometry.shaft_path(),
                            Stroke::default()
                                .with_width(3.0)
                                .with_color(highlight)
                                .with_line_cap(LineCap::Round),
                        );
                        frame.fill(&geometry.arrowhead_path(), highlight);
                        let fill = if deleting { p.danger_soft } else { p.accent_soft };
                        for (rect, label) in geometry.label_rects(labels) {
                            draw_label_pill(frame, p, rect, label, highlight, fill);
                        }
                    }
                }
                None => {}
            }

            if let (Some(PendingTransition::Start { from_state_id, from_point }), Some(cursor_position)) =
                (interaction.pending, cursor.position_in(bounds))
            {
                let target = self.to_world(cursor_position);
                let radius = node_by_id.get(&from_state_id).map(|node| node.radius).unwrap_or(30.0);
                let start = from_point + (target - from_point).unit() * radius;
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

        // Delete badge next to the cursor, in screen space.
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
                        Stroke::default().with_width(1.8).with_color(p.surface).with_line_cap(LineCap::Round),
                    );
                }
            }
        }

        self.draw_scrollbars(&mut overlay, p, bounds.size());
        geometries.push(overlay.into_geometry());
        geometries
    }
}

impl StateMachine<'_> {
    /* Dot grid over the visible world area; coarser when zoomed out so the
     * dot count stays bounded. */
    fn draw_grid(&self, frame: &mut Frame, p: &Palette, viewport: Size) {
        let zoom = self.zoom();
        let scroll = self.state.scroll();
        let spacing = if zoom < 0.7 { GRID_SPACING * 2.0 } else { GRID_SPACING };
        let visible = Size::new(viewport.width / zoom, viewport.height / zoom);
        let start_x = (scroll.x / spacing).floor() * spacing;
        let start_y = (scroll.y / spacing).floor() * spacing;
        // Tiny squares: two triangles each instead of a tessellated circle,
        // indistinguishable at this size.
        let side = 2.4 / zoom.max(0.8);
        let dots = Path::new(|b| {
            let mut y = start_y;
            while y <= scroll.y + visible.height + spacing {
                let mut x = start_x;
                while x <= scroll.x + visible.width + spacing {
                    b.rectangle(Point::new(x - side / 2.0, y - side / 2.0), Size::new(side, side));
                    x += spacing;
                }
                y += spacing;
            }
        });
        frame.fill(&dots, p.canvas_grid);
    }

    /* Thin scroll thumbs whenever part of the drawing is off screen: the
     * track spans the union of the content box and the visible area, the
     * thumb is the visible area inside it. */
    fn draw_scrollbars(&self, frame: &mut Frame, p: &Palette, view: Size) {
        const THICKNESS: f32 = 5.0;
        const MARGIN: f32 = 4.0;
        const MIN_THUMB: f32 = 28.0;
        if self.states.is_empty() {
            return;
        }
        let zoom = self.zoom();
        let scroll = self.state.scroll();
        let content = content_bounds(self.states);
        let visible = Rectangle::new(Point::new(scroll.x, scroll.y), Size::new(view.width / zoom, view.height / zoom));
        let color = theme::alpha(p.text_faint, 0.55);

        // (track start, track length) in world units per axis.
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

/* One transition label on a rounded pill. */
fn draw_label_pill(frame: &mut Frame, p: &Palette, rect: Rectangle, label: &str, stroke: iced::Color, fill: iced::Color) {
    let pill = Path::rounded_rectangle(rect.position(), rect.size(), (rect.height / 2.0).into());
    frame.fill(&pill, fill);
    frame.stroke(&pill, Stroke::default().with_width(1.0).with_color(stroke));
    frame.fill_text(Text {
        content: label.to_string(),
        position: rect.center(),
        color: p.text,
        size: LABEL_FONT_SIZE.into(),
        font: theme::MONO,
        shaping: crate::gui::widgets::shaping_for(label),
        horizontal_alignment: alignment::Horizontal::Center,
        vertical_alignment: alignment::Vertical::Center,
        ..Text::default()
    });
}
