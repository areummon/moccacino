use iced::widget::canvas::{self, Canvas};
use iced::{Element, Fill, Point, Rectangle, Size, Vector};
use std::collections::HashSet;

use super::message::CanvasMessage;
use super::node::StateNode;
use super::program::StateMachine;
use super::tool::EditorTool;
use crate::gui::theme::Family;

pub(crate) const MIN_ZOOM: f32 = 0.4;
pub(crate) const MAX_ZOOM: f32 = 2.5;
/* Fit-to-content never magnifies past this, so tiny machines stay calm. */
const FIT_MAX_ZOOM: f32 = 1.25;
/* World margin kept around the drawn states: room for labels, the
 * initial pointer and (upwards) self-loops. */
const CONTENT_MARGIN: f32 = 80.0;
const CONTENT_MARGIN_TOP: f32 = 120.0;

pub struct State {
    /* Machine drawing (edges, nodes); cleared by every edit. */
    pub(crate) cache: canvas::Cache,
    /* Dot grid; depends only on the view, so it survives drags and edits. */
    pub(crate) grid_cache: canvas::Cache,
    ctrl_pressed: bool,
    shift_pressed: bool,
    alt_pressed: bool,
    /* The active editing tool (JFLAP-style); Delete doubles as the old
     * deletion mode. */
    tool: EditorTool,
    /* Tool to restore when a temporary Delete engagement ends (the Delete
     * key press stashes the current tool, its release restores it). */
    tool_before_temp_delete: EditorTool,
    /* World position of the viewport's top-left corner. */
    scroll: Vector,
    /* Screen pixels per world unit. */
    zoom: f32,
    pub next_id: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            cache: canvas::Cache::default(),
            grid_cache: canvas::Cache::default(),
            ctrl_pressed: false,
            shift_pressed: false,
            alt_pressed: false,
            tool: EditorTool::default(),
            tool_before_temp_delete: EditorTool::default(),
            scroll: Vector::new(0.0, 0.0),
            zoom: 1.0,
            next_id: 0,
        }
    }
}

/* Everything the canvas needs besides the drawing itself. */
pub(crate) struct CanvasContext<'a> {
    pub(crate) active_states: &'a HashSet<usize>,
    pub(crate) family: Family,
    /* False while a modal is open: the canvas then ignores the mouse. */
    pub(crate) interactive: bool,
    /* Last viewport size the app knows about; the canvas reports changes. */
    pub(crate) known_viewport: Size,
}

impl State {
    pub(crate) fn view<'a>(
        &'a self,
        states: &'a [StateNode],
        transitions: &'a std::collections::HashMap<(usize, usize), indexmap::IndexSet<String>>,
        initial_state: Option<usize>,
        final_states: &'a HashSet<usize>,
        context: CanvasContext<'a>,
    ) -> Element<'a, CanvasMessage> {
        Canvas::new(StateMachine {
            state: self,
            states,
            transitions,
            initial_state,
            final_states,
            active_states: context.active_states,
            family: context.family,
            interactive: context.interactive,
            known_viewport: context.known_viewport,
        })
        .width(Fill)
        .height(Fill)
        .into()
    }

    pub fn scroll(&self) -> Vector {
        self.scroll
    }

    pub fn set_scroll(&mut self, scroll: Vector) {
        self.scroll = scroll;
        self.grid_cache.clear();
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        self.grid_cache.clear();
    }

    /* Zoom by `factor` keeping the world point under `anchor` (viewport
     * coordinates) fixed on screen. */
    pub(crate) fn zoom_around(&mut self, factor: f32, anchor: Point, states: &[StateNode], viewport: Size) {
        let (zoom, scroll) = zoom_target(self.zoom, self.scroll, factor, anchor, states, viewport);
        self.zoom = zoom;
        self.scroll = scroll;
        self.cache.clear();
        self.grid_cache.clear();
    }

    /* Frame every state in the viewport, centered, without magnifying
     * small machines beyond FIT_MAX_ZOOM. */
    pub(crate) fn fit_to(&mut self, states: &[StateNode], viewport: Size) {
        if states.is_empty() || viewport.width < 1.0 || viewport.height < 1.0 {
            self.zoom = 1.0;
            self.scroll = Vector::new(0.0, 0.0);
            self.request_full_redraw();
            return;
        }
        let bounds = content_bounds(states);
        let zoom = (viewport.width / bounds.width)
            .min(viewport.height / bounds.height)
            .clamp(MIN_ZOOM, FIT_MAX_ZOOM);
        let visible = Size::new(viewport.width / zoom, viewport.height / zoom);
        self.zoom = zoom;
        self.scroll = Vector::new(
            bounds.x - (visible.width - bounds.width) / 2.0,
            bounds.y - (visible.height - bounds.height) / 2.0,
        );
        self.request_full_redraw();
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear();
    }

    /* Repaint everything, grid included (theme switches). */
    pub(crate) fn request_full_redraw(&mut self) {
        self.cache.clear();
        self.grid_cache.clear();
    }

    pub fn set_ctrl_pressed(&mut self, pressed: bool) {
        self.ctrl_pressed = pressed;
    }

    pub fn set_shift_pressed(&mut self, pressed: bool) {
        self.shift_pressed = pressed;
    }

    pub fn set_alt_pressed(&mut self, pressed: bool) {
        self.alt_pressed = pressed;
    }

    pub fn is_ctrl_pressed(&self) -> bool {
        self.ctrl_pressed
    }

    pub fn is_shift_pressed(&self) -> bool {
        self.shift_pressed
    }

    pub fn is_alt_pressed(&self) -> bool {
        self.alt_pressed
    }

    /* The active tool; Delete doubles as deletion mode, which is why the
     * old flag accessor derives from it. */
    pub(crate) fn active_tool(&self) -> EditorTool {
        self.tool
    }

    pub(crate) fn set_tool(&mut self, tool: EditorTool) {
        self.tool = tool;
        self.cache.clear();
    }

    /* Remembers the current tool before a temporary Delete engagement. */
    pub fn stash_tool(&mut self) {
        if self.tool != EditorTool::Delete {
            self.tool_before_temp_delete = self.tool;
        }
    }

    /* Restores the stashed tool, falling back to Arrow when Delete itself
     * was stashed (never re-engage Delete from a release). */
    pub fn restore_tool(&mut self) {
        self.tool = match self.tool_before_temp_delete {
            EditorTool::Delete => EditorTool::Arrow,
            tool => tool,
        };
        self.cache.clear();
    }

    pub fn is_deletion_mode(&self) -> bool {
        self.tool == EditorTool::Delete
    }

    pub fn get_current_next_id(&self) -> usize {
        self.next_id
    }

    pub fn reset_id_counter(&mut self) {
        self.next_id = 0;
    }
}

