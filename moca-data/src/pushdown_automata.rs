use std::collections::{HashMap, HashSet, BTreeSet};
use crate::state::{Input, State, StateID};
use crate::state_machine::{Machine, MachineKind, StateMachine};

#[derive(Debug, Clone)]
pub struct PushdownAutomata {
    states_by_id: HashMap<StateID, State>,
    string_transitions: HashMap<(StateID, String), (StateID, String)>,
    initial_state_id: Option<StateID>,
    final_states: HashSet<StateID>,
    initial_stack_symbol: String,
    deterministic: bool,
    parsed: HashMap<StateID, Vec<(StateID, PdaParsedOp)>>,
}

#[derive(Debug, Clone, PartialEq)]
struct PdaParsedOp {
    is_bare_epsilon: bool,
    read: String,
    pops: bool,
    pop: String,
    push_entries: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PdaConfiguration {
    state_id: StateID,
    remaining_input: Input,
    stack: Vec<String>,
}

impl PdaConfiguration {
    pub fn state_id(&self) -> StateID {
        self.state_id
    }

    pub fn remaining_input(&self) -> &Input {
        &self.remaining_input
    }

    pub fn stack(&self) -> &[String] {
        &self.stack
    }
}

impl PushdownAutomata {
    pub fn new(initial_stack_symbol: String) -> Self {
        PushdownAutomata {
            states_by_id: HashMap::new(),
            string_transitions: HashMap::new(),
            initial_state_id: None,
            final_states: HashSet::new(),
            initial_stack_symbol,
            deterministic: true,
            parsed: HashMap::new(),
        }
    }

    fn rebuild_parsed_state(&mut self, state_id: StateID) {
        self.string_transitions.retain(|(from, _), _| *from != state_id);
        let Some(state) = self.states_by_id.get(&state_id) else {
            self.parsed.remove(&state_id);
            self.refresh_determinism();
            return;
        };
        for (target, labels) in state.iter_by_transition() {
            for label in labels {
                let spelled = if label == "ε" { "ε;ε/ε" } else { label.as_str() };
                if let Some((read, rest)) = spelled.split_once(';') {
                    if !rest.contains(';') {
                        self.string_transitions
                            .insert((state_id, read.to_string()), (*target, rest.to_string()));
                    }
                }
            }
        }
        let mut list: Vec<(StateID, PdaParsedOp)> = Vec::new();
        for (target, labels) in state.iter_by_transition() {
            for string in labels {
                if string == "ε" {
                    list.push((
                        *target,
                        PdaParsedOp {
                            is_bare_epsilon: true,
                            read: String::new(),
                            pops: false,
                            pop: String::new(),
                            push_entries: Vec::new(),
                        },
                    ));
                    continue;
                }
                let string_transitions: Vec<&str> = string.split(';').collect();
                if string_transitions.len() != 2 {
                    continue;
                }
                let stack_transition: Vec<&str> = string_transitions[1].split('/').collect();
                if stack_transition.len() != 2 {
                    continue;
                }
                let push_symbols = stack_transition[1];
                if push_symbols.contains(',')
                    && push_symbols.split(',').any(|part| part.is_empty())
                {
                    continue;
                }
                let push_entries: Vec<String> = if push_symbols == "ε" || push_symbols.is_empty()
                {
                    Vec::new()
                } else if push_symbols.contains(',') {
                    push_symbols
                        .split(',')
                        .rev()
                        .filter(|part| !part.is_empty() && *part != "ε")
                        .map(|part| part.to_string())
                        .collect()
                } else {
                    push_symbols
                        .chars()
                        .rev()
                        .map(|symbol| symbol.to_string())
                        .collect()
                };
                list.push((
                    *target,
                    PdaParsedOp {
                        is_bare_epsilon: false,
                        read: string_transitions[0].to_string(),
                        pops: !is_epsilon(stack_transition[0]),
                        pop: stack_transition[0].to_string(),
                        push_entries,
                    },
                ));
            }
        }
        self.parsed.insert(state_id, list);
        self.refresh_determinism();
    }

    fn refresh_determinism(&mut self) {
        self.deterministic = self.parsed.values().all(|ops| {
            ops.iter().enumerate().all(|(index, first)| {
                ops[index + 1..].iter().all(|second| !ops_conflict(first, second))
            })
        });
    }

