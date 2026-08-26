use iced::widget::canvas::{self, Canvas};
use iced::{Element, Fill};
use std::collections::HashSet;

use super::message::CanvasMessage;
use super::node::StateNode;
use super::program::StateMachine;

pub struct State {
    pub(crate) cache: canvas::Cache,
    ctrl_pressed: bool,
    shift_pressed: bool,
    alt_pressed: bool,
    deletion_mode: bool,
    pub next_id: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            cache: canvas::Cache::default(),
            ctrl_pressed: false,
            shift_pressed: false,
            alt_pressed: false,
            deletion_mode: false,
            next_id: 0,
        }
    }
}

impl State {
    pub fn view<'a>(
        &'a self,
        states: &'a [StateNode],
        transitions: &'a std::collections::HashMap<(usize, usize), indexmap::IndexSet<String>>,
        initial_state: Option<usize>,
        final_states: &'a HashSet<usize>
    ) -> Element<'a, CanvasMessage> {
        Canvas::new(StateMachine {
            state: self,
            states,
            transitions,
            initial_state,
            final_states,
        })
        .width(Fill)
        .height(Fill)
        .into()
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear();
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

    pub fn set_deletion_mode(&mut self, enabled: bool) {
        self.deletion_mode = enabled;
        self.cache.clear();
    }

    pub fn is_deletion_mode(&self) -> bool {
        self.deletion_mode
    }

    pub fn get_current_next_id(&self) -> usize {
        self.next_id
    }

    pub fn reset_id_counter(&mut self) {
        self.next_id = 0;
    }

    pub fn check_input(&self, _input: &str) -> bool {
        false
    }
}
