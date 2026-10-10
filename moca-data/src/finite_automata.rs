use std::collections::{HashMap, HashSet, BTreeSet};
use crate::state::{StateID, Input, State};
use crate::state_machine::{Machine, MachineKind, StateMachine};
use crate::regex::ast::RegexAst;

#[derive(Debug, Default, Clone)]
pub struct FiniteAutomata {
    states_by_id: HashMap<StateID, State>,
    string_transitions: BTreeSet<String>,
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
            string_transitions: BTreeSet::new(),
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

    pub fn get_string_transitions(&self) -> &BTreeSet<String> {
        &self.string_transitions
    }

    pub fn check_input(&self, input: &str) -> bool {
        let Some(initial_id) = self.initial_state_id else {
            return false;
        };
        let mut visited: HashSet<(StateID, usize)> = HashSet::new();
        let mut pending = vec![(initial_id, 0)];
        while let Some((state_id, offset)) = pending.pop() {
            if !visited.insert((state_id, offset)) {
                continue;
            }
            let Some(state) = self.states_by_id.get(&state_id) else {
                continue;
            };
            let rest = &input[offset..];
            if state.final_flag && rest.is_empty() {
                return true;
            }
            for (next_id, labels) in state.iter_by_transition() {
                for label in labels {
                    if is_epsilon_label(label) {
                        pending.push((*next_id, offset));
                    } else if rest.starts_with(label.as_str()) {
                        pending.push((*next_id, offset + label.len()));
                    }
                }
            }
        }
        false
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
                    let consumed = if is_epsilon_label(label) {
                        0
                    } else if config.remaining_input.starts_with(label.as_str()) {
                        label.len()
                    } else {
                        continue;
                    };
                    successors.push(FiniteConfiguration {
                        state_id: *next_id,
                        remaining_input: config.remaining_input[consumed..].to_string(),
                    });
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
        let mut alphabet: BTreeSet<String> = BTreeSet::new();
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

    fn reachable_from(&self, start: StateID) -> HashSet<StateID> {
        let mut reachable = HashSet::from([start]);
        let mut pending = vec![start];
        while let Some(id) = pending.pop() {
            if let Some(state) = self.states_by_id.get(&id) {
                for (target, labels) in state.iter_by_transition() {
                    if !labels.is_empty() && reachable.insert(*target) {
                        pending.push(*target);
                    }
                }
            }
        }
        reachable
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
        let subsets_and_transitions = subset_construction(self);
        let initial_subset = self.lambda_closure(self.initial_state_id.expect("There is not an initial state."), "");
        let order = breadth_first(&initial_subset, |subset| {
            subsets_and_transitions[subset]
                .iter()
                .map(|(target, _)| target)
                .filter(|target| !target.is_empty())
                .collect()
        });
        let mut states_by_id: HashMap<StateID, State> = HashMap::new();
        let mut id_by_subsets: HashMap<&BTreeSet<StateID>, StateID> = HashMap::new();
        let mut final_states: HashSet<StateID> = HashSet::new();
        for (id, subset) in (0..).zip(order) {
            let mut state = State::new(format!("q{id}"));
            state.initial_flag = id == 0;
            if subset.iter().any(|member| self.states_by_id.get(member).is_some_and(|s| s.final_flag)) {
                state.final_flag = true;
                final_states.insert(id);
            }
            state.label = subset.clone();
            id_by_subsets.insert(subset, id);
            states_by_id.insert(id, state);
        }
        for (subset, transitions) in &subsets_and_transitions {
            let from = id_by_subsets[subset];
            for (set, string) in transitions {
                if let Some(&to) = id_by_subsets.get(set) {
                    if let Some(state) = states_by_id.get_mut(&from) {
                        state.add_transition(to, string.to_string());
                    }
                }
            }
        }
        FiniteAutomata {
            states_by_id,
            string_transitions: self.string_transitions.clone(),
            initial_state_id: Some(0),
            final_states,
            deterministic: true,
        }
    }

    pub fn minimize(&self) -> Self {
        if !self.deterministic {
            panic!("Cannon minimize a nfa");
        }
        let mut minimized_automata = self.clone();
        if let Some(initial_id) = self.initial_state_id {
            let reachable = self.reachable_from(initial_id);
            for id in self.states_by_id.keys().filter(|id| !reachable.contains(id)) {
                minimized_automata.remove_state(*id);
            }
        }
        convert_minimized_dfa(&minimized_automata, hopcroft_algorithm(&minimized_automata))
    }

    pub fn to_regex(&self) -> RegexAst {
        use RegexAst::*;

        let mut edges: HashMap<(StateID, StateID), RegexAst> = HashMap::new();
        let mut union_edge = |from: StateID, to: StateID, label: &str| {
            let piece = if is_epsilon_label(label) {
                Epsilon
            } else {
                label
                    .chars()
                    .map(Char)
                    .reduce(RegexAst::concat)
                    .expect("non-empty label")
            };
            match edges.get_mut(&(from, to)) {
                Some(existing) => {
                    *existing = RegexAst::union(std::mem::replace(existing, Epsilon), piece);
                },
                None => {
                    edges.insert((from, to), piece);
                },
            }
        };

        let Some(initial_id) = self.initial_state_id else {
            return Empty;
        };

        let reachable = self.reachable_from(initial_id);

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
            let loop_star = edges.get(&(r, r)).cloned().map_or(Epsilon, RegexAst::star);
            let mut into: Vec<StateID> = edges.keys().filter(|(_, to)| *to == r).map(|(from, _)| *from).collect();
            let mut out_of: Vec<StateID> = edges.keys().filter(|(from, _)| *from == r).map(|(_, to)| *to).collect();
            into.sort();
            out_of.sort();
            for p in into.iter().filter(|p| **p != r) {
                for q in out_of.iter().filter(|q| **q != r) {
                    let head = edges.get(&(*p, r)).cloned().expect("p->r exists");
                    let tail = edges.get(&(r, *q)).cloned().expect("r->q exists");
                    let through = RegexAst::concat(head, RegexAst::concat(loop_star.clone(), tail));
                    let merged = match edges.remove(&(*p, *q)) {
                        Some(direct) => RegexAst::union(direct, through),
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

    fn markers_mut(&mut self) -> (&mut HashMap<StateID, State>, &mut Option<StateID>, &mut HashSet<StateID>) {
        (&mut self.states_by_id, &mut self.initial_state_id, &mut self.final_states)
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

    fn forget_state(&mut self, _state_id: StateID) {
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
}

impl Machine for FiniteAutomata {
    fn kind(&self) -> MachineKind {
        MachineKind::Finite
    }

    fn accepts(&self, input: &str) -> bool {
        self.check_input(input)
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
                        "State {id} has a transition to nonexistent state {target}."
                    ));
                }
            }
        }
        Ok(())
    }
}

const SINK: StateID = u64::MAX;

fn hopcroft_algorithm(automata: &FiniteAutomata) -> HashSet<BTreeSet<StateID>> {
    let mut sources_into: HashMap<(&str, StateID), Vec<StateID>> = HashMap::new();
    for symbol in &automata.string_transitions {
        for &id in automata.states_by_id.keys() {
            let target = automata.transition_function(id, symbol).unwrap_or(SINK);
            sources_into.entry((symbol, target)).or_default().push(id);
        }
        sources_into.entry((symbol, SINK)).or_default().push(SINK);
    }

    let accepting: BTreeSet<StateID> = automata.final_states.iter().copied().collect();
    let mut rejecting: BTreeSet<StateID> = automata
        .states_by_id
        .keys()
        .filter(|id| !automata.final_states.contains(id))
        .copied()
        .collect();
    rejecting.insert(SINK);
    let mut partition_p: HashSet<BTreeSet<StateID>> = HashSet::from([rejecting]);
    if !accepting.is_empty() {
        partition_p.insert(accepting);
    }

    let mut partition_w: Vec<BTreeSet<StateID>> = partition_p.iter().cloned().collect();

    while let Some(set_a) = partition_w.pop() {
        for symbol in &automata.string_transitions {
            let set_x: BTreeSet<StateID> = set_a
                .iter()
                .filter_map(|target| sources_into.get(&(symbol.as_str(), *target)))
                .flatten()
                .copied()
                .collect();
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

    partition_p
        .into_iter()
        .map(|set| set.into_iter().filter(|&id| id != SINK).collect::<BTreeSet<StateID>>())
        .filter(|set| !set.is_empty())
        .collect()
}

fn convert_minimized_dfa(automata: &FiniteAutomata, partition: HashSet<BTreeSet<StateID>>) -> FiniteAutomata {
    let mut blocks: Vec<BTreeSet<StateID>> = partition.into_iter().collect();
    blocks.sort();
    let block_of: HashMap<StateID, usize> = blocks
        .iter()
        .enumerate()
        .flat_map(|(block, set)| set.iter().map(move |&id| (id, block)))
        .collect();
    let successor = |block: usize, symbol: &str| {
        let representative = *blocks[block].first()?;
        let target = automata.transition_function(representative, symbol)?;
        block_of.get(&target).copied()
    };
    let mut order = match automata.initial_state_id.and_then(|initial_id| block_of.get(&initial_id)) {
        Some(&start) => breadth_first(start, |block| {
            automata.string_transitions.iter().filter_map(|symbol| successor(block, symbol)).collect()
        }),
        None => Vec::new(),
    };
    order.extend((0..blocks.len()).filter(|block| !order.contains(block)).collect::<Vec<_>>());

    let mut minimized_automata = FiniteAutomata::new();
    let mut id_of_block = vec![0; blocks.len()];
    for &block in &order {
        id_of_block[block] = minimized_automata.add_state();
    }
    for &block in &order {
        let id = id_of_block[block];
        let set = &blocks[block];
        if automata.initial_state_id.is_some_and(|initial_id| set.contains(&initial_id)) {
            minimized_automata.make_initial(id);
        }
        if set.iter().any(|state| automata.final_states.contains(state)) {
            minimized_automata.make_final(id);
        }
        for symbol in &automata.string_transitions {
            if let Some(target) = successor(block, symbol) {
                minimized_automata.add_transition(id, id_of_block[target], symbol.clone());
            }
        }
        minimized_automata.add_label(id, set.clone());
    }
    minimized_automata
}

fn breadth_first<K: Copy + Eq + std::hash::Hash>(start: K, successors: impl Fn(K) -> Vec<K>) -> Vec<K> {
    let mut order = vec![start];
    let mut seen = HashSet::from([start]);
    let mut next = 0;
    while let Some(&current) = order.get(next) {
        for successor in successors(current) {
            if seen.insert(successor) {
                order.push(successor);
            }
        }
        next += 1;
    }
    order
}

type SubsetTransitions<'a> = HashMap<BTreeSet<StateID>, Vec<(BTreeSet<StateID>, &'a str)>>;

fn subset_construction(automata: &FiniteAutomata) -> SubsetTransitions<'_> {
    let initial_id = automata.initial_state_id.expect("There is not an initial state.");
    let initial_subset = automata.lambda_closure(initial_id, "");
    let mut transitions_by_subsets: SubsetTransitions = HashMap::from([(initial_subset.clone(), Vec::new())]);
    let mut sets_to_visit = vec![initial_subset];
    while let Some(current_subset) = sets_to_visit.pop() {
        let mut vector_transitions: Vec<(BTreeSet<StateID>, &str)> = Vec::new();
        for string in &automata.string_transitions {
            let new_subset: BTreeSet<StateID> = current_subset
                .iter()
                .flat_map(|id| automata.lambda_closure(*id, string))
                .collect();
            if !new_subset.is_empty() && !transitions_by_subsets.contains_key(&new_subset) {
                sets_to_visit.push(new_subset.clone());
                transitions_by_subsets.insert(new_subset.clone(), Vec::new());
            }
            vector_transitions.push((new_subset, string));
        }
        transitions_by_subsets.insert(current_subset, vector_transitions);
    }
    transitions_by_subsets
}

fn is_epsilon_label(label: &str) -> bool {
    label.is_empty() || label == "ε"
}
