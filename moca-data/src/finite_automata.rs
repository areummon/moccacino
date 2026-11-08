use std::collections::{HashMap, HashSet, BTreeSet};
use crate::state::{is_epsilon, StateID, Input, EPSILON};
use crate::state_machine::{Changed, Graph, Machine, StateMachine};
use crate::regex::ast::RegexAst;

#[derive(Debug, Default, Clone)]
pub struct FiniteAutomata {
    graph: Graph,
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
        Self::default()
    }

    fn moves<'a>(&'a self, state_id: StateID, rest: &'a str) -> impl Iterator<Item = (StateID, usize)> + 'a {
        self.graph
            .state(state_id)
            .into_iter()
            .flat_map(|state| state.labels())
            .filter_map(move |(target, label)| {
                if is_epsilon(label) {
                    Some((target, 0))
                } else if rest.starts_with(label) {
                    Some((target, label.len()))
                } else {
                    None
                }
            })
    }

    pub fn check_input(&self, input: &str) -> bool {
        let Some(initial_id) = self.graph.initial() else {
            return false;
        };
        let mut visited: HashSet<(StateID, usize)> = HashSet::new();
        let mut pending = vec![(initial_id, 0)];
        while let Some((state_id, offset)) = pending.pop() {
            if !visited.insert((state_id, offset)) {
                continue;
            }
            let rest = &input[offset..];
            if self.graph.accepts_at(state_id, rest) {
                return true;
            }
            pending.extend(self.moves(state_id, rest).map(|(target, consumed)| (target, offset + consumed)));
        }
        false
    }

    pub fn initial_configuration(&self, input: &str) -> Option<FiniteConfiguration> {
        Some(FiniteConfiguration {
            state_id: self.graph.initial()?,
            remaining_input: input.to_string(),
        })
    }

    pub fn is_accepting(&self, config: &FiniteConfiguration) -> bool {
        self.graph.accepts_at(config.state_id, &config.remaining_input)
    }

    pub fn step_all(&self, config: &FiniteConfiguration) -> Vec<FiniteConfiguration> {
        let mut successors: Vec<FiniteConfiguration> = self
            .moves(config.state_id, &config.remaining_input)
            .map(|(target, consumed)| FiniteConfiguration {
                state_id: target,
                remaining_input: config.remaining_input[consumed..].to_string(),
            })
            .collect();
        successors.sort_by(|a, b| {
            (a.state_id, a.remaining_input.as_str())
                .cmp(&(b.state_id, b.remaining_input.as_str()))
        });
        successors
    }

    fn alphabet(&self) -> BTreeSet<&str> {
        self.graph
            .labels()
            .map(|(_, _, label)| label)
            .filter(|label| !is_epsilon(label))
            .collect()
    }

    fn reachable_from(&self, start: StateID) -> HashSet<StateID> {
        let mut reachable = HashSet::from([start]);
        let mut pending = vec![start];
        while let Some(id) = pending.pop() {
            if let Some(state) = self.graph.state(id) {
                for (target, labels) in state.iter_by_transition() {
                    if !labels.is_empty() && reachable.insert(*target) {
                        pending.push(*target);
                    }
                }
            }
        }
        reachable
    }

    fn epsilon_closure(&self, seeds: impl IntoIterator<Item = StateID>) -> BTreeSet<StateID> {
        let mut closure: BTreeSet<StateID> = BTreeSet::new();
        let mut pending: Vec<StateID> = seeds.into_iter().collect();
        while let Some(id) = pending.pop() {
            if !closure.insert(id) {
                continue;
            }
            if let Some(state) = self.graph.state(id) {
                pending.extend(state.labels().filter(|(_, label)| is_epsilon(label)).map(|(target, _)| target));
            }
        }
        closure
    }

    fn step(&self, subset: &BTreeSet<StateID>, symbol: &str) -> BTreeSet<StateID> {
        self.epsilon_closure(
            subset
                .iter()
                .filter_map(|id| self.graph.state(*id))
                .flat_map(|state| state.labels())
                .filter(|(_, label)| *label == symbol)
                .map(|(target, _)| target),
        )
    }

    pub fn to_dfa(&self) -> FiniteAutomata {
        let mut dfa = FiniteAutomata::new();
        let Some(initial_id) = self.graph.initial() else {
            return dfa;
        };
        let alphabet = self.alphabet();
        let initial_subset = self.epsilon_closure([initial_id]);
        let mut id_of: HashMap<BTreeSet<StateID>, StateID> = HashMap::from([(initial_subset.clone(), dfa.add_state())]);
        let mut pending = std::collections::VecDeque::from([initial_subset]);
        while let Some(subset) = pending.pop_front() {
            let from = id_of[&subset];
            if subset.iter().any(|member| self.graph.is_final(*member)) {
                dfa.make_final(from);
            }
            for symbol in &alphabet {
                let target = self.step(&subset, symbol);
                if target.is_empty() {
                    continue;
                }
                let to = match id_of.get(&target) {
                    Some(&to) => to,
                    None => {
                        let to = dfa.add_state();
                        id_of.insert(target.clone(), to);
                        pending.push_back(target);
                        to
                    },
                };
                dfa.add_transition(from, to, symbol.to_string());
            }
        }
        dfa.make_initial(0);
        dfa
    }

    pub fn minimize(&self) -> Self {
        if !self.is_deterministic() {
            return self.to_dfa().minimize();
        }
        let mut minimized_automata = self.clone();
        if let Some(initial_id) = self.graph.initial() {
            let reachable = self.reachable_from(initial_id);
            for id in self.get_states_by_id_ref().keys().filter(|id| !reachable.contains(id)) {
                minimized_automata.remove_state(*id);
            }
        }
        convert_minimized_dfa(&minimized_automata, hopcroft_algorithm(&minimized_automata))
    }

    pub fn to_regex(&self) -> RegexAst {
        use RegexAst::*;

        let mut edges: HashMap<(StateID, StateID), RegexAst> = HashMap::new();
        let mut union_edge = |from: StateID, to: StateID, label: &str| {
            let piece = if is_epsilon(label) {
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

        let Some(initial_id) = self.graph.initial() else {
            return Empty;
        };

        let reachable = self.reachable_from(initial_id);

        let next_id = self.get_states_by_id_ref().keys().copied().max().map_or(0, |max| max + 1);
        let new_start = next_id;
        let new_final = next_id + 1;

        union_edge(new_start, initial_id, EPSILON);
        for id in &reachable {
            if self.graph.is_final(*id) {
                union_edge(*id, new_final, EPSILON);
            }
        }
        for id in &reachable {
            if let Some(state) = self.graph.state(*id) {
                for (target, label) in state.labels() {
                    if reachable.contains(&target) {
                        union_edge(*id, target, label);
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

    fn transition_function(&self, state_id: StateID, symbol: &str) -> Option<StateID> {
        self.graph
            .state(state_id)?
            .labels()
            .find(|(_, label)| *label == symbol)
            .map(|(target, _)| target)
    }
}

impl StateMachine for FiniteAutomata {
    fn graph(&self) -> &Graph {
        &self.graph
    }

    fn graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }

    fn is_deterministic(&self) -> bool {
        self.get_states_by_id_ref().values().all(|state| {
            let mut target_of: HashMap<&str, StateID> = HashMap::new();
            state.labels().all(|(target, label)| {
                !is_epsilon(label) && target_of.insert(label, target).is_none()
            })
        })
    }

    fn transitions_changed(&mut self, _changed: Changed) {}
}

impl Machine for FiniteAutomata {
    fn accepts(&self, input: &str) -> bool {
        self.check_input(input)
    }

    fn validate(&self) -> Result<(), String> {
        self.graph.validate_labels("automaton", |label| self.check_label(label))
    }
}

const SINK: StateID = u64::MAX;

fn hopcroft_algorithm(automata: &FiniteAutomata) -> HashSet<BTreeSet<StateID>> {
    let alphabet = automata.alphabet();
    let state_ids = automata.get_states_by_id_ref().keys();
    let mut sources_into: HashMap<(&str, StateID), Vec<StateID>> = HashMap::new();
    for symbol in &alphabet {
        for &id in state_ids.clone() {
            let target = automata.transition_function(id, symbol).unwrap_or(SINK);
            sources_into.entry((symbol, target)).or_default().push(id);
        }
        sources_into.entry((symbol, SINK)).or_default().push(SINK);
    }

    let finals = automata.get_final_states();
    let accepting: BTreeSet<StateID> = finals.iter().copied().collect();
    let mut rejecting: BTreeSet<StateID> = state_ids.filter(|id| !finals.contains(id)).copied().collect();
    rejecting.insert(SINK);
    let mut partition_p: HashSet<BTreeSet<StateID>> = HashSet::from([rejecting]);
    if !accepting.is_empty() {
        partition_p.insert(accepting);
    }

    let mut partition_w: Vec<BTreeSet<StateID>> = partition_p.iter().cloned().collect();

    while let Some(set_a) = partition_w.pop() {
        for symbol in &alphabet {
            let set_x: BTreeSet<StateID> = set_a
                .iter()
                .filter_map(|target| sources_into.get(&(*symbol, *target)))
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
    let alphabet = automata.alphabet();
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
    let initial = automata.graph.initial();
    let mut order = match initial.and_then(|initial_id| block_of.get(&initial_id)) {
        Some(&start) => breadth_first(start, |block| {
            alphabet.iter().filter_map(|symbol| successor(block, symbol)).collect()
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
        if initial.is_some_and(|initial_id| set.contains(&initial_id)) {
            minimized_automata.make_initial(id);
        }
        if set.iter().any(|state| automata.graph.is_final(*state)) {
            minimized_automata.make_final(id);
        }
        for symbol in &alphabet {
            if let Some(target) = successor(block, symbol) {
                minimized_automata.add_transition(id, id_of_block[target], symbol.to_string());
            }
        }
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
