use std::collections::{HashMap, HashSet, BTreeSet};
use crate::state::{StateID, Input, State};
use crate::state_machine::StateMachine;

/* Structure that represent a finite automaton.
 * The initial_state_id represents the initial state
 * of the automaton, if the value in None, then some
 * algorithms and functions will not work.
 * The string_transitions field is used to store all
 * the string transitions the automata have. */
#[derive(Debug, Default, Clone)]
pub struct FiniteAutomata {
    states_by_id: HashMap<StateID, State>,
    string_transitions: HashSet<String>,
    initial_state_id: Option<StateID>,
    final_states: HashSet<StateID>,
    deterministic: bool,
}

impl FiniteAutomata {
    pub fn new() -> Self {
        FiniteAutomata {
            states_by_id: HashMap::new(),
            string_transitions: HashSet::new(),
            initial_state_id: None,
            final_states: HashSet::new(),
            deterministic: true,
        }
    }

    pub fn clear(&mut self) {
        self.states_by_id.clear();
        self.string_transitions.clear();
        self.initial_state_id = None;
        self.final_states.clear();
        self.deterministic = true;
    }

    // Getter for the string transitions of the automata.
    pub fn get_string_transitions(&self) -> &HashSet<String> {
        &self.string_transitions
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

    // Function to add a label to a state given by it's id.
    pub fn add_label(&mut self, state_id: StateID, label: BTreeSet<StateID>) {
        if let Some(state) = self.states_by_id.get_mut(&state_id) {
            state.label = label;
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
     * If one string from the transitions is ε, it uses the function without
     * check the other conditions by definition. I also use clone on the input string because
     * in my implementation I "consume" it.
     * It works for both, NFA and DFA.
     */
    fn recursive_traversing(&self, state_id: &StateID, input: &mut Input) -> bool {
        let mut visited = HashSet::new();
        self.recursive_traversing_aux(state_id, input, &mut visited)
    }

    // Auxiliary function to traverse the automata. It prevents infinite recursion by checking which
    // state and input have been already visited. It also "consumes" the input. It's necessary because of 
    // the loops.
    fn recursive_traversing_aux(&self, state_id: &StateID, input: &mut Input, visited: &mut HashSet<(StateID, String)>) -> bool {
        let key = (*state_id, input.clone());
        if visited.contains(&key) {
            return false;
        }
        visited.insert(key);

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
                        if string == "ε" {
                            accepted_bool = accepted_bool || self.recursive_traversing_aux(&id, &mut input.clone(), visited);
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
                    accepted_bool != true { return false; } 
                input.replace_range(0..string_ref.len(),"");
                for id in string_matches_id {
                    accepted_bool = accepted_bool || self.recursive_traversing_aux(&id, &mut input.clone(), visited);
                }
                return accepted_bool;
            }
            None => {return false;},
        }
    }

    /*  ε-closure transition function of the DFA given a state and a string. */
    pub fn lambda_closure(&self, state_id: StateID, input_string: &str) -> BTreeSet<StateID> {
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
                        if string == "ε" || string == input_string {
                            // If statement to add the states with loops
                            if *id == state_id {
                                closure_set.insert(*id);
                            }
                            // To add the states that have a lambda transition
                            if string == "ε" && input_string.is_empty() {
                                closure_set.insert(*id);
                            }
                            valid_transitions += 1;
                            // It can have repeated states for the lambdas, but the replace handle
                            // it.
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
    pub fn to_dfa(&self) -> FiniteAutomata {
            if self.deterministic {
                panic!("For now this doesn't do anything, but it should return an Error()");
            }
            let subsets_and_transitions = subset_construction(&self);
            let mut states_by_id: HashMap<StateID, State> = HashMap::new();
            let mut id_by_subsets: HashMap<BTreeSet<StateID>, StateID> = HashMap::new();
            let mut new_initial_id = 0;
            let mut final_states: HashSet<StateID> = HashSet::new();
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
                            final_states.insert(id);
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
            FiniteAutomata {
                states_by_id,
                string_transitions: self.string_transitions.clone(),
                initial_state_id: Some(new_initial_id),
                final_states,
                deterministic: true,
            }
    }
    

    // Method that minimizes an automata only if it is deterministic, using the Hopcroft's
    // algorithm, and returns a copy of the automata minimized.
    pub fn minimize(&self)  -> Self  {
        if !self.deterministic {
            panic!("Cannon minimize a nfa");
        }
        let mut unreachable_states: Vec<StateID> = Vec::new();
        if let Some(initial_id) = self.initial_state_id {
            unreachable_states = get_unreachable_states(&self, initial_id);
        }
        let mut minimized_automata = self.clone();
        for id in unreachable_states {
            minimized_automata.remove_state(id);
        }
        convert_minimized_dfa(&minimized_automata, hopcroft_algorithm(&minimized_automata))
    }

    

    // The transition function of the automata.
    // Maps a state id and a string transition to a state that can be None if there is no
    // transition defined for that string.
    pub fn transition_function(&self, state_id: StateID, string: &str) -> Option<StateID> {
        if let Some(state) = self.states_by_id.get(&state_id) {
            for (id, transitions) in state.iter_by_transition() {
                for transition_string in transitions {
                    if string == transition_string {
                        return Some(*id);
                    }
                }
            }
        }
        None
    }
}

impl StateMachine for FiniteAutomata {
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
    
    /* The implementation for finite automata checks if the automaton
     * is deterministic or not. */
    fn add_transition(&mut self, state_id1: StateID, state_id2: StateID, mut input: Input) {
        if input == "ε" {
            self.deterministic = false;
        }
        match self.states_by_id.get_mut(&state_id2) {
            Some(_) => {
                if let Some(state) = self.states_by_id.get_mut(&state_id1) {
                    if input != "ε" {
                        self.string_transitions.replace(input.clone());
                    }
                    self.deterministic = state.add_transition(state_id2, input) && self.deterministic;
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

// Function that returns the unreacheable states of an automata as ids in a vector.
// The complexity is O(n+m) where n is the number of states and m is the number of transitions
// of the automaton.
pub fn get_unreachable_states(automata: &FiniteAutomata, initial_id: StateID) -> Vec<StateID> {
    let mut reachable_states: HashSet<StateID> = HashSet::new();
    let mut new_states: HashSet<StateID> = HashSet::new();
    reachable_states.insert(initial_id);
    new_states.insert(initial_id);
    while !new_states.is_empty() {
        let mut temp = HashSet::new();
        for state_id in new_states {
            for string in automata.string_transitions.iter() {
                match automata.transition_function(state_id, &string) {
                    Some(new_id) => { temp.insert(new_id); },
                    None => (),
                }
            }
        }
        new_states = temp.symmetric_difference(&reachable_states).cloned().collect();
        if new_states.is_subset(&reachable_states) {
            break;
        }
        reachable_states = reachable_states.union(&new_states).cloned().collect();
    }
    let mut unreachable_states = Vec::new();
    for (id, _) in automata.states_by_id.iter() {
        if !reachable_states.contains(id) {
            unreachable_states.push(*id);
        }
    }
    unreachable_states
}

// Hopcroft's algorithm for minimizing dfas, it works by using the nerode congruence, and defining
// partitions that are indistinguishable (for all input strings, δ(q,w) in any
// q in a subset lead to an acception/rejection state). The first partitions are in rejecting
// states and non-rejections states, and the algorithm finish when all subsets are equivalent (δ(q,w))
// in any q in a set leads to an accepting or rejection state). The implementation 
// This algorithm have O(ns log n) complexity time where n is the number of states and s the size of the alphabet.
// The function returns an equivalent (using the earlier definition) partition of state ids.
// This algorithm is adapted from https://en.wikipedia.org/wiki/DFA_minimization.
pub fn hopcroft_algorithm(automata: &FiniteAutomata) -> HashSet<BTreeSet<StateID>> {
    // Initial partition: split states into accepting and non-accepting
    let non_rejecting_states = hashmap_set_difference(automata.get_states_by_id_ref(),
                                                    automata.get_final_states());
    let mut partition_p: HashSet<BTreeSet<StateID>> = HashSet::new();
    let rejecting_states: BTreeSet<StateID> = automata.get_final_states().iter().cloned().collect();
    
    if !rejecting_states.is_empty() {
        partition_p.insert(rejecting_states);
    }
    if !non_rejecting_states.is_empty() {
        partition_p.insert(non_rejecting_states);
    }

    let mut partition_w: Vec<BTreeSet<StateID>> = partition_p.iter().cloned().collect();
    let mut visited_subsets: HashSet<BTreeSet<StateID>> = HashSet::new();

    while let Some(set_a) = partition_w.pop() {
        if visited_subsets.contains(&set_a) {
            continue;
        }
        visited_subsets.insert(set_a.clone());

        for string in automata.get_string_transitions() {
            let set_x = transition_function_set(automata, &set_a, string);
            if set_x.is_empty() {
                continue;
            }

            let mut new_partitions = Vec::new();
            let mut partitions_to_remove = Vec::new();

            for set_y in &partition_p {
                let x_y_intersection: BTreeSet<StateID> = set_y.intersection(&set_x).cloned().collect();
                let y_minus_x: BTreeSet<StateID> = set_y.difference(&set_x).cloned().collect();

                if !x_y_intersection.is_empty() && !y_minus_x.is_empty() {
                    partitions_to_remove.push(set_y.clone());
                    new_partitions.push(x_y_intersection);
                    new_partitions.push(y_minus_x);
                }
            }

            for partition in partitions_to_remove {
                partition_p.remove(&partition);
            }
            for partition in new_partitions {
                partition_p.insert(partition.clone());
                if !visited_subsets.contains(&partition) {
                    partition_w.push(partition);
                }
            }
        }
    }

    partition_p
}


// Auxiliar function that returns the difference between a hashmap of states by ids, and 
// a hashset of ids.
fn hashmap_set_difference(map: &HashMap<StateID, State>, set: &HashSet<StateID>) -> BTreeSet<StateID> {
    let mut difference_set = BTreeSet::new();
    for (map_id, _) in map.iter() {
        if !set.contains(map_id) {
            difference_set.insert(*map_id);
        }
    }
    difference_set
}

// Auxiliar function for the hopcroft algorithm, that takes an automata and a subset of that
// automata as parameters, and returns state ids gotten by the transition function on the condition
// that the state id returned by the transition funciton have to be in the set given by the
// function.
fn transition_function_set(automata: &FiniteAutomata, set: &BTreeSet<StateID>, string: &str) -> BTreeSet<StateID> {
    let mut new_set = BTreeSet::new();
    for (id,_) in automata.get_states_by_id_ref() {
        if let Some(new_id) = automata.transition_function(*id, string) {
            if set.contains(&new_id) {
                new_set.insert(*id);
            }
        }
    }
    new_set
}

// Auxiliar function to convert an equivalent partition of an automaton
// to a deterministic automaton.
fn convert_minimized_dfa(automata: &FiniteAutomata, partition: HashSet<BTreeSet<StateID>>) -> FiniteAutomata {
    let mut state_id_by_label: HashMap<BTreeSet<StateID>, StateID> = HashMap::new();
    let mut index = 0;
    let mut minimized_automata = FiniteAutomata::new();
    let og_final_states = automata.get_final_states();
    for set in partition.into_iter() {
        minimized_automata.add_state();
        if let Some(initial_id) = automata.get_initial_state_id() {
            if set.contains(initial_id) {
                minimized_automata.make_initial(index);
            }
        }
        for id in set.iter() {
            if og_final_states.contains(id) {
                minimized_automata.make_final(index);
                break;
            }
        }
        minimized_automata.add_label(index, set.clone());
        state_id_by_label.insert(set, index);
        index += 1;
    }
    for (set, id) in state_id_by_label.iter() {
        for set_id in set.iter() {
            for string in automata.get_string_transitions() {
                if let Some(state_id) = automata.transition_function(*set_id, string) {
                    for (minimized_set, minimized_id) in state_id_by_label.iter() {
                        if minimized_set.contains(&state_id) {
                            minimized_automata.add_transition(*id, *minimized_id, string.to_string());
                            break;
                        }
                    }
                }
            }
            break;
        }
    }
    minimized_automata
}

// Powerset/subset construction algorithm to convert a NDA to DFA. It returns a HashMap of
// particular subsets mapped to a vector of tuples of the form (subset of ids, string) that
// represents the transition.
// For now the implementation is very inefficient (multiple clones), I plan to improve it in the
// future.
// The implementation uses BTreeSet instead of HashSet because it already have an
// implementation of a hasher, so it can be used as a key in a hashmap, also it is a set of
// id's so there is not a significant advantage to use either.
pub fn subset_construction(automata: &FiniteAutomata) -> HashMap<BTreeSet<StateID>, Vec<(BTreeSet<StateID>, &str)>> {
    let mut sets_to_visit: Vec<BTreeSet<StateID>> = Vec::new();
    let mut visited_sets: HashSet<BTreeSet<StateID>> = HashSet::new();
    let mut transitions_by_subsets: HashMap<BTreeSet<StateID>, Vec<(BTreeSet<StateID>,&str)>> = HashMap::new();
    
    let initial_id = match automata.initial_state_id {
        Some(id) => id,
        None => panic!("There is not an initial state.")
    };

    // Initialize with the initial state's closure
    let mut current_subset = automata.lambda_closure(initial_id, "");
    current_subset.insert(initial_id);
    sets_to_visit.push(current_subset.clone());
    transitions_by_subsets.insert(current_subset, Vec::new());

    while let Some(current_subset) = sets_to_visit.pop() {
        let mut vector_transitions: Vec<(BTreeSet<StateID>, &str)> = Vec::new();
        
        for string in automata.get_string_transitions() {
            let new_subset = lambda_closure_subset(automata, &current_subset, string);
            
            if !new_subset.is_empty() && !visited_sets.contains(&new_subset) {
                sets_to_visit.push(new_subset.clone());
                transitions_by_subsets.insert(new_subset.clone(), Vec::new());
                visited_sets.insert(new_subset.clone());
            }
            
            vector_transitions.push((new_subset, string));
        }

        if let Some(vector) = transitions_by_subsets.get_mut(&current_subset) {
            *vector = vector_transitions;
        }
    }

    transitions_by_subsets
}

fn lambda_closure_subset(automata: &FiniteAutomata, subset: &BTreeSet<StateID>, input_string: &str) -> BTreeSet<StateID> {
    let mut result: BTreeSet<StateID> = BTreeSet::new();
    let mut to_process: Vec<StateID> = subset.iter().cloned().collect();
    let mut processed: HashSet<StateID> = HashSet::new();

    while let Some(id) = to_process.pop() {
        if processed.contains(&id) {
            continue;
        }
        processed.insert(id);

        let closure = automata.lambda_closure(id, input_string);
        for state_id in &closure {
            if !processed.contains(state_id) {
                to_process.push(*state_id);
            }
        }
        result.extend(closure);
    }

    result
}





