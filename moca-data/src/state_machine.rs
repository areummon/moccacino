use std::collections::{HashMap};
use std::collections::hash_map::Iter;
use crate::state::{StateID, Input, State};

pub trait StateMachine {

    /* Getter of a mutable reference of the hashmap to define the 
     * implementations of many functions of a state machine. */
    fn get_states_by_id_mut_ref(&mut self) -> &mut HashMap<StateID, State>;

    /* Getter of the deterministic flag. */
    fn get_deterministic_flag(&mut self) -> &mut bool;


    /* The name is assigned automatically as well as the id. */
    fn add_state(&mut self) {
        let states_by_id = self.get_states_by_id_mut_ref();
        let states_by_id_len = states_by_id.len();
        let state_name = format!("q{}", states_by_id.len());
        states_by_id.insert(states_by_id_len as u64, State::new(state_name));
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
    fn add_transition(&mut self, state_id1: StateID, state_id2: StateID, mut input: Input) {
        if input.is_empty() {
            input.push_str("λ");
            let deterministic_flag = self.get_deterministic_flag();
            *deterministic_flag = true;
        }
        let states_by_id = self.get_states_by_id_mut_ref();
        match states_by_id.get_mut(&state_id2) {
            Some(state2) => {
                if let Some(state) = states_by_id.get_mut(&state_id1) {
                    state.add_transition(state_id2, input);
                }
            },
            None => (),
        }
    }
    
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

    /* Function to delete a state, this implies that it's id will be removed
     * from all the transitions with another state. */
    fn remove_state(&mut self, state_id: StateID) {
        let states_by_id = self.get_states_by_id_mut_ref();
        if let Some(state) = states_by_id.get_mut(&state_id) {
            states_by_id.remove(&state_id);
            for (_, states) in states_by_id.iter_mut() {
                states.remove_state(state_id);
            }
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
    fn make_final(&mut self, state_id: StateID) {
        let states_by_id = self.get_states_by_id_mut_ref();
        if let Some(state) = states_by_id.get_mut(&state_id) {
            state.final_flag = true;
        }
    }

    /* Iterator for the states_by_id HashMap. */
    fn iter_by_state(&mut self) -> Iter<'_, StateID, State> {
        let states_by_id = self.get_states_by_id_mut_ref();
        states_by_id.iter()
    }
}
