use std::collections::{HashMap, HashSet};

/* Type to represent the input a trasntition has. */
type Input = HashSet<String>;


/* Struct that represents an state of a machine.
 * The name has &'a str for better optimization.
 * The key of transition_states is the name of 
 * the transition state. */
#[derive(PartialEq, Debug, Eq, Hash)]
pub struct State<'a> {
    pub name: &'a str,
    transition_states: HashMap<&'a str, Input>,
    pub initial_flag: bool,
    pub final_flag: bool,
}

impl<'a> State<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            transition_states: HashMap::new(),
            initial_flag: false,
            final_flag: false,
        }
    }

    /* Function to add a transition state. */
    pub fn add_transition(&mut self, state_name: &'a str, input: String) {
        self.transition_states.entry(state_name).or_insert(HashSet::new()).replace(input);
    }

    /* Function to remove a transition. */
    pub fn remove_transition(&mut self, state_name: &'a str, input: String) {
        if let Some(transitions) = self.transition_states.get_mut(state_name) {
            transitions.remove(&input);
        }
    }

    /* Function to modify the input of a transition.
     * It has to remove the old input and add the new input in the
     * hashset, it might change in the future. */
    pub fn modify_input(&mut self, state_name: &'a str, input: String, new_input: String) {
        if let Some(transitions) = self.transition_states.get_mut(state_name) {
            transitions.remove(&input);
            transitions.insert(new_input);
        }
    }

    /* Function to update a modified state name. 
     * I remove and add a new entry with the new name to update the hashmap,
     * because I think the hashmap is a better option for the operations,
     * but it might change in the future. */
    pub fn update(&mut self, state_name: &'a str, new_name: &'a str) {
        if let Some(transitions) = self.transition_states.remove(state_name) {
            self.transition_states.insert(new_name, transitions);
        }
    }

    /* Getter of the transitions of a given state name. */
    pub fn get_transitions(&mut self, state_name: &str) -> Option<&Input> {
        self.transition_states.get(state_name)
    }
}

