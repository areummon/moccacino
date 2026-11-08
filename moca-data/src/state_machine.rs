use std::collections::{HashMap, HashSet};
use crate::state::{StateID, Input, State};

pub trait Machine {
    fn accepts(&self, input: &str) -> bool;

    /// Why one transition label cannot run on this machine as it stands; `validate` applies it to
    /// every label, and the .ce loader to each label it reads so it can name the offending line.
    fn check_label(&self, _label: &str) -> Result<(), String> {
        Ok(())
    }

    fn validate(&self) -> Result<(), String>;
}

#[derive(Debug, Clone, Default)]
pub struct Graph {
    states_by_id: HashMap<StateID, State>,
    initial_state_id: Option<StateID>,
    final_states: HashSet<StateID>,
}

impl Graph {
    pub(crate) fn state(&self, id: StateID) -> Option<&State> {
        self.states_by_id.get(&id)
    }

    pub(crate) fn initial(&self) -> Option<StateID> {
        self.initial_state_id
    }

    pub(crate) fn is_final(&self, id: StateID) -> bool {
        self.final_states.contains(&id)
    }

    pub(crate) fn accepts_at(&self, id: StateID, remaining_input: &str) -> bool {
        self.is_final(id) && remaining_input.is_empty()
    }

    pub(crate) fn labels(&self) -> impl Iterator<Item = (StateID, StateID, &str)> {
        self.states_by_id
            .iter()
            .flat_map(|(from, state)| state.labels().map(move |(to, label)| (*from, to, label)))
    }

    pub(crate) fn validate_labels(
        &self,
        noun: &str,
        check: impl Fn(&str) -> Result<(), String>,
    ) -> Result<(), String> {
        if self.initial_state_id.is_none() {
            return Err(format!("The {noun} has no initial state."));
        }
        self.labels()
            .try_for_each(|(from, _, label)| check(label).map_err(|problem| format!("State {from}: {problem}.")))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Changed {
    State(StateID),
    All,
}

pub trait StateMachine {
    fn graph(&self) -> &Graph;

    fn graph_mut(&mut self) -> &mut Graph;

    fn is_deterministic(&self) -> bool;

    fn transitions_changed(&mut self, changed: Changed);

    fn get_states_by_id_ref(&self) -> &HashMap<StateID, State> {
        &self.graph().states_by_id
    }

    fn get_final_states(&self) -> &HashSet<StateID> {
        &self.graph().final_states
    }

    fn get_initial_state_id(&self) -> Option<StateID> {
        self.graph().initial_state_id
    }

    fn add_state(&mut self) -> StateID {
        let states_by_id = &mut self.graph_mut().states_by_id;
        let mut id = states_by_id.len() as StateID;
        while states_by_id.contains_key(&id) {
            id += 1;
        }
        states_by_id.insert(id, State::new(format!("q{id}")));
        id
    }

    fn add_state_with_id_label(&mut self, id: StateID, label: &str) {
        let replaced = self.graph_mut().states_by_id.insert(id, State::new(label.to_string()));
        if replaced.is_some() {
            self.transitions_changed(Changed::State(id));
        }
    }

    fn add_n_states(&mut self, n: u64) {
        for _ in 0..n {
            self.add_state();
        }
    }

    fn add_transition(&mut self, from: StateID, to: StateID, input: Input) {
        let states_by_id = &mut self.graph_mut().states_by_id;
        if !states_by_id.contains_key(&to) {
            return;
        }
        let Some(state) = states_by_id.get_mut(&from) else {
            return;
        };
        state.add_transition(to, input);
        self.transitions_changed(Changed::State(from));
    }

    fn modify_name(&mut self, state_id: StateID, new_name: String) {
        if let Some(state) = self.graph_mut().states_by_id.get_mut(&state_id) {
            state.name = new_name;
        }
    }

    fn modify_input(&mut self, from: StateID, to: StateID, old_input: &str, new_input: Input) {
        if let Some(state) = self.graph_mut().states_by_id.get_mut(&from) {
            state.modify_input(to, old_input, new_input);
            self.transitions_changed(Changed::State(from));
        }
    }

    fn remove_transition(&mut self, from: StateID, to: StateID, input: &str) {
        if let Some(state) = self.graph_mut().states_by_id.get_mut(&from) {
            state.remove_transition(to, input);
            self.transitions_changed(Changed::State(from));
        }
    }

    fn remove_state(&mut self, state_id: StateID) {
        let graph = self.graph_mut();
        if graph.states_by_id.remove(&state_id).is_none() {
            return;
        }
        for state in graph.states_by_id.values_mut() {
            state.remove_state(state_id);
        }
        graph.final_states.remove(&state_id);
        if graph.initial_state_id == Some(state_id) {
            graph.initial_state_id = None;
        }
        self.transitions_changed(Changed::All);
    }

    fn make_initial(&mut self, state_id: StateID) {
        let graph = self.graph_mut();
        if graph.states_by_id.contains_key(&state_id) {
            graph.initial_state_id = Some(state_id);
        }
    }

    fn make_final(&mut self, state_id: StateID) {
        let graph = self.graph_mut();
        if graph.states_by_id.contains_key(&state_id) {
            graph.final_states.insert(state_id);
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedLabels<Op> {
    by_state: HashMap<StateID, Vec<(StateID, Op)>>,
}

impl<Op> Default for ParsedLabels<Op> {
    fn default() -> Self {
        ParsedLabels { by_state: HashMap::new() }
    }
}

impl<Op> ParsedLabels<Op> {
    pub(crate) fn refresh(&mut self, graph: &Graph, changed: Changed, parse: impl Fn(&str) -> Option<Op>) {
        let parse_state = |state: &State| -> Vec<(StateID, Op)> {
            state
                .labels()
                .filter_map(|(target, label)| parse(label).map(|op| (target, op)))
                .collect()
        };
        match changed {
            Changed::State(id) => match graph.state(id) {
                Some(state) => {
                    self.by_state.insert(id, parse_state(state));
                },
                None => {
                    self.by_state.remove(&id);
                },
            },
            Changed::All => {
                self.by_state = graph
                    .states_by_id
                    .iter()
                    .map(|(id, state)| (*id, parse_state(state)))
                    .collect();
            },
        }
    }

    pub(crate) fn of(&self, state_id: StateID) -> &[(StateID, Op)] {
        self.by_state.get(&state_id).map(Vec::as_slice).unwrap_or(&[])
    }

    pub(crate) fn all(&self) -> impl Iterator<Item = &(StateID, Op)> {
        self.by_state.values().flatten()
    }

    pub(crate) fn no_conflicts(&self, conflict: impl Fn(&(StateID, Op), &(StateID, Op)) -> bool) -> bool {
        self.by_state.values().all(|ops| {
            ops.iter()
                .enumerate()
                .all(|(index, first)| ops[index + 1..].iter().all(|second| !conflict(first, second)))
        })
    }
}
