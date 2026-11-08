use std::cell::Cell;

use iced::widget::canvas;
use iced::{Point, Rectangle, Size, Vector};

use super::node::StateNode;
use super::tool::EditorTool;

const MIN_ZOOM: f32 = 0.4;
const MAX_ZOOM: f32 = 2.5;
const FIT_MAX_ZOOM: f32 = 1.25;
const CONTENT_MARGIN: f32 = 80.0;
const CONTENT_MARGIN_TOP: f32 = 120.0;

pub struct State {
    /// The graph, split so edges stay under nodes. While a state is dragged both leave it
    /// (and its edges) out, and the scene draws those few shapes fresh each frame.
    pub(crate) edge_cache: canvas::Cache,
    pub(crate) node_cache: canvas::Cache,
    dragged: Option<usize>,
    /// One viewport of dots; kept across pans and edits, rebuilt on zoom, resize or theme.
    pub(crate) grid_cache: canvas::Cache,
    /// World rectangle the cached scene covers; computed with the cache and cleared with it.
    scene_extent: Cell<Option<Rectangle>>,
    tool: EditorTool,
    tool_before_temp_delete: EditorTool,
    scroll: Vector,
    zoom: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            edge_cache: canvas::Cache::default(),
            node_cache: canvas::Cache::default(),
            dragged: None,
            grid_cache: canvas::Cache::default(),
            scene_extent: Cell::new(None),
            tool: EditorTool::default(),
            tool_before_temp_delete: EditorTool::default(),
            scroll: Vector::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

impl State {
    pub(crate) fn scroll(&self) -> Vector {
        self.scroll
    }

    pub(crate) fn set_scroll(&mut self, scroll: Vector) {
        self.scroll = scroll;
    }

    pub(crate) fn zoom(&self) -> f32 {
        self.zoom
    }

    pub(crate) fn set_zoom(&mut self, zoom: f32) {
        let zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        if zoom != self.zoom {
            self.zoom = zoom;
            self.request_full_redraw();
        }
    }

    pub(crate) fn to_world(&self, screen: Point) -> Point {
        Point::new(screen.x / self.zoom + self.scroll.x, screen.y / self.zoom + self.scroll.y)
    }

    pub(crate) fn visible_world(&self, viewport: Size) -> Rectangle {
        visible_world(self.zoom, self.scroll, viewport)
    }

    pub(crate) fn zoom_target(&self, factor: f32, anchor: Point, states: &[StateNode], viewport: Size) -> (f32, Vector) {
        let new_zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        let world_anchor = self.to_world(anchor);
        let unclamped = Vector::new(world_anchor.x - anchor.x / new_zoom, world_anchor.y - anchor.y / new_zoom);
        (new_zoom, clamp_scroll(unclamped, states, viewport, new_zoom))
    }

    pub(crate) fn zoom_around(&mut self, factor: f32, anchor: Point, states: &[StateNode], viewport: Size) {
        let (zoom, scroll) = self.zoom_target(factor, anchor, states, viewport);
        self.set_zoom(zoom);
        self.set_scroll(scroll);
    }

    pub(crate) fn fit_to(&mut self, states: &[StateNode], viewport: Size) {
        if states.is_empty() || viewport.width < 1.0 || viewport.height < 1.0 {
            self.set_zoom(1.0);
            self.set_scroll(Vector::new(0.0, 0.0));
            return;
        }
        let bounds = content_bounds(states);
        let zoom = (viewport.width / bounds.width)
            .min(viewport.height / bounds.height)
            .clamp(MIN_ZOOM, FIT_MAX_ZOOM);
        let visible = visible_world(zoom, Vector::new(0.0, 0.0), viewport);
        self.set_zoom(zoom);
        self.set_scroll(Vector::new(
            bounds.x - (visible.width - bounds.width) / 2.0,
            bounds.y - (visible.height - bounds.height) / 2.0,
        ));
    }

    pub(crate) fn request_redraw(&mut self) {
        self.edge_cache.clear();
        self.node_cache.clear();
        self.scene_extent.set(None);
        self.dragged = None;
    }

    /// Redraw after `id` moved. The first move of a drag rebuilds the caches without it;
    /// later moves of the same state reuse them.
    pub(crate) fn redraw_moved(&mut self, id: usize) {
        if self.dragged != Some(id) {
            self.request_redraw();
            self.dragged = Some(id);
        }
    }

    pub(crate) fn dragged(&self) -> Option<usize> {
        self.dragged
    }

    pub(crate) fn request_full_redraw(&mut self) {
        self.request_redraw();
        self.grid_cache.clear();
    }

    pub(crate) fn scene_extent(&self, compute: impl FnOnce() -> Rectangle) -> Rectangle {
        let extent = self.scene_extent.get().unwrap_or_else(compute);
        self.scene_extent.set(Some(extent));
        extent
    }

    pub(crate) fn active_tool(&self) -> EditorTool {
        self.tool
    }

    pub(crate) fn set_tool(&mut self, tool: EditorTool) {
        self.tool = tool;
        self.request_redraw();
    }

    pub(crate) fn stash_tool(&mut self) {
        if self.tool != EditorTool::Delete {
            self.tool_before_temp_delete = self.tool;
        }
    }

    pub(crate) fn restore_tool(&mut self) {
        self.set_tool(match self.tool_before_temp_delete {
            EditorTool::Delete => EditorTool::Arrow,
            tool => tool,
        });
    }

    pub(crate) fn is_deletion_mode(&self) -> bool {
        self.tool == EditorTool::Delete
    }
}

fn visible_world(zoom: f32, scroll: Vector, viewport: Size) -> Rectangle {
    Rectangle::new(Point::new(scroll.x, scroll.y), Size::new(viewport.width / zoom, viewport.height / zoom))
}

pub(crate) fn content_bounds(states: &[StateNode]) -> Rectangle {
    if states.is_empty() {
        return Rectangle::new(Point::ORIGIN, Size::new(1.0, 1.0));
    }
    let (mut min_x, mut min_y) = (f32::MAX, f32::MAX);
    let (mut max_x, mut max_y) = (f32::MIN, f32::MIN);
    for state in states {
        min_x = min_x.min(state.position.x - StateNode::RADIUS);
        min_y = min_y.min(state.position.y - StateNode::RADIUS);
        max_x = max_x.max(state.position.x + StateNode::RADIUS);
        max_y = max_y.max(state.position.y + StateNode::RADIUS);
    }
    Rectangle::new(
        Point::new(min_x - CONTENT_MARGIN, min_y - CONTENT_MARGIN_TOP),
        Size::new(
            max_x - min_x + 2.0 * CONTENT_MARGIN,
            max_y - min_y + CONTENT_MARGIN + CONTENT_MARGIN_TOP,
        ),
    )
}

fn scroll_range(states: &[StateNode], viewport: Size, zoom: f32) -> ((f32, f32), (f32, f32)) {
    const KEEP_VISIBLE: f32 = 120.0;
    const UNBOUNDED: (f32, f32) = (-1.0e7, 1.0e7);
    if states.is_empty() {
        return (UNBOUNDED, UNBOUNDED);
    }
    let bounds = content_bounds(states);
    let visible = visible_world(zoom, Vector::new(0.0, 0.0), viewport);
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
