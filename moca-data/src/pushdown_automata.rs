use core::panic;
use std::collections::{HashMap, HashSet, BTreeSet};
use crate::state::{Input, State, StateID};
use crate::state_machine::StateMachine;

/* Structure that represents a pushdown automaton.
 * The inisital_state_id represents the initial state
 * of the automaton, if the value is None, then some 
 * algorithms and functions will not work.
 * The string_transitions field is used to store all
 * the string transitions the automaton has.
 * The stack represents the stack of the pushdown automaton.
 */
#[derive(Debug, Clone)]
pub struct PushdownAutomata {
    states_by_id: HashMap<StateID, State>,
    string_transitions: HashMap<(StateID, String), (StateID, String)>,
    initial_state_id: Option<StateID>,
    final_states: HashSet<StateID>,
    initial_stack_symbol: String,
    deterministic: bool,
}

impl PushdownAutomata {
    /* Only the initial symbol is needed because the stack is only used 
     * to operate the automata, otherwise is not necessary. */
    pub fn new(initial_stack_symbol: String) -> Self {
        PushdownAutomata {
            states_by_id: HashMap::new(),
            string_transitions: HashMap::new(),
            initial_state_id: None,
            final_states: HashSet::new(),
            initial_stack_symbol,
            deterministic: true,
        }
    }

    // Getter for the string transitions of the automata,
    pub fn get_string_transitions(&self) -> &HashMap<(StateID, String), (StateID, String)> {
        &self.string_transitions
    }

    /* Function to check if a given input string is accepted by the automata,
    * i.e. the final state is final and the input is consumed. 
    * This implementation works with acceptting states (final states). */
    pub fn check_input(&self, input: &mut Input) -> bool {
        match self.initial_state_id {
            Some(initial_id) => {
                let mut stack: Vec<String> = vec![self.initial_stack_symbol.to_string()];
                self.recursive_traversing(&initial_id, input, &mut stack)
            },
            None => todo!(), // Future implementation of an error
        }
    }

    // Function to add a label to a state given by it's id.
    pub fn add_label(&mut self, state_id: StateID, label: BTreeSet<StateID>) {
        if let Some(state) = self.states_by_id.get_mut(&state_id) {
            state.label = label;
        }
    }

    /* Function to traverse the automaton and checking the input string, i.e.
     * Check if the automaton accepts the input string. This function is very
     * similar to the implementation in the finite_automaton function with 
     * the same name, with the stack added. */
    fn recursive_traversing(&self, state_id: &StateID, input: &mut Input, stack: &mut Vec<String>) -> bool {
        match self.states_by_id.get(&state_id) {
            Some(state) => {
                print!("-------------------------------------\n");
                print!("The stack is {:?}\n", stack);
                print!("The input is {} and the state is {}\n", input, state_id);
                print!("-------------------------------------");
                if state.final_flag == true && input.is_empty() {
                    return true;
                }
                let mut string_matches_id: Vec<u64> = Vec::new();
                let mut stack_matches: Vec<String> = Vec::new();
                let mut string_ref = "";
                let mut string_len_max = 0;
                let mut accepted_bool = false;
                for (id, transition) in state.iter_by_transition() {
                    for string in transition.iter() {
                        if string == "ε" {
                            accepted_bool = accepted_bool || self.recursive_traversing(&id, &mut input.clone(), stack);
                            continue;
                        }
                        let string_transitions: Vec<&str> = string.split(';').collect();
                        let stack_transition: Vec<_> = string_transitions[1].split('/').collect();
                        if let Some(value) = stack.get(stack.len()-1) {
                            if stack_transition[0] != *value {
                               continue;
                            }
                        }
                        if input.starts_with(string_transitions[0]) {
                            if string.len() == string_len_max {
                                string_matches_id.push(*id);
                                stack_matches.push((string_transitions[1]).to_string());
                            }
                            else if string.len() > string_len_max {
                                string_len_max = string.len();
                                string_ref = string_transitions[0];
                                string_matches_id.clear();
                                stack_matches.clear();
                                string_matches_id.push(*id);
                                stack_matches.push((string_transitions[1]).to_string());
                            }
                        }
                        if (input.trim() == string_ref.trim()) && stack.len() == 1 {
                            stack_matches.push(string.clone());
                            string_matches_id.push(*id);
                        }
                    }
                }
                if (string_len_max == 0 && string_matches_id.is_empty()) &&
                    accepted_bool != true { return false; }
                input.replace_range(0..string_ref.len(),"");
                let mut count = 0;
                for id in string_matches_id {
                    let string = stack_matches[count].clone();
                    let stack_string: Vec<_> = string.split('/').map(|s| s.to_string()).collect();
                    self.stack_transition(stack_string[0].clone(), stack_string[1].clone(), stack);
                    accepted_bool = accepted_bool || self.recursive_traversing(&id, &mut input.clone(), stack);
                    count += count + 1;
                }
                return accepted_bool;
            }
            None => {return false;},
        }
    }

    // Auxiliar function to modify the stack given a stack change 
    fn stack_transition<'a> (&self, string1: String, string2: String, stack: &mut Vec<String>) {
        if string2.contains(&string1) && string2.len() > string1.len() {
            let (first,_) = string2.split_at(string1.len());
            stack.push(first.to_string())
        }
        else if string2 == "ε" {
            stack.pop();
        }
        else if string1 != string2 {
            stack.pop();
            stack.push(string2)
        }
    }
}

impl StateMachine for PushdownAutomata {
    fn get_states_by_id_mut_ref(&mut self) -> &mut HashMap<StateID, State> {
        &mut self.states_by_id
    }
    
    fn get_states_by_id_ref(&self) -> &HashMap<StateID, State> {
        &self.states_by_id
    }
    
    fn is_deterministic(&self) -> bool {
        self.deterministic
    }
    
    fn get_final_states(&self) -> &HashSet<StateID> {
        &self.final_states
    }
    
    fn get_initial_state_id(&self) -> &Option<StateID> {
        &self.initial_state_id
    }

    /* The implementation for finite automaton checks if the automaton
     * is deterministic or not. */
    fn add_transition(&mut self, state_id1: StateID, state_id2: StateID, input: Input) {
        let mut input_clone = input.clone();
        if input == "ε" {
            // This is for ease to use
            input_clone = "ε;ε".to_string();
            self.deterministic = false;
        }
        let transition: Vec<_> = input_clone.split(';').collect();
        // The second condition dictates that the automata is non deterministic, because if the
        // stack transition is a ε-transition and there exists another input transition with the
        // same symbol, then if the input is equal (the get returns a value) then the automata can
        // take 2 different paths with the same input transition
        if let Some(value) = self.string_transitions.get(&(state_id1, transition[0].to_string())) {
            if (value.0 != state_id2 && value.1 == transition[1]) || (value.1 == "ε") {
                self.deterministic = false;
            }
        }
        match self.states_by_id.get_mut(&state_id1) {
            Some(state) => {
                    state.add_transition(state_id2, input);
                    self.string_transitions.insert((state_id1, transition[0].to_string()), (state_id2, transition[1].to_string()));
            },
            None => (),
        }
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

    /* Function to make a state final. */
    // It has to do it in the particular module because of the mutability of the structure fields.
    fn make_final(&mut self, state_id: StateID) {
        match self.states_by_id.get_mut(&state_id) {
            Some(state) => {
                state.final_flag = true;
                self.final_states.insert(state_id);
            }
            None => panic!("The states does not exist."),
        }
    }
}
