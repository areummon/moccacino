//! The canvas backdrop: background, dot grid and the cached graph, drawn under the
//! interactive `Canvas` (which keeps only gestures and the hover overlay).
//!
//! Panning must not rebuild anything. The dot grid is one cached tile shifted by the scroll
//! remainder, and the graph is tessellated once per zoom level (or edit) into a frame
//! covering its whole extent, then slid into place with a translation.

use iced::advanced::graphics::geometry::Renderer as _;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad, Renderer as _};
use iced::advanced::widget::{Tree, Widget};
use iced::widget::canvas::{Cache, Frame, Path};
use iced::{mouse, Element, Length, Point, Rectangle, Renderer, Size, Theme, Vector};

use super::canvas::StateMachine;
use crate::theme::{self, Palette};

const GRID_SPACING: f32 = 28.0;

pub(crate) fn scene<'a, Message: 'a>(machine: StateMachine<'a>) -> Element<'a, Message> {
    Element::new(Scene { machine })
}

struct Scene<'a> {
    machine: StateMachine<'a>,
}

impl<Message> Widget<Message, Theme, Renderer> for Scene<'_> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::atomic(limits, Length::Fill, Length::Fill)
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let p = theme::palette(theme);
        renderer.with_layer(bounds, |renderer| {
            renderer.fill_quad(Quad { bounds, ..Quad::default() }, p.canvas_bg);
            self.draw_grid(renderer, p, bounds);
            self.draw_graph(renderer, p, bounds);
        });
    }
}

impl Scene<'_> {
    fn draw_grid(&self, renderer: &mut Renderer, p: &Palette, bounds: Rectangle) {
        let state = self.machine.state;
        let (zoom, scroll) = (state.zoom(), state.scroll());
        let spacing = if zoom < 0.7 { GRID_SPACING * 2.0 } else { GRID_SPACING };
        let step = spacing * zoom;
        let side = 2.4 * zoom / zoom.max(0.8);
        // The grid is periodic, so one tile a step larger than the view on every side covers
        // any scroll: shifting it by the scroll's remainder modulo a step lines the dots up
        // with the world.
        let size = Size::new(bounds.width + 2.0 * step, bounds.height + 2.0 * step);
        let geometry = state.grid_cache.draw(renderer, size, |frame| {
            let dots = Path::new(|builder| {
                let mut y = step;
                while y < size.height {
                    let mut x = step;
                    while x < size.width {
                        builder.rectangle(Point::new(x - side / 2.0, y - side / 2.0), Size::new(side, side));
                        x += step;
                    }
                    y += step;
                }
            });
            frame.fill(&dots, p.canvas_grid);
        });
        let phase = |scroll: f32| (scroll * zoom).rem_euclid(step) + step;
        let offset = Vector::new(bounds.x - phase(scroll.x), bounds.y - phase(scroll.y));
        renderer.with_translation(offset, |renderer| renderer.draw_geometry(geometry));
    }

    fn draw_graph(&self, renderer: &mut Renderer, p: &Palette, bounds: Rectangle) {
        let machine = &self.machine;
        let state = machine.state;
        if machine.states.is_empty() {
            return;
        }
        let (zoom, scroll) = (state.zoom(), state.scroll());
        let extent = state.scene_extent(|| machine.content_extent());
        let size = Size::new((extent.width * zoom).ceil().max(1.0), (extent.height * zoom).ceil().max(1.0));
        let dragged = state.dragged();
        let still = |id: usize| Some(id) != dragged;
        let cached = |cache: &Cache, draw: &dyn Fn(&mut Frame)| {
            cache.draw(renderer, size, |frame| {
                frame.scale(zoom);
                frame.translate(Vector::new(-extent.x, -extent.y));
                draw(frame);
            })
        };
        let edges = cached(&state.edge_cache, &|frame| machine.draw_edges(frame, p, |from, to| still(from) && still(to)));
        let nodes = cached(&state.node_cache, &|frame| machine.draw_nodes(frame, p, still));
        // The dragged state and its edges, drawn fresh at viewport size.
        let live = |draw: &dyn Fn(&mut Frame)| {
            let mut frame = Frame::new(renderer, bounds.size());
            frame.scale(zoom);
            frame.translate(-scroll);
            draw(&mut frame);
            frame.into_geometry()
        };
        let moving = dragged.map(|id| {
            (
                live(&|frame| machine.draw_edges(frame, p, |from, to| from == id || to == id)),
                live(&|frame| machine.draw_nodes(frame, p, |node| node == id)),
            )
        });

        let offset = Vector::new(bounds.x + (extent.x - scroll.x) * zoom, bounds.y + (extent.y - scroll.y) * zoom);
        let origin = Vector::new(bounds.x, bounds.y);
        let (moving_edges, moving_node) = moving.unzip();
        renderer.with_translation(offset, |renderer| renderer.draw_geometry(edges));
        if let Some(geometry) = moving_edges {
            renderer.with_translation(origin, |renderer| renderer.draw_geometry(geometry));
        }
        renderer.with_translation(offset, |renderer| renderer.draw_geometry(nodes));
        if let Some(geometry) = moving_node {
            renderer.with_translation(origin, |renderer| renderer.draw_geometry(geometry));
        }
    }
}
