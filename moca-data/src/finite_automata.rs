use std::collections::{HashMap, HashSet, BTreeSet};
use crate::state::{StateID, Input, State};
use crate::state_machine::{Machine, MachineKind, StateMachine};
use crate::regex::ast::RegexAst;

#[derive(Debug, Default, Clone)]
pub struct FiniteAutomata {
    states_by_id: HashMap<StateID, State>,
    string_transitions: HashSet<String>,
    initial_state_id: Option<StateID>,
    final_states: HashSet<StateID>,
    deterministic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FiniteConfiguration {
    state_id: StateID,
    remaining_input: Input,
}

impl FiniteConfiguration {
    pub fn state_id(&self) -> StateID {
        self.state_id
    }

    pub fn remaining_input(&self) -> &Input {
        &self.remaining_input
    }
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

    pub fn get_string_transitions(&self) -> &HashSet<String> {
        &self.string_transitions
    }

    pub fn check_input(&self, input: &mut Input) -> bool {
        match self.initial_state_id {
            Some(initial_id) => {
                self.recursive_traversing(&initial_id, input)
            },
            None => false,
        }
    }

    pub fn initial_configuration(&self, input: &str) -> Option<FiniteConfiguration> {
        let initial_id = self.initial_state_id?;
        Some(FiniteConfiguration {
            state_id: initial_id,
            remaining_input: input.to_string(),
        })
    }

    pub fn is_accepting(&self, config: &FiniteConfiguration) -> bool {
        self.final_states.contains(&config.state_id) && config.remaining_input.is_empty()
    }

    pub fn step_all(&self, config: &FiniteConfiguration) -> Vec<FiniteConfiguration> {
        let mut successors: Vec<FiniteConfiguration> = Vec::new();
        if let Some(state) = self.states_by_id.get(&config.state_id) {
            for (next_id, transitions) in state.iter_by_transition() {
                for label in transitions.iter() {
                    if label.is_empty() || label == "ε" {
                        successors.push(FiniteConfiguration {
                            state_id: *next_id,
                            remaining_input: config.remaining_input.clone(),
                        });
                    } else if config.remaining_input.starts_with(label.as_str()) {
                        let mut rest = config.remaining_input.clone();
                        rest.replace_range(0..label.len(), "");
                        successors.push(FiniteConfiguration {
                            state_id: *next_id,
                            remaining_input: rest,
                        });
                    }
                }
            }
        }
        successors.sort_by(|a, b| {
            (a.state_id, a.remaining_input.as_str())
                .cmp(&(b.state_id, b.remaining_input.as_str()))
        });
        successors
    }

    fn refresh_determinism(&mut self) {
        let mut deterministic = true;
        let mut alphabet: HashSet<String> = HashSet::new();
        for state in self.states_by_id.values() {
            let mut target_of: HashMap<&str, StateID> = HashMap::new();
            for (target, labels) in state.iter_by_transition() {
                for label in labels {
                    if is_epsilon_label(label) {
                        deterministic = false;
                        continue;
                    }
                    alphabet.insert(label.clone());
                    if let Some(previous) = target_of.insert(label.as_str(), *target) {
                        if previous != *target {
                            deterministic = false;
                        }
                    }
                }
            }
        }
        self.deterministic = deterministic;
        self.string_transitions = alphabet;
    }

    pub fn add_label(&mut self, state_id: StateID, label: BTreeSet<StateID>) {
        if let Some(state) = self.states_by_id.get_mut(&state_id) {
            state.label = label;
        }
    }

    fn recursive_traversing(&self, state_id: &StateID, input: &mut Input) -> bool {
        let mut visited = HashSet::new();
        self.recursive_traversing_aux(state_id, input, &mut visited)
    }

    fn recursive_traversing_aux(&self, state_id: &StateID, input: &mut Input, visited: &mut HashSet<(StateID, String)>) -> bool {
        let key = (*state_id, input.clone());
        if visited.contains(&key) {
            return false;
        }
        visited.insert(key);

        match self.states_by_id.get(state_id) {
            Some(state) => {
                if state.final_flag && input.is_empty() {
                    return true;
                }
                let mut accepted_bool = false;
                for (id, transition) in state.iter_by_transition() {
                    for string in transition.iter() {
                        if string.is_empty() || string == "ε" {
                            accepted_bool = accepted_bool || self.recursive_traversing_aux(id, &mut input.clone(), visited);
                            continue;
                        }
                        if input.starts_with(string.as_str()) {
                            let mut rest = input.clone();
                            rest.replace_range(0..string.len(), "");
                            accepted_bool = accepted_bool || self.recursive_traversing_aux(id, &mut rest, visited);
                        }
                    }
                    if accepted_bool {
                        break;
                    }
                }
                return accepted_bool;
            }
            None => {return false;},
        }
    }

