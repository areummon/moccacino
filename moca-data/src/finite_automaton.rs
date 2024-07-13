use std::collections::{HashMap, HashSet, BTreeSet};
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
    // For now the implementation is very inefficient (multiple clones), I plan to improve it in the
    // future.
    // The implementation uses BTreeSet instead of HashSet because it already have an
    // implementation of a hasher, so it can be used as a key in a hashmap, also it is a set of
    // id's so there is not a significant advantage to use either.
    pub fn nfa_to_dfa(&mut self) -> Self {
        if self.deterministic {
            panic!("For now this doesn't do anything, but it should return an Error()");
        }
        // This act as a stack to check every new subset gotten from the lambda closure function
        let mut sets_to_visit: Vec<BTreeSet<StateID>> = Vec::new();
        // This is used to not add visited sets to sets_to_visit vector
        let mut visited_sets: HashSet<BTreeSet<StateID>> = HashSet::new();
        // This is used to store all the subsets and their transitions in a table-like form, this
        // is used to construct the resulting dfa automaton.
        let mut transitions_by_subsets: HashMap<BTreeSet<StateID>, Vec<(BTreeSet<StateID>,&str)>> = HashMap::new();
        let initial_id = match self.initial_state_id {
            Some(id) => id,
            None => panic!("There is not an initial state.")
        };
        let mut current_subset = self.lambda_closure(initial_id, "");
        current_subset.insert(initial_id); // This line is required in this implementation.
        sets_to_visit.push(current_subset.clone());
        transitions_by_subsets.insert(current_subset, Vec::new());
        while !sets_to_visit.is_empty() {
            let mut vector_transitions: Vec<(BTreeSet<u64>, &str)> = Vec::new();
            let current_subset = match sets_to_visit.pop() {
                Some(set) => {
                    set
                },
                None => panic!("There is no subset, this should never occur"),
            };
            
            for string in self.string_transitions.iter() {
                let new_subset = self.lambda_closure_subset(&current_subset, string);
                if new_subset.is_empty() || visited_sets.contains(&new_subset) {
                    vector_transitions.push((new_subset, string));
                    continue;
                }
                sets_to_visit.push(new_subset.clone());
                transitions_by_subsets.insert(new_subset.clone(), Vec::new());
                visited_sets.insert(new_subset.clone());
                vector_transitions.push((new_subset, string));
            }
            // It needs to do this because when adding an entry, It needs to add a subset and a
            // vector.
            if let Some(vector) = transitions_by_subsets.get_mut(&current_subset) {
                *vector = vector_transitions;
            }
        }
        self.transform_to_dfa(transitions_by_subsets)
    }

    /* λ-closure function given a subset as a parameter. */
    fn lambda_closure_subset(&self, subset: &BTreeSet<StateID>, input_string: &str) -> BTreeSet<StateID> {
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
    // If the state_id is the initial id of the automaton, then it will not be added in this
    // function, so it needs to be added outside the function. It also "consumes" the input.
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
                        closure_set.replace(state_id);
                }
            },
            None => panic!("This should never occurr"),
        }
    }

    /* Auxiliar function that takes a hahsmap of btreesets of u64 mapped to 
     * a vector of tuples (btreeset<u64>, &str) that represents the transitions
     * given by the subset construction algorithm. */
    fn transform_to_dfa(&self, subsets_and_transitions: HashMap<BTreeSet<StateID>, Vec<(BTreeSet<StateID>, &str)>>)
        -> Self {
            let mut states_by_id: HashMap<StateID, State> = HashMap::new();
            let mut id_by_subsets: HashMap<BTreeSet<StateID>, StateID> = HashMap::new();
            let mut new_initial_id = 0;
            let mut id = 0;
            // It needs to be iterated two times because in the first iteration it might not know
            // what is the id of a subset in a transition.
            for (subset, _) in subsets_and_transitions.iter() {
                let mut state = State::new(format!("q{}", id));
                if let Some(initial_id) = self.initial_state_id {
                    let mut initial_subset = self.lambda_closure(initial_id,"");
                    initial_subset.insert(initial_id);
                    if initial_subset == *subset {
                        new_initial_id = id;
                        state.initial_flag = true;
                    }
                }
                for current_id in subset {
                    if let Some(current_state) = self.states_by_id.get(current_id) {
                        if current_state.final_flag == true {
                            state.final_flag = true;
                            break;
                        }
                    }
                }
                id_by_subsets.insert(subset.clone(), id);
                state.label = subset.clone();
                states_by_id.insert(id, state);
                id += 1;
            }
            id = 0;
            for (_, transitions) in subsets_and_transitions {
                if let Some(state) = states_by_id.get_mut(&id) {
                    for (set, string) in transitions {
                        if let Some(id) = id_by_subsets.get(&set) {
                            state.add_transition(*id, string.to_string());
                        }
                    }
                }
                id += 1;
            }
            FiniteAutomaton {
                states_by_id: states_by_id,
                string_transitions: self.string_transitions.clone(),
                initial_state_id: Some(new_initial_id),
                deterministic: true,
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
