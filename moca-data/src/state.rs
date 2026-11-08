use std::collections::{BTreeMap, BTreeSet};
use std::collections::btree_map::Iter;

pub type StateID = u64;
pub type Input = String;

pub const EPSILON: &str = "ε";

pub fn is_epsilon(label: &str) -> bool {
    label.is_empty() || label == EPSILON
}

#[derive(PartialEq, Debug, Eq, Clone)]
pub struct State {
    pub name: String,
    transitions_by_id: BTreeMap<StateID, BTreeSet<Input>>,
}

impl State {
    pub fn new(name: String) -> Self {
        Self {
            name,
            transitions_by_id: BTreeMap::new(),
        }
    }

    pub(crate) fn add_transition(&mut self, state_id: StateID, input: Input) {
        self.transitions_by_id.entry(state_id).or_default().insert(input);
    }

    pub(crate) fn remove_transition(&mut self, state_id: StateID, input: &str) {
        if let Some(transitions) = self.transitions_by_id.get_mut(&state_id) {
            transitions.remove(input);
        }
    }

    pub(crate) fn modify_input(&mut self, state_id: StateID, old_input: &str, new_input: Input) {
        if let Some(transitions) = self.transitions_by_id.get_mut(&state_id) {
            transitions.remove(old_input);
            transitions.insert(new_input);
        }
    }

    pub(crate) fn remove_state(&mut self, state_id: StateID) {
        self.transitions_by_id.remove(&state_id);
    }

    pub fn iter_by_transition(&self) -> Iter<'_, StateID, BTreeSet<Input>> {
        self.transitions_by_id.iter()
    }

    pub fn labels(&self) -> impl Iterator<Item = (StateID, &str)> {
        self.transitions_by_id
            .iter()
            .flat_map(|(target, labels)| labels.iter().map(move |label| (*target, label.as_str())))
    }
}
