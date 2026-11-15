use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use iced::{Point, Vector};
use indexmap::IndexSet;

use super::node::StateNode;

pub(crate) type Transitions = HashMap<(usize, usize), IndexSet<String>>;

/// What the canvas shows and every edit changes: the unit the undo history snapshots.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Drawing {
    pub(crate) states: Vec<StateNode>,
    pub(crate) transitions: Transitions,
    pub(crate) initial_state: Option<usize>,
    pub(crate) final_states: HashSet<usize>,
    next_state_id: usize,
}

impl Drawing {
    pub(crate) fn new(
        states: Vec<StateNode>,
        transitions: Transitions,
        initial_state: Option<usize>,
        final_states: HashSet<usize>,
    ) -> Self {
        let next_state_id = states.iter().map(|node| node.id + 1).max().unwrap_or(0);
        Self { states, transitions, initial_state, final_states, next_state_id }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.states.is_empty() && self.transitions.is_empty()
    }

    pub(crate) fn node(&self, state_id: usize) -> Option<&StateNode> {
        self.states.iter().find(|node| node.id == state_id)
    }

    fn node_mut(&mut self, state_id: usize) -> Option<&mut StateNode> {
        self.states.iter_mut().find(|node| node.id == state_id)
    }

    pub(crate) fn add_state(&mut self, position: Point) {
        let id = self.next_state_id;
        self.next_state_id += 1;
        self.states.push(StateNode::new(id, position, format!("q{id}")));
        if self.states.len() == 1 && self.initial_state.is_none() {
            self.initial_state = Some(id);
        }
    }

    pub(crate) fn translate(&mut self, ids: &HashSet<usize>, delta: Vector) {
        for node in self.states.iter_mut().filter(|node| ids.contains(&node.id)) {
            node.position = node.position + delta;
        }
    }

    pub(crate) fn rename_state(&mut self, state_id: usize, label: String) {
        if let Some(node) = self.node_mut(state_id) {
            node.label = label;
        }
    }

    pub(crate) fn delete_state(&mut self, state_id: usize) {
        self.states.retain(|node| node.id != state_id);
        self.transitions.retain(|&(from, to), _| from != state_id && to != state_id);
        if self.initial_state == Some(state_id) {
            self.initial_state = None;
        }
        self.final_states.remove(&state_id);
    }

    pub(crate) fn set_initial_state(&mut self, state_id: usize) {
        self.initial_state = Some(state_id);
    }

    pub(crate) fn toggle_final_state(&mut self, state_id: usize) {
        if !self.final_states.remove(&state_id) {
            self.final_states.insert(state_id);
        }
    }

    pub(crate) fn add_transition(&mut self, pair: (usize, usize), label: String) {
        self.transitions.entry(pair).or_default().insert(label);
    }

    /// An empty label set removes the transition.
    pub(crate) fn set_transition_labels(&mut self, pair: (usize, usize), labels: IndexSet<String>) {
        if labels.is_empty() {
            self.transitions.remove(&pair);
        } else {
            self.transitions.insert(pair, labels);
        }
    }

    pub(crate) fn delete_transition(&mut self, pair: (usize, usize)) {
        self.transitions.remove(&pair);
    }

    /// Hashes what a save persists: names, labels as sets, initial and finals, never positions.
    pub(crate) fn hash_content(&self, hasher: &mut impl Hasher) {
        let mut states: Vec<(usize, &str)> = self.states.iter().map(|node| (node.id, node.label.as_str())).collect();
        states.sort_unstable();
        states.hash(hasher);
        let mut transitions: Vec<((usize, usize), Vec<&str>)> = self
            .transitions
            .iter()
            .map(|(pair, labels)| {
                let mut labels: Vec<&str> = labels.iter().map(String::as_str).collect();
                labels.sort_unstable();
                (*pair, labels)
            })
            .collect();
        transitions.sort_unstable();
        transitions.hash(hasher);
        self.initial_state.hash(hasher);
        let mut finals: Vec<usize> = self.final_states.iter().copied().collect();
        finals.sort_unstable();
        finals.hash(hasher);
    }
}
