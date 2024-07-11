use std::collections::{HashMap, HashSet, BTreeSet};
use bimap::hash::BiHashMap;
use std::collections::hash_map::Iter;
use crate::state::{StateID, Input, State};
use crate::state_machine::StateMachine;

/* Structure that represent a finite automaton.
 * The initial_state_id represents the initial state
 * of the automaton, if the value in None, then some
 * algorithms and functions will not work.
 * The string_transitions field is used to store all
 * the string transitions the automata have. */
#[derive(Debug)]
pub struct FiniteAutomaton {
    states_by_id: HashMap<StateID, State>,
    string_transitions: HashSet<String>,
    initial_state_id: Option<u64>,
    deterministic: bool,
}

impl FiniteAutomaton {
    pub fn new() -> Self {
        FiniteAutomaton {
            states_by_id: HashMap::new(),
            string_transitions: HashSet::new(),
            initial_state_id: None,
            deterministic: true,
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
    // ------ This note is to modify and optimize this function other day --------
    // ------ because I implemented this only to see if it would work -------
    /* Note: I create a vector of all the ids, because if a certain state have
     * multiple transitions with the same string (i.e. is non deterministic)
     * then i add them to the vector to apply te function recursively to all the
     * the ids from the string matches, this works because if a string returns true in starts_with
     * then all the strings that return true and have the same length are the same,
     * so I use and or with accepted_bool that is going to be the bool value of the function.
     * If there is one path that accepts the input, then the value will be true.
     * If one string from the transitions is λ, it uses the function without
     * check the other conditions by definition. I also use clone on the input string because
     * in my implementation I "consume" it.
     * It works for both, NFA and DFA.
     */
    fn recursive_traversing(&self, state_id: &StateID, input: &mut Input) -> bool {
        match self.states_by_id.get(&state_id) {
            Some(state) => {
                if state.final_flag == true && input.is_empty() {
                    return true;
                }
                let mut string_matches_id: Vec<u64> = Vec::new();
                let mut string_ref = "";
                let mut string_len_max = 0;
                let mut accepted_bool = false;
                for (id, transition) in state.iter_by_transition() {
                    for string in transition.iter() {
                        if string == "λ" {
                            accepted_bool = accepted_bool || self.recursive_traversing(&id, &mut input.clone());
                        }
                        if input.starts_with(string) {
                            if string.len() == string_len_max {
                                string_matches_id.push(*id);
                            }
                            else if string.len() > string_len_max {
                                string_len_max = string.len();
                                string_ref = string;
                                string_matches_id.clear();
                                string_matches_id.push(*id);
                            }
                        }
                    }
                }
                if (string_len_max == 0 || string_matches_id.is_empty()) &&
                    accepted_bool != true { return false;
                } 
                input.replace_range(0..string_ref.len(),"");
                for id in string_matches_id {
                    accepted_bool = accepted_bool || self.recursive_traversing(&id, &mut input.clone());
                }
                return accepted_bool;
            }
            None => {return false;},
        }
    }

    /* Function to transform an NFA to a DFA using the powerset construction
     * algorithm. */
    pub fn nfa_to_dfa(&mut self) //-> Self// 
                                 {
        if self.deterministic {
            panic!("For now this doesn't do anything");
        }
        let mut subsets_by_id: BiHashMap<u64, BTreeSet<u64>> = BiHashMap::new();
        let mut subsets_to_visit: Vec<StateID> = Vec::new();
        let mut subsets_and_transitions: HashMap<StateID,  Vec<(u64, &str)>> = HashMap::new();
        match self.initial_state_id {
            Some(id) => {
                let mut initial_subset = self.lambda_closure(id, "");
                initial_subset.insert(id);
                subsets_by_id.insert(0, initial_subset);
                subsets_to_visit.push(0);
                while !subsets_to_visit.is_empty() {
                            println!("SUbsets to visit: {:?}",subsets_to_visit);
                    let current_id = match subsets_to_visit.pop() {
                        Some(id) => id,
                        None => panic!("The id was not found.")
                    };
                    println!("Current id is: {:?}", current_id);
                    let mut index_aux = subsets_by_id.len() as u64;
                    let mut vector_aux: Vec<(u64, &str)> = Vec::new();
                    let mut subsets_to_add: Vec<BTreeSet<u64>> = Vec::new();
                    if let Some(subset) = subsets_by_id.get_by_left(&current_id) {
                        for string in self.string_transitions.iter() {
                            let new_subset = self.apply_lambda_closure(subset, string);
                            if new_subset.is_empty() {
                                continue;
                            }
                            println!("New_subset is {:?}", new_subset);
                            subsets_to_add.push(new_subset);
                            println!("The vector is {:?}",subsets_to_add);
                            subsets_to_visit.push(index_aux);
                            vector_aux.push((index_aux, string));
                            index_aux += 1;
                            println!("SUbsets to visit: {:?}",subsets_to_visit);
                        }
                    }
                    match subsets_and_transitions.get_mut(&(subsets_by_id.len() as u64)) {
                        Some(vec_ref) => { *vec_ref = vector_aux; },
                        None => { subsets_and_transitions.insert((subsets_by_id.len() as u64)-1, vector_aux); },
                    }
                    self.add_subsets(subsets_to_add, subsets_by_id.len() as u64, &mut subsets_by_id);
                    
                } 
            } 
            None => panic!("The automata doesn't have an initial state.")
        }
        println!("The bihashmap final is: {:?}", subsets_by_id);
        println!("The hashmap is: {:?}\n", subsets_and_transitions); 
    }

    /* Auxiliar function to add a to the subsets_by_id bihashmap from a vector of subsets. */
    fn add_subsets(&self, mut vector: Vec<BTreeSet<u64>>, bihashmap_len: u64, bihashmap: &mut BiHashMap<u64, BTreeSet<u64>>) {
        let mut index_aux = 0;
        for set in vector.into_iter() {
            if bihashmap.contains_right(&set) {
                continue;
            }
            bihashmap.insert(bihashmap_len + index_aux, set);
            index_aux += 1;
        }
    }

    /* Auxiliar function to create the next subset from a subset. */
    fn apply_lambda_closure(&self, subset: &BTreeSet<StateID>, input_string: &str) -> BTreeSet<StateID> {
        let mut subset_result: BTreeSet<StateID> = BTreeSet::new();
        for id in subset {
            subset_result = subset_result.union(&self.lambda_closure(*id, input_string)).cloned().collect();
        }
        subset_result
    }

    /*  λ-closure transition function of the DFA given a state and a string. */
    fn lambda_closure(&self, state_id: StateID, input_string: &str) -> BTreeSet<StateID> {
        let mut closure_set: BTreeSet<StateID> = BTreeSet::new();
        self.lambda_closure_aux(state_id, input_string,&mut closure_set);
        closure_set
    }


    /* The auxiliar recursive function of the lambda closure function. */
    fn lambda_closure_aux(&self, state_id: StateID, input_string: &str, closure_set: &mut BTreeSet<StateID>) {
        match self.states_by_id.get(&state_id) {
            Some(state) => {
                let mut valid_transitions = 0;
                for (id, transitions) in state.iter_by_transition() {
                    if closure_set.contains(id) {
                        continue
                    }
                    for string in transitions {
                        if string == "λ" || string == input_string {
                            valid_transitions += 1;
                            match input_string.strip_prefix(string) {
                                Some(new_input) => self.lambda_closure_aux(*id, new_input, closure_set),
                                None => self.lambda_closure_aux(*id, input_string, closure_set),
                            }
                        }
                    }
                }
                    if valid_transitions == 0 && input_string.len() == 0 {
                        closure_set.insert(state_id);
                }
            },
            None => panic!("This should never occurr"),
        }
    }
}

impl StateMachine for FiniteAutomaton {
    fn get_states_by_id_mut_ref(&mut self) -> &mut HashMap<StateID, State> {
        &mut self.states_by_id
    }
    
    fn get_deterministic_flag(&mut self) -> &mut bool {
        &mut self.deterministic
    }

    /* The implementation for finite automaton checks if the automaton
     * is deterministic or not. */
    fn add_transition(&mut self, state_id1: StateID, state_id2: StateID, mut input: Input) {
        if input.is_empty() {
            input.push_str("λ");
            self.deterministic = true;
        }
        match self.states_by_id.get_mut(&state_id2) {
            Some(_) => {
                if let Some(state) = self.states_by_id.get_mut(&state_id1) {
                    if input != "λ" {
                        self.string_transitions.replace(input.clone());
                    }
                    self.deterministic = state.add_transition(state_id2, input);
                }
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
}
