use std::collections::HashMap;
use std::collections::hash_map::Iter;
use crate::state::{StateID, Input, State};
use crate::state_machine::StateMachine;

/* Structure that represent a finite automaton.
 * The initial_state_id represents the initial state
 * of the automaton, if the value in None, then some
 * algorithms and functions will not work. */
#[derive(Debug)]
pub struct FiniteAutomaton {
    states_by_id: HashMap<StateID, State>,
    initial_state_id: Option<u64>
}

impl FiniteAutomaton {
    pub fn new() -> Self {
        FiniteAutomaton {
            states_by_id: HashMap::new(),
            initial_state_id: None,
        }
    }

    /* Function to check if a given input string is accepted by the automata,
    * i.e. the final state is final. */
    pub fn check_input(&self, input: &mut Input) -> bool {
        match self.initial_state_id {
            Some(initial_id) => {
                self.recursive_traversing(&initial_id, input)
            },
            None => todo!(), // implement an error or exception because there is not initial state.
        }
    }

    /* Auxiliary recursive function to travel between states.
     * string_ref, string_len_max and string_id are used to know the largest
     * substring in the set of transitions, this is do it like this
     * to have a more accurate input reading than reading one character at
     * a time. The input is "consumed" if there is a transition valid from
     * one state to another, and that new input is passed in the recursive
     * function. */
    fn recursive_traversing(&self, state_id: &StateID, input: &mut Input) -> bool {
        if let Some(state) = self.states_by_id.get(&state_id) {
            if state.final_flag == true && input.is_empty() {
                return true;
            }
            let mut string_ref = "";
            let mut string_len_max = 0;
            let mut string_id = 0;
            for (id, transition) in state.iter_by_transition() {
                for string in transition.iter() {
                    if input.starts_with(string) {
                        if string.len() > string_len_max {
                            string_len_max = string.len();
                            string_ref = string;
                            string_id = *id;
                        }
                    }
                }
            }
            if string_len_max == 0 {
                return false;
            }
            input.replace_range(0..string_ref.len(),"");
            return self.recursive_traversing(&string_id, input);
        }
        false
    }
}

impl StateMachine for FiniteAutomaton {
    fn get_states_by_id_mut_ref(&mut self) -> &mut HashMap<StateID, State> {
        &mut self.states_by_id
    }
    
    fn make_initial(&mut self, state_id: StateID) {
        // this part will be omitted in the future because the ui will not allow this. //
        match self.states_by_id.get(&state_id) {
            Some(_) => (),
            None => return,
        }
        ///////////////////////////////////////////////
        match self.initial_state_id {
            Some(old_id) => {
                if let Some(old_initial_state) = self.states_by_id.get_mut(&old_id) {
                    old_initial_state.initial_flag = false;
                }
            }
            None => (), 
        }
        if let Some(state) = self.states_by_id.get_mut(&state_id) {
            state.initial_flag = true;
            self.initial_state_id = Some(state_id);
        }
    }
}
