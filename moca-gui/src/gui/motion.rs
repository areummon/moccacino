//! Shared motion: the app's one easing curve and the toast slide.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::widget::{tree, Operation, Tree, Widget};
use iced::advanced::{overlay, Clipboard, Shell};
use iced::event::{self, Event};
use iced::time::Instant;
use iced::{mouse, window, Element, Length, Rectangle, Renderer, Size, Theme, Vector};

/// cubic-bezier(0.22, 1, 0.36, 1): a strong ease-out, quick to answer and soft to land.
pub(crate) fn ease_out(t: f32) -> f32 {
    let bezier = |a: f32, b: f32, s: f32| 3.0 * a * s * (1.0 - s).powi(2) + 3.0 * b * s * s * (1.0 - s) + s.powi(3);
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..24 {
        let mid = (lo + hi) / 2.0;
        if bezier(0.22, 0.36, mid) < t {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    bezier(1.0, 1.0, (lo + hi) / 2.0)
}

const ENTER: f32 = 0.26;
pub(crate) const EXIT: f32 = 0.18;
/// Past the window's right edge, clearing the toast layer's margin and shadow.
const OFFSTAGE: f32 = 40.0;

/// Slides `content` in from the right edge when it first appears and back out
/// the same way once `leaving` is set. Only drawing moves; layout stays put.
pub(crate) fn slide_from_right<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    leaving: bool,
) -> Element<'a, Message> {
    Element::new(Slide { content: content.into(), leaving })
}

struct Slide<'a, Message> {
    content: Element<'a, Message>,
    leaving: bool,
}

/// 0 = offstage, 1 = in place.
struct Shown {
    value: f32,
    from: f32,
    target: f32,
    start: Option<Instant>,
}

impl<Message> Widget<Message, Theme, Renderer> for Slide<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Shown>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Shown { value: 0.0, from: 0.0, target: 0.0, start: None })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.content.as_widget().layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(&self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        self.content.as_widget().operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let shown = tree.state.downcast_mut::<Shown>();
            let target = if self.leaving { 0.0 } else { 1.0 };
            if target != shown.target {
                shown.from = shown.value;
                shown.target = target;
                shown.start = Some(now);
            }
            if let Some(start) = shown.start {
                let duration = if self.leaving { EXIT } else { ENTER };
                let t = (now.saturating_duration_since(start).as_secs_f32() / duration).min(1.0);
                shown.value = shown.from + (shown.target - shown.from) * ease_out(t);
                if t < 1.0 {
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                } else {
                    shown.start = None;
                }
            }
        }
        self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let shown = tree.state.downcast_ref::<Shown>().value;
        if shown <= 0.0 {
            return;
        }
        let offset = Vector::new((1.0 - shown) * (layout.bounds().width + OFFSTAGE), 0.0);
        renderer.with_translation(offset, |renderer| {
            self.content.as_widget().draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(&mut tree.children[0], layout, renderer, translation)
    }
}
