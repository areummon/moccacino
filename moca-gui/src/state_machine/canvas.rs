use iced::widget::canvas::Canvas;
use iced::widget::stack;
use iced::{Element, Fill, Size};
use std::collections::HashSet;

use super::drawing::{Drawing, Transitions};
use super::message::CanvasMessage;
use super::node::StateNode;
use super::state::State;
use crate::family::Family;

#[derive(Clone, Copy)]
pub(crate) struct StateMachine<'a> {
    pub(crate) state: &'a State,
    pub(crate) states: &'a [StateNode],
    pub(crate) transitions: &'a Transitions,
    pub(crate) initial_state: Option<usize>,
    pub(crate) final_states: &'a HashSet<usize>,
    pub(crate) active_states: &'a HashSet<usize>,
    pub(crate) family: Family,
    pub(crate) interactive: bool,
    pub(crate) known_viewport: Size,
}

pub(crate) struct CanvasContext<'a> {
    pub(crate) active_states: &'a HashSet<usize>,
    pub(crate) family: Family,
    pub(crate) interactive: bool,
    pub(crate) known_viewport: Size,
}

impl State {
    pub(crate) fn view<'a>(
        &'a self,
        drawing: &'a Drawing,
        context: CanvasContext<'a>,
    ) -> Element<'a, CanvasMessage> {
        let machine = StateMachine {
            state: self,
            states: &drawing.states,
            transitions: &drawing.transitions,
            initial_state: drawing.initial_state,
            final_states: &drawing.final_states,
            active_states: context.active_states,
            family: context.family,
            interactive: context.interactive,
            known_viewport: context.known_viewport,
        };
        stack![super::scene::scene(machine), Canvas::new(machine).width(Fill).height(Fill)].into()
    }
}
