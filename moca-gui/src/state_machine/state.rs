use iced::widget::canvas::{self, Canvas};
use iced::{Element, Fill, Vector};
use std::collections::HashSet;

use super::message::CanvasMessage;
use super::node::StateNode;
use super::program::StateMachine;
use super::tool::EditorTool;

pub struct State {
    pub(crate) cache: canvas::Cache,
    ctrl_pressed: bool,
    shift_pressed: bool,
    alt_pressed: bool,
    /* The active editing tool (JFLAP-style); Delete doubles as the old
     * deletion mode. */
    tool: EditorTool,
    /* Tool to restore when a temporary Delete engagement ends (the Delete
     * key press stashes the current tool, its release restores it). */
    tool_before_temp_delete: EditorTool,
    /* World position of the viewport's top-left corner; the wheel pans
     * this offset so drawings outside the visible area stay reachable. */
    scroll: Vector,
    pub next_id: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            cache: canvas::Cache::default(),
            ctrl_pressed: false,
            shift_pressed: false,
            alt_pressed: false,
            tool: EditorTool::default(),
            tool_before_temp_delete: EditorTool::default(),
            scroll: Vector::new(0.0, 0.0),
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

    pub fn scroll(&self) -> Vector {
        self.scroll
    }

    pub fn set_scroll(&mut self, scroll: Vector) {
        self.scroll = scroll;
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

    /* The active tool; Delete doubles as deletion mode, which is why the
     * old flag accessor derives from it. */
    pub fn active_tool(&self) -> EditorTool {
        self.tool
    }

    pub fn set_tool(&mut self, tool: EditorTool) {
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

    pub fn check_input(&self, _input: &str) -> bool {
        false
    }
}
