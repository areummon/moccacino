use std::collections::{HashMap, HashSet, BTreeSet};
use std::collections::hash_map::Iter;

pub type StateID = u64;
pub type Input = String;

#[derive(PartialEq, Debug, Eq, Clone)]
pub struct State {
    pub name: String,
    transitions_by_id: HashMap<StateID, HashSet<Input>>,
    input_transitions: HashSet<String>,
    pub label: BTreeSet<StateID>,
    pub initial_flag: bool,
    pub final_flag: bool,
}

impl State {
    pub fn new(name: String) -> Self {
        Self {
            name,
            transitions_by_id: HashMap::new(),
            input_transitions: HashSet::new(),
            label: BTreeSet::new(),
            initial_flag: false,
            final_flag: false,
        }
    }

    pub fn add_transition(&mut self, state_id: StateID, input: Input) -> bool {
        let already_to_target = self
            .transitions_by_id
            .get(&state_id)
            .is_some_and(|labels| labels.contains(&input));
        let deterministic_flag = already_to_target || !self.input_transitions.contains(&input);
        self.input_transitions.replace(input.clone());
        self.transitions_by_id.entry(state_id).or_insert(HashSet::new()).replace(input);
        deterministic_flag
    }

    pub fn remove_transition(&mut self, state_id: StateID, input: &str) {
        if let Some(transitions) = self.transitions_by_id.get_mut(&state_id) {
            transitions.remove(input);
        }
        self.rebuild_input_transitions();
    }

    pub fn modify_input(&mut self, state_id: StateID, old_input: &str, new_input: Input) {
        if let Some(transitions) = self.transitions_by_id.get_mut(&state_id) {
            transitions.remove(old_input);
            transitions.replace(new_input);
        }
        self.rebuild_input_transitions();
    }

    pub fn remove_state(&mut self, state_id: StateID) {
        self.transitions_by_id.remove(&state_id);
        self.rebuild_input_transitions();
    }

    fn rebuild_input_transitions(&mut self) {
        self.input_transitions = self
            .transitions_by_id
            .values()
            .flat_map(|labels| labels.iter().cloned())
            .collect();
    }

    pub fn iter_by_transition(&self) -> Iter<'_, StateID, HashSet<Input>> {
        self.transitions_by_id.iter()
    }
}
