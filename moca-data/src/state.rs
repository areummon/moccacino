use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Iter;

/* Type to represent the unique id of a state. */
pub type StateID = u64;
/* Type to represent the input a trasntition has. */
pub type Input = String;

/* Struct that represents an state of a machine. 
 * The state has a hashmap because it is going to be used
 * in different structs, so implementing this feature instead
 * of the outer structure gives more practicality.
 * The input_transitions variable is used to save all string transitions.
 * The label is used only in automata reductions or transformations like NDA -> DFA. */
#[derive(PartialEq, Debug, Eq)]
pub struct State {
    pub name: String,
    transitions_by_id: HashMap<StateID, HashSet<Input>>,
    input_transitions: HashSet<String>,
    pub label: HashSet<StateID>,
    pub initial_flag: bool,
    pub final_flag: bool,
}

impl State {
    pub fn new(name: String) -> Self {
        Self {
            name,
            transitions_by_id: HashMap::new(),
            input_transitions: HashSet::new(),
            label: HashSet::new(),
            initial_flag: false,
            final_flag: false,
        }
    }

    /* Functon to add a transition to another state given it's id.
     * It uses replace nstead of insert to avoid having two same
     * input transitions. If there is a transition to another state
     * with the same input then the function returns true, and false 
     * otherwise. It checks in O(1) if a transition already exists. */
    pub fn add_transition(&mut self, state_id: StateID, input: Input) -> bool {
        let mut deterministic_flag = false;
        if self.input_transitions.contains(&input) {
            deterministic_flag = true;
        }
        self.input_transitions.replace(input.clone());
        self.transitions_by_id.entry(state_id).or_insert(HashSet::new()).replace(input);
        deterministic_flag
    }

    /* Function to remove a transition. */
    pub fn remove_transition(&mut self, state_id: StateID, input: &str) {
        if let Some(transitions) = self.transitions_by_id.get_mut(&state_id) {
            transitions.remove(input);
        }
    }

    /* Function to modify an input transition, in the current implementation
     * the program uses a hashset so to modify a input trasition, it has
     * to remove it and add the modified version. It uses replace in case 
     * the new input is already an input transition. */
    pub fn modify_input(&mut self, state_id: StateID, old_input: &str, new_input: Input) {
        if let Some(transitions) = self.transitions_by_id.get_mut(&state_id) {
            transitions.remove(old_input);
            transitions.replace(new_input);
        }
    }

    /* Function to remove an entry in the transitions HashMap in case
     * a state was removed. */
    pub fn remove_state(&mut self, state_id: StateID) {
        self.transitions_by_id.remove(&state_id);
    }

    /* Iterator for transitions_by_id hashmap. */
    pub fn iter_by_transition(&self) -> Iter<'_, StateID, HashSet<Input>> {
        self.transitions_by_id.iter()
    }
}