/* World rectangle covering every state plus drawing margins; a unit box
 * at the origin when the canvas is empty. */
pub(crate) fn content_bounds(states: &[StateNode]) -> Rectangle {
    if states.is_empty() {
        return Rectangle::new(Point::ORIGIN, Size::new(1.0, 1.0));
    }
    let (mut min_x, mut min_y) = (f32::MAX, f32::MAX);
    let (mut max_x, mut max_y) = (f32::MIN, f32::MIN);
    for state in states {
        min_x = min_x.min(state.position.x - state.radius);
        min_y = min_y.min(state.position.y - state.radius);
        max_x = max_x.max(state.position.x + state.radius);
        max_y = max_y.max(state.position.y + state.radius);
    }
    Rectangle::new(
        Point::new(min_x - CONTENT_MARGIN, min_y - CONTENT_MARGIN_TOP),
        Size::new(
            max_x - min_x + 2.0 * CONTENT_MARGIN,
            max_y - min_y + CONTENT_MARGIN + CONTENT_MARGIN_TOP,
        ),
    )
}

/* Allowed scroll interval per axis at a zoom level. Panning is free as long
 * as a strip of the content box stays on screen, so the drawing can never
 * be lost off-canvas, while placing a state near an edge never makes the
 * view jump. An empty canvas pans freely. */
pub(crate) fn scroll_range(states: &[StateNode], viewport: Size, zoom: f32) -> ((f32, f32), (f32, f32)) {
    /* World units of content that must remain visible. */
    const KEEP_VISIBLE: f32 = 120.0;
    const UNBOUNDED: (f32, f32) = (-1.0e7, 1.0e7);
    if states.is_empty() {
        return (UNBOUNDED, UNBOUNDED);
    }
    let bounds = content_bounds(states);
    let visible = Size::new(viewport.width / zoom, viewport.height / zoom);
    let axis = |start: f32, length: f32, visible: f32| {
        let keep = KEEP_VISIBLE.min(length / 2.0).min(visible / 2.0);
        let lo = start + keep - visible;
        let hi = start + length - keep;
        (lo.min(hi), lo.max(hi))
    };
    (
        axis(bounds.x, bounds.width, visible.width),
        axis(bounds.y, bounds.height, visible.height),
    )
}

pub(crate) fn clamp_scroll(scroll: Vector, states: &[StateNode], viewport: Size, zoom: f32) -> Vector {
    let ((x_lo, x_hi), (y_lo, y_hi)) = scroll_range(states, viewport, zoom);
    Vector::new(scroll.x.clamp(x_lo, x_hi), scroll.y.clamp(y_lo, y_hi))
}

/* The zoom and scroll after zooming by `factor` around a viewport point. */
pub(crate) fn zoom_target(
    zoom: f32,
    scroll: Vector,
    factor: f32,
    anchor: Point,
    states: &[StateNode],
    viewport: Size,
) -> (f32, Vector) {
    let new_zoom = (zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
    let world_anchor = Point::new(anchor.x / zoom + scroll.x, anchor.y / zoom + scroll.y);
    let unclamped = Vector::new(
        world_anchor.x - anchor.x / new_zoom,
        world_anchor.y - anchor.y / new_zoom,
    );
    (new_zoom, clamp_scroll(unclamped, states, viewport, new_zoom))
}