    pub fn lambda_closure(&self, state_id: StateID, input_string: &str) -> BTreeSet<StateID> {
        let mut closure_set: BTreeSet<StateID> = BTreeSet::new();
        self.lambda_closure_aux(state_id, &mut closure_set);
        if input_string.is_empty() {
            return closure_set;
        }
        let mut reachable_set: BTreeSet<StateID> = BTreeSet::new();
        for id in closure_set {
            if let Some(state) = self.states_by_id.get(&id) {
                for (next_id, transitions) in state.iter_by_transition() {
                    if transitions.contains(input_string) {
                        self.lambda_closure_aux(*next_id, &mut reachable_set);
                    }
                }
            }
        }
        reachable_set
    }

    fn lambda_closure_aux(&self, state_id: StateID, closure_set: &mut BTreeSet<StateID>) {
        if !closure_set.insert(state_id) {
            return;
        }
        if let Some(state) = self.states_by_id.get(&state_id) {
            for (id, transitions) in state.iter_by_transition() {
                for string in transitions.iter() {
                    if string.is_empty() || string == "ε" {
                        self.lambda_closure_aux(*id, closure_set);
                    }
                }
            }
        }
    }

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
            let initial_closure: Option<BTreeSet<StateID>> =
                self.initial_state_id.map(|initial_id| {
                    let mut closure = self.lambda_closure(initial_id, "");
                    closure.insert(initial_id);
                    closure
                });
            for (subset, _) in subsets_and_transitions.iter() {
                let mut state = State::new(format!("q{}", id));
                if let Some(initial_subset) = &initial_closure {
                    if initial_subset == subset {
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
            for (subset, transitions) in subsets_and_transitions {
                let Some(&from) = id_by_subsets.get(&subset) else { continue };
                for (set, string) in transitions {
                    if let Some(&to) = id_by_subsets.get(&set) {
                        if let Some(state) = states_by_id.get_mut(&from) {
                            state.add_transition(to, string.to_string());
                        }
                    }
                }
            }
            FiniteAutomata {
                states_by_id,
                string_transitions: self.string_transitions.clone(),
                initial_state_id: Some(new_initial_id),
                final_states,
                deterministic: true,
            }
    }

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

    pub fn to_regex(&self) -> RegexAst {
        use RegexAst::*;

        let mut edges: HashMap<(StateID, StateID), RegexAst> = HashMap::new();
        let mut union_edge = |from: StateID, to: StateID, label: &str| {
            let piece: RegexAst = if label == "ε" || label.is_empty() {
                Epsilon
            } else {
                let mut ast: Option<RegexAst> = None;
                for c in label.chars() {
                    let node = Char(c);
                    ast = Some(match ast {
                        Some(existing) => Concat(Box::new(existing), Box::new(node)),
                        None => node,
                    });
                }
                ast.expect("non-empty label")
            };
            match edges.get_mut(&(from, to)) {
                Some(existing) => {
                    *existing = Union(Box::new(std::mem::replace(existing, Epsilon)), Box::new(piece));
                },
                None => {
                    edges.insert((from, to), piece);
                },
            }
        };

        let initial_id = match self.initial_state_id {
            Some(id) => id,
            None => return Empty,
        };

        let reachable = {
            let mut visited: HashSet<StateID> = HashSet::from([initial_id]);
            let mut stack = vec![initial_id];
            while let Some(id) = stack.pop() {
                if let Some(state) = self.states_by_id.get(&id) {
                    for (target, _) in state.iter_by_transition() {
                        if visited.insert(*target) {
                            stack.push(*target);
                        }
                    }
                }
            }
            visited
        };

        let next_id = self.states_by_id.keys().copied().max().map_or(0, |max| max + 1);
        let new_start = next_id;
        let new_final = next_id + 1;

        union_edge(new_start, initial_id, "ε");
        for id in &reachable {
            if self.final_states.contains(id) {
                union_edge(*id, new_final, "ε");
            }
        }
        for id in &reachable {
            if let Some(state) = self.states_by_id.get(id) {
                for (target, labels) in state.iter_by_transition() {
                    if reachable.contains(target) {
                        for label in labels {
                            union_edge(*id, *target, label);
                        }
                    }
                }
            }
        }

        let mut remaining: Vec<StateID> = reachable.iter().copied().collect();
        remaining.sort();
        for r in remaining {
            let loop_ast = edges.get(&(r, r)).cloned();
            let loop_star = loop_ast.map(|ast| Star(Box::new(ast)));
            let into: Vec<StateID> = edges.keys().filter(|(_, to)| *to == r).map(|(from, _)| *from).collect();
            let out_of: Vec<StateID> = edges.keys().filter(|(from, _)| *from == r).map(|(_, to)| *to).collect();
            let mut into = into;
            let mut out_of = out_of;
            into.sort();
            out_of.sort();
            for p in into.iter().filter(|p| **p != r) {
                for q in out_of.iter().filter(|q| **q != r) {
                    let head = edges.get(&(*p, r)).cloned().expect("p->r exists");
                    let tail = edges.get(&(r, *q)).cloned().expect("r->q exists");
                    let through = match &loop_star {
                        Some(star) => Concat(
                            Box::new(head),
                            Box::new(Concat(Box::new(star.clone()), Box::new(tail))),
                        ),
                        None => Concat(Box::new(head), Box::new(tail)),
                    };
                    let merged = match edges.get(&(*p, *q)) {
                        Some(direct) => Union(Box::new(direct.clone()), Box::new(through)),
                        None => through,
                    };
                    edges.insert((*p, *q), merged);
                }
            }
            edges.retain(|(from, to), _| from != &r && to != &r);
        }

        edges.get(&(new_start, new_final)).cloned().unwrap_or(Empty)
    }

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

    fn forget_state(&mut self, state_id: StateID) {
        self.final_states.remove(&state_id);
        if self.initial_state_id == Some(state_id) {
            self.initial_state_id = None;
        }
        self.refresh_determinism();
    }

    fn add_transition(&mut self, state_id1: StateID, state_id2: StateID, input: Input) {
        if !self.states_by_id.contains_key(&state_id2) {
            return;
        }
        let Some(state) = self.states_by_id.get_mut(&state_id1) else {
            return;
        };
        if is_epsilon_label(&input) {
            state.add_transition(state_id2, input);
            self.deterministic = false;
        } else {
            self.string_transitions.replace(input.clone());
            self.deterministic = state.add_transition(state_id2, input) && self.deterministic;
        }
    }

    fn modify_input(&mut self, state_id: StateID, state_transition_id: StateID, old_input: &str, new_input: Input) {
        if let Some(state) = self.states_by_id.get_mut(&state_id) {
            state.modify_input(state_transition_id, old_input, new_input);
        }
        self.refresh_determinism();
    }

    fn remove_transition(&mut self, state_id: StateID, state_transition_id: StateID, input: &str) {
        if let Some(state) = self.states_by_id.get_mut(&state_id) {
            state.remove_transition(state_transition_id, input);
        }
        self.refresh_determinism();
    }

    fn make_initial(&mut self, state_id: StateID) {
        match self.states_by_id.get(&state_id) {
            Some(_) => (),
            None => return,
        }
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

    fn make_final(&mut self, state_id: StateID) {
        if let Some(state) = self.states_by_id.get_mut(&state_id) {
            state.final_flag = true;
            self.final_states.insert(state_id);
        }
    }
}

impl Machine for FiniteAutomata {
    fn kind(&self) -> MachineKind {
        MachineKind::Finite
    }

    fn accepts(&self, input: &str) -> bool {
        self.check_input(&mut input.to_string())
    }

    fn validate(&self) -> Result<(), String> {
        if self.initial_state_id.is_none() {
            return Err("The automaton has no initial state.".to_string());
        }
        let states = self.get_states_by_id_ref();
        for (id, state) in states {
            for (target, _) in state.iter_by_transition() {
                if !states.contains_key(target) {
                    return Err(format!(
                        "State {} has a transition to nonexistent state {}.",
                        id, target
                    ));
                }
            }
        }
        Ok(())
    }
}

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

pub fn hopcroft_algorithm(automata: &FiniteAutomata) -> HashSet<BTreeSet<StateID>> {
    const SINK: StateID = u64::MAX;

    let rejecting_states: BTreeSet<StateID> = automata.get_final_states().iter().cloned().collect();
    let mut non_rejecting_states = hashmap_set_difference(automata.get_states_by_id_ref(),
                                                    automata.get_final_states());
    let mut partition_p: HashSet<BTreeSet<StateID>> = HashSet::new();
    if !rejecting_states.is_empty() {
        partition_p.insert(rejecting_states);
    }
    non_rejecting_states.insert(SINK);
    partition_p.insert(non_rejecting_states);

    let mut partition_w: Vec<BTreeSet<StateID>> = partition_p.iter().cloned().collect();

    while let Some(set_a) = partition_w.pop() {
        for string in automata.get_string_transitions() {
            let set_x = transition_function_set(automata, &set_a, string);
            if set_x.is_empty() {
                continue;
            }

            let mut splits: Vec<(BTreeSet<StateID>, BTreeSet<StateID>, BTreeSet<StateID>)> = Vec::new();
            for set_y in &partition_p {
                let x_y_intersection: BTreeSet<StateID> = set_y.intersection(&set_x).cloned().collect();
                let y_minus_x: BTreeSet<StateID> = set_y.difference(&set_x).cloned().collect();

                if !x_y_intersection.is_empty() && !y_minus_x.is_empty() {
                    splits.push((set_y.clone(), x_y_intersection, y_minus_x));
                }
            }

            for (set_y, x_y_intersection, y_minus_x) in splits {
                partition_p.remove(&set_y);
                partition_p.insert(x_y_intersection.clone());
                partition_p.insert(y_minus_x.clone());
                if let Some(position) = partition_w.iter().position(|set| *set == set_y) {
                    partition_w.swap_remove(position);
                    partition_w.push(x_y_intersection);
                    partition_w.push(y_minus_x);
                } else if x_y_intersection.len() <= y_minus_x.len() {
                    partition_w.push(x_y_intersection);
                } else {
                    partition_w.push(y_minus_x);
                }
            }
        }
    }

    let mut result_partition: HashSet<BTreeSet<StateID>> = HashSet::new();
    for set in partition_p {
        let real_states: BTreeSet<StateID> = set.into_iter().filter(|&id| id != SINK).collect();
        if !real_states.is_empty() {
            result_partition.insert(real_states);
        }
    }
    result_partition
}

fn hashmap_set_difference(map: &HashMap<StateID, State>, set: &HashSet<StateID>) -> BTreeSet<StateID> {
    let mut difference_set = BTreeSet::new();
    for (map_id, _) in map.iter() {
        if !set.contains(map_id) {
            difference_set.insert(*map_id);
        }
    }
    difference_set
}

fn transition_function_set(automata: &FiniteAutomata, set: &BTreeSet<StateID>, string: &str) -> BTreeSet<StateID> {
    const SINK: StateID = u64::MAX;
    let mut new_set = BTreeSet::new();
    for (id,_) in automata.get_states_by_id_ref() {
        let target = match automata.transition_function(*id, string) {
            Some(target) => target,
            None => SINK,
        };
        if set.contains(&target) {
            new_set.insert(*id);
        }
    }
    if set.contains(&SINK) {
        new_set.insert(SINK);
    }
    new_set
}

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

pub fn subset_construction(automata: &FiniteAutomata) -> HashMap<BTreeSet<StateID>, Vec<(BTreeSet<StateID>, &str)>> {
    let mut sets_to_visit: Vec<BTreeSet<StateID>> = Vec::new();
    let mut visited_sets: HashSet<BTreeSet<StateID>> = HashSet::new();
    let mut transitions_by_subsets: HashMap<BTreeSet<StateID>, Vec<(BTreeSet<StateID>,&str)>> = HashMap::new();
    let initial_id = match automata.initial_state_id {
        Some(id) => id,
        None => panic!("There is not an initial state.")
    };
    let mut current_subset = automata.lambda_closure(initial_id, "");
    current_subset.insert(initial_id);
    sets_to_visit.push(current_subset.clone());
    visited_sets.insert(current_subset.clone());
    transitions_by_subsets.insert(current_subset, Vec::new());
    while !sets_to_visit.is_empty() {
        let mut vector_transitions: Vec<(BTreeSet<u64>, &str)> = Vec::new();
        let current_subset = match sets_to_visit.pop() {
            Some(set) => {
                set
            },
            None => panic!("There is no subset, this should never occur"),
        };

        for string in automata.get_string_transitions() {
            let new_subset = lambda_closure_subset(&automata, &current_subset, string);
            if new_subset.is_empty() || visited_sets.contains(&new_subset) {
                vector_transitions.push((new_subset, string));
                continue;
            }
            sets_to_visit.push(new_subset.clone());
            transitions_by_subsets.insert(new_subset.clone(), Vec::new());
            visited_sets.insert(new_subset.clone());
            vector_transitions.push((new_subset, string));
        }
        if let Some(vector) = transitions_by_subsets.get_mut(&current_subset) {
            *vector = vector_transitions;
        }
    }
    transitions_by_subsets
}

fn lambda_closure_subset(automata: &FiniteAutomata, subset: &BTreeSet<StateID>, input_string: &str) -> BTreeSet<StateID> {
    let mut subset_result: BTreeSet<StateID> = BTreeSet::new();
    for id in subset {
        subset_result = subset_result.union(&automata.lambda_closure(*id, input_string)).cloned().collect();
    }
    subset_result
}

fn is_epsilon_label(label: &str) -> bool {
    label.is_empty() || label == "ε"
}