    pub fn get_string_transitions(&self) -> &HashMap<(StateID, String), (StateID, String)> {
        &self.string_transitions
    }

    pub fn get_initial_stack_symbol(&self) -> &str {
        &self.initial_stack_symbol
    }

    pub fn initial_configuration(&self, input: &str) -> Option<PdaConfiguration> {
        let initial_id = self.initial_state_id?;
        Some(PdaConfiguration {
            state_id: initial_id,
            remaining_input: input.to_string(),
            stack: vec![self.initial_stack_symbol.to_string()],
        })
    }

    pub fn is_accepting(&self, config: &PdaConfiguration) -> bool {
        self.final_states.contains(&config.state_id) && config.remaining_input.is_empty()
    }

    pub fn step_all(&self, config: &PdaConfiguration) -> Vec<PdaConfiguration> {
        self.successors(config, usize::MAX)
    }

    pub fn check_input(&self, input: &mut Input) -> bool {
        let stack_depth = Self::STACK_DEPTH_BASE
            + Self::STACK_DEPTH_PER_CHAR * input.chars().count();
        self.check_input_with_limit(input, Self::MAX_VISITED_CONFIGURATIONS, stack_depth)
    }

    pub fn check_input_with_limit(
        &self,
        input: &mut Input,
        max_visited: usize,
        max_stack_depth: usize,
    ) -> bool {
        match self.initial_configuration(input) {
            Some(initial_config) => {
                let mut visited = HashSet::new();
                self.traverse(initial_config, &mut visited, max_visited, max_stack_depth)
            },
            None => false,
        }
    }

    pub fn add_label(&mut self, state_id: StateID, label: BTreeSet<StateID>) {
        if let Some(state) = self.states_by_id.get_mut(&state_id) {
            state.label = label;
        }
    }

    pub const MAX_VISITED_CONFIGURATIONS: usize = 150_000;
    pub const STACK_DEPTH_PER_CHAR: usize = 2;
    pub const STACK_DEPTH_BASE: usize = 16;

    fn traverse(
        &self,
        initial: PdaConfiguration,
        visited: &mut HashSet<PdaConfiguration>,
        max_visited: usize,
        max_stack_depth: usize,
    ) -> bool {
        let mut worklist: std::collections::VecDeque<PdaConfiguration> =
            std::collections::VecDeque::from([initial]);

        while let Some(current) = worklist.pop_front() {
            if !visited.insert(current.clone()) {
                continue;
            }
            if visited.len() > max_visited {
                return false;
            }
            if self.is_accepting(&current) {
                return true;
            }
            for successor in self.successors(&current, max_stack_depth) {
                worklist.push_back(successor);
            }
        }
        false
    }

