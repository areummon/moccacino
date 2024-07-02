use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Iter;
use crate::state::{StateID, State};

/* Type that represents all the inputs for transitions to another state. */
type Input = String;
/* Type that represents a transition for a finite automaton.
 * It has a HashSet to avoid having same inputs. */
type Transitions = HashMap<StateID, HashSet<Input>>;

/* Structure that represent a finite automaton. */
#[derive(Debug)]
pub struct FiniteAutomaton {
    states_by_id: HashMap<StateID, State>,
    state_transitions_by_id: HashMap<StateID, Transitions>,
}

impl FiniteAutomaton {
    pub fn new() -> Self {
        FiniteAutomaton { 
            states_by_id: HashMap::new(),
            state_transitions_by_id: HashMap::new(),
        }
    }
    
    /* The name is assigned automatically as well as the id. */
    pub fn add_state(&mut self) {
        let state_name = format!("q{}", self.states_by_id.len());
        self.states_by_id.insert(self.states_by_id.len() as u64, State::new(state_name));
    }

    /* Functon to add a transition between two given states.
     * The transition goes from state1 to state2. It uses replace 
     * instead of insert to avoid having two same input transitions.
     * It also checks if a given id/state exists, if not, then it doesn't 
     * add it. */
    pub fn add_transition(&mut self, state_id1: StateID, state_id2: StateID, input: Input) {
        if !self.states_by_id.contains_key(&state_id2) { return; }
        self.state_transitions_by_id.entry(state_id1).or_insert(HashMap::new());
        if let Some(transitions) = self.state_transitions_by_id.get_mut(&state_id1) {
            transitions.entry(state_id2).or_insert(HashSet::new()).replace(input);
        }
    }

    pub fn modify_name(&mut self, state_id: StateID, new_name: String) {
        if let Some(state) = self.states_by_id.get_mut(&state_id) {
            state.name = new_name;
        }
    }

    /* Function to modify an input transition, in the current implementation
     * the program uses a hashset so to modify a input trasition, it has
     * to remove it and add the modified version. It uses replace in case 
     * the new input is already an input transition. */
    pub fn  modify_input(&mut self, state_id: StateID, state_transition_id: StateID,
                         old_input: &str, new_input: Input) {
        if let Some(transitions) = self.state_transitions_by_id.get_mut(&state_id) {
            if let Some(state_transitions) = transitions.get_mut(&state_transition_id) {
                state_transitions.remove(old_input);
                state_transitions.replace(new_input);
            }
        }
    }

    /* Function to delete a transition from one state to another. */
    pub fn remove_transition(&mut self, state_id: StateID, state_transition_id: StateID, input: &str) {
        if let Some(transitions) = self.state_transitions_by_id.get_mut(&state_id) {
            if let Some(state_transitions) = transitions.get_mut(&state_transition_id) {
                state_transitions.remove(input);
            }
        }
    }

    /* Function to delete a state, this implies that it's id will be removed
     * from all the transitions with another state. */
    pub fn remove_state(&mut self, state_id: StateID) {
        self.states_by_id.remove(&state_id);
        self.state_transitions_by_id.remove(&state_id);
        for (_, transitions) in self.state_transitions_by_id.iter_mut() {
            transitions.remove(&state_id);
        }
    }

    /* Function to make a state initial. */
    pub fn make_initial(&mut self, state_id: StateID) {
        if let Some(state) = self.states_by_id.get_mut(&state_id) {
            state.initial_flag = true;
        }
    }

    /* Function to make a state final. */
    pub fn make_final(&mut self, state_id: StateID) {
        if let Some(state) = self.states_by_id.get_mut(&state_id) {
            state.final_flag = true;
        }
    }

    /* Iterator for the states_by_id HashMap. */
    pub fn iter_by_state(&self) -> Iter<'_, StateID, State> {
        self.states_by_id.iter()
    }
   
    /* Iterator for state_transitions_by_id HashMap. */
    pub fn iter_by_transition(&self) -> Iter<'_, StateID, Transitions> {
        self.state_transitions_by_id.iter()
    }
}
