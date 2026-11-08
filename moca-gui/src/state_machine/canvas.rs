use iced::widget::canvas::Canvas;
use iced::widget::stack;
use iced::{Element, Fill, Size};
use indexmap::IndexSet;
use std::collections::{HashMap, HashSet};

use super::message::CanvasMessage;
use super::node::StateNode;
use super::state::State;
use crate::family::Family;

#[derive(Clone, Copy)]
pub(crate) struct StateMachine<'a> {
    pub(crate) state: &'a State,
    pub(crate) states: &'a [StateNode],
    pub(crate) transitions: &'a HashMap<(usize, usize), IndexSet<String>>,
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
        states: &'a [StateNode],
        transitions: &'a HashMap<(usize, usize), IndexSet<String>>,
        initial_state: Option<usize>,
        final_states: &'a HashSet<usize>,
        context: CanvasContext<'a>,
    ) -> Element<'a, CanvasMessage> {
        let machine = StateMachine {
            state: self,
            states,
            transitions,
            initial_state,
            final_states,
            active_states: context.active_states,
            family: context.family,
            interactive: context.interactive,
            known_viewport: context.known_viewport,
        };
        stack![super::scene::scene(machine), Canvas::new(machine).width(Fill).height(Fill)].into()
    }
}
