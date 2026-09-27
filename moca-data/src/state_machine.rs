use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Iter;
use crate::state::{StateID, Input, State};

/* Family of formal machines modeled by the library. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineKind {
    Finite,
    Pushdown,
    Turing,
}

/* Behavioral interface shared by every machine of the library, independent of
 * the structural StateMachine trait. New machine types (e.g. Turing machines)
 * implement this to expose a uniform acceptance and validation API.
 *
 * Transition label conventions per family:
 * - Finite: plain symbol strings; both "" and "ε" mean the empty transition.
 * - Pushdown: "input;pop/push" ("ε" as pop or push means no operation).
 * - Turing (planned): "read;write/dir" with dir in {L, R, S}; multitape
 *   transitions will join per-tape operations with commas.
 */
pub trait Machine {
    /* Family of this machine. */
    fn kind(&self) -> MachineKind;

    /* Check if the machine accepts the input string. */
    fn accepts(&self, input: &str) -> bool;

    /* Check semantic validity: an initial state exists and every transition
     * references states that exist. */
    fn validate(&self) -> Result<(), String>;
}

pub trait StateMachine {

    /* Getter of a mutable reference of the hashmap to define the 
     * implementations of many functions of a state machine. */
    fn get_states_by_id_mut_ref(&mut self) -> &mut HashMap<StateID, State>;

    /* Getter of a reference of the hashmap to define the 
     * implementations of many functions of a state machine. */
    fn get_states_by_id_ref(&self) -> &HashMap<StateID, State>;

    /* Getter of the deterministic flag. */
    fn is_deterministic(&self) -> bool;

    /* Getter of the final states of the machine. */
    fn get_final_states(&self) -> &HashSet<StateID>;

    /* Getter of the initial state id. */
    fn get_initial_state_id(&self) -> &Option<StateID>;

    /* The name is assigned automatically as well as the id: the first free
     * id from the current state count upwards (the count itself while ids
     * are dense), so a state is never overwritten after deletions. Returns
     * the assigned id. */
    fn add_state(&mut self) -> StateID {
        let states_by_id = self.get_states_by_id_mut_ref();
        let mut id = states_by_id.len() as StateID;
        while states_by_id.contains_key(&id) {
            id += 1;
        }
        states_by_id.insert(id, State::new(format!("q{}", id)));
        id
    }

    /* Another adding method that asigns a state given a label. */
    fn add_state_with_id_label(&mut self, id: u64, label: &str) {
        let states_by_id = self.get_states_by_id_mut_ref();
        let state_name = label;
        states_by_id.insert(id, State::new(state_name.to_string()));
    }

    /* Convenient function to add n states. */
    fn add_n_states(&mut self, n: u64) {
        for _ in 0..n {
            self.add_state();
        }
    }

    /* Functon to add a transition between two given states.
     * The transition goes from state1 to state2. It also checks
     * if a given id/state exists, if not, then it doesn't add it. */
    fn add_transition(&mut self, state_id1: StateID, state_id2: StateID, input: Input);
    
    /* Modify the name of a state. */
    fn modify_name(&mut self, state_id: StateID, new_name: String) {
        let states_by_id = self.get_states_by_id_mut_ref();
        if let Some(state) = states_by_id.get_mut(&state_id) {
            state.name = new_name;
        }
    }

    /* Function to modify an input transition between two states. */
    fn modify_input(&mut self, state_id: StateID, state_transition_id: StateID,
                        old_input: &str, new_input: Input) {
        let states_by_id = self.get_states_by_id_mut_ref();
        if let Some(state) = states_by_id.get_mut(&state_id) {
            state.modify_input(state_transition_id, old_input, new_input);
        }
    }

    /* Hook that lets each machine clean its own bookkeeping (final state
     * registries, initial state id, transition tables) when a state is
     * deleted, so no dangling references to removed states are left behind. */
    fn forget_state(&mut self, _state_id: StateID) {}

    /* Function to delete a state, this implies that it's id will be removed
     * from all the transitions with another state. */
    fn remove_state(&mut self, state_id: StateID) {
        let states_by_id = self.get_states_by_id_mut_ref();
        if let Some(_) = states_by_id.get_mut(&state_id) {
            states_by_id.remove(&state_id);
            for (_, states) in states_by_id.iter_mut() {
                states.remove_state(state_id);
            }
            self.forget_state(state_id);
        }
    }

    /* Function to delete a transition from one state to another. */
    fn remove_transition(&mut self, state_id: StateID, state_transition_id: StateID, input: &str) {
        let states_by_id = self.get_states_by_id_mut_ref();
        if let Some(state) = states_by_id.get_mut(&state_id) {
            state.remove_transition(state_transition_id, input);
        }
    }

    /* Function to make a state initial.
     * If the machine already has a initial state, then it makes the it changes
     * the initial_flag from the state to false and then modify the new one,
     * if the new one does not exist, then the old one remain unchanged. */
    fn make_initial(&mut self, state_id: StateID); 

    /* Function to make a state final. */
    fn make_final(&mut self, state_id: StateID); 

    /* Iterator for the states_by_id HashMap. */
    fn iter_by_state(&mut self) -> Iter<'_, StateID, State> {
        let states_by_id = self.get_states_by_id_mut_ref();
        states_by_id.iter()
    }
}