    fn successors(&self, config: &PdaConfiguration, max_stack_depth: usize) -> Vec<PdaConfiguration> {
        let mut successors: Vec<PdaConfiguration> = Vec::new();
        let current_input = &config.remaining_input;
        let current_stack = &config.stack;

        for (id, op) in self
            .parsed
            .get(&config.state_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
        {
            if op.is_bare_epsilon {
                successors.push(PdaConfiguration {
                    state_id: *id,
                    remaining_input: current_input.clone(),
                    stack: current_stack.clone(),
                });
                continue;
            }
            if op.pops && current_stack.last() != Some(&op.pop) {
                continue;
            }
            let mut next_stack_len = current_stack.len();
            if op.pops {
                next_stack_len -= 1;
            }
            next_stack_len += op.push_entries.len();
            if next_stack_len > max_stack_depth {
                continue;
            }
            let apply_stack = |stack: &mut Vec<String>| {
                if op.pops {
                    stack.pop();
                }
                for entry in &op.push_entries {
                    stack.push(entry.clone());
                }
            };
            if is_epsilon(&op.read) {
                let mut branch_stack = current_stack.clone();
                apply_stack(&mut branch_stack);
                successors.push(PdaConfiguration {
                    state_id: *id,
                    remaining_input: current_input.clone(),
                    stack: branch_stack,
                });
                continue;
            }
            if !current_input.starts_with(&op.read) {
                continue;
            }
            let mut rest = current_input.clone();
            rest.replace_range(0..op.read.len(), "");
            let mut branch_stack = current_stack.clone();
            apply_stack(&mut branch_stack);
            successors.push(PdaConfiguration {
                state_id: *id,
                remaining_input: rest,
                stack: branch_stack,
            });
        }
        successors
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

    fn add_transition(&mut self, state_id1: StateID, state_id2: StateID, input: Input) {
        if !self.states_by_id.contains_key(&state_id2) {
            return;
        }
        let Some(state) = self.states_by_id.get_mut(&state_id1) else {
            return;
        };
        state.add_transition(state_id2, input);
        self.rebuild_parsed_state(state_id1);
    }

    fn modify_input(
        &mut self,
        state_id: StateID,
        state_transition_id: StateID,
        old_input: &str,
        new_input: Input,
    ) {
        if let Some(state) = self.states_by_id.get_mut(&state_id) {
            state.modify_input(state_transition_id, old_input, new_input);
        }
        self.rebuild_parsed_state(state_id);
    }

    fn remove_transition(
        &mut self,
        state_id: StateID,
        state_transition_id: StateID,
        input: &str,
    ) {
        if let Some(state) = self.states_by_id.get_mut(&state_id) {
            state.remove_transition(state_transition_id, input);
        }
        self.rebuild_parsed_state(state_id);
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

    fn forget_state(&mut self, state_id: StateID) {
        self.final_states.remove(&state_id);
        if self.initial_state_id == Some(state_id) {
            self.initial_state_id = None;
        }
        self.string_transitions.retain(|(from, _), (to, _)| {
            *from != state_id && *to != state_id
        });
        self.parsed.remove(&state_id);
        for list in self.parsed.values_mut() {
            list.retain(|(target, _)| *target != state_id);
        }
        self.refresh_determinism();
    }
}

impl Machine for PushdownAutomata {
    fn kind(&self) -> MachineKind {
        MachineKind::Pushdown
    }

    fn accepts(&self, input: &str) -> bool {
        self.check_input(&mut input.to_string())
    }

    fn validate(&self) -> Result<(), String> {
        if self.initial_state_id.is_none() {
            return Err("The pushdown automaton has no initial state.".to_string());
        }
        let states = self.get_states_by_id_ref();
        for (id, state) in states {
            for (target, labels) in state.iter_by_transition() {
                if !states.contains_key(target) {
                    return Err(format!(
                        "State {} has a transition to nonexistent state {}.",
                        id, target
                    ));
                }
                for label in labels {
                    if label == "ε" {
                        continue;
                    }
                    let fields: Vec<&str> = label.split(';').collect();
                    if fields.len() != 2 {
                        return Err(format!(
                            "State {} has malformed transition label {:?} (expected \"input;pop/push\").",
                            id, label
                        ));
                    }
                    let halves: Vec<&str> = fields[1].split('/').collect();
                    if halves.len() != 2 {
                        return Err(format!(
                            "State {} has malformed transition label {:?} (expected \"input;pop/push\").",
                            id, label
                        ));
                    }
                    if halves[1].contains(',') && halves[1].split(',').any(|part| part.is_empty()) {
                        return Err(format!(
                            "State {} has malformed transition label {:?} (empty push segment).",
                            id, label
                        ));
                    }
                }
            }
        }
        for ((from, _), (to, _)) in self.string_transitions.iter() {
            if !states.contains_key(from) || !states.contains_key(to) {
                return Err(format!(
                    "The transition table references a nonexistent state ({} -> {}).",
                    from, to
                ));
            }
        }
        Ok(())
    }
}

fn is_epsilon(symbol: &str) -> bool {
    symbol.is_empty() || symbol == "ε"
}

fn ops_conflict(first: &(StateID, PdaParsedOp), second: &(StateID, PdaParsedOp)) -> bool {
    if first == second {
        return false;
    }
    let (_, a) = first;
    let (_, b) = second;
    if a.is_bare_epsilon || b.is_bare_epsilon {
        return true;
    }
    let reads_overlap = a.read == b.read || is_epsilon(&a.read) || is_epsilon(&b.read);
    let pops_overlap = !a.pops || !b.pops || a.pop == b.pop;
    reads_overlap && pops_overlap
}
