use std::collections::{HashMap, HashSet, VecDeque};
use crate::state::{Input, State, StateID};
use crate::state_machine::{Machine, MachineKind, StateMachine};

#[derive(Debug, Clone)]
pub struct PushdownAutomata {
    states_by_id: HashMap<StateID, State>,
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

impl PdaParsedOp {
    fn parse(label: &str) -> Result<Self, &'static str> {
        const SHAPE: &str = "expected \"input;pop/push\"";
        if label == "ε" {
            return Ok(PdaParsedOp {
                is_bare_epsilon: true,
                read: String::new(),
                pops: false,
                pop: String::new(),
                push_entries: Vec::new(),
            });
        }
        let (read, stack_part) = label.split_once(';').ok_or(SHAPE)?;
        let (pop, push) = stack_part.split_once('/').ok_or(SHAPE)?;
        if stack_part.contains(';') || push.contains('/') {
            return Err(SHAPE);
        }
        let push_entries: Vec<String> = if is_epsilon(push) {
            Vec::new()
        } else if push.contains(',') {
            if push.split(',').any(str::is_empty) {
                return Err("empty push segment");
            }
            push.split(',')
                .rev()
                .filter(|part| *part != "ε")
                .map(str::to_string)
                .collect()
        } else {
            push.chars().rev().map(|symbol| symbol.to_string()).collect()
        };
        Ok(PdaParsedOp {
            is_bare_epsilon: false,
            read: read.to_string(),
            pops: !is_epsilon(pop),
            pop: pop.to_string(),
            push_entries,
        })
    }
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
    pub const MAX_VISITED_CONFIGURATIONS: usize = 150_000;
    pub const STACK_DEPTH_PER_CHAR: usize = 2;
    pub const STACK_DEPTH_BASE: usize = 16;

    pub fn new(initial_stack_symbol: String) -> Self {
        PushdownAutomata {
            states_by_id: HashMap::new(),
            initial_state_id: None,
            final_states: HashSet::new(),
            initial_stack_symbol,
            deterministic: true,
            parsed: HashMap::new(),
        }
    }

    fn rebuild_parsed_state(&mut self, state_id: StateID) {
        match self.states_by_id.get(&state_id) {
            Some(state) => {
                let list = state
                    .iter_by_transition()
                    .flat_map(|(target, labels)| {
                        labels
                            .iter()
                            .filter_map(|label| PdaParsedOp::parse(label).ok())
                            .map(|op| (*target, op))
                    })
                    .collect();
                self.parsed.insert(state_id, list);
            },
            None => {
                self.parsed.remove(&state_id);
            },
        }
        self.refresh_determinism();
    }

    fn refresh_determinism(&mut self) {
        self.deterministic = self.parsed.values().all(|ops| {
            ops.iter().enumerate().all(|(index, first)| {
                ops[index + 1..].iter().all(|second| !ops_conflict(first, second))
            })
        });
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

    pub fn check_input(&self, input: &str) -> bool {
        let stack_depth = Self::STACK_DEPTH_BASE
            + Self::STACK_DEPTH_PER_CHAR * input.chars().count();
        self.check_input_with_limit(input, Self::MAX_VISITED_CONFIGURATIONS, stack_depth)
    }

    pub fn check_input_with_limit(
        &self,
        input: &str,
        max_visited: usize,
        max_stack_depth: usize,
    ) -> bool {
        let Some(initial) = self.initial_configuration(input) else {
            return false;
        };
        let mut visited: HashSet<PdaConfiguration> = HashSet::new();
        let mut worklist = VecDeque::from([initial]);
        while let Some(current) = worklist.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            if visited.len() >= max_visited {
                return false;
            }
            if self.is_accepting(&current) {
                return true;
            }
            worklist.extend(self.successors(&current, max_stack_depth));
            visited.insert(current);
        }
        false
    }

    fn successors(&self, config: &PdaConfiguration, max_stack_depth: usize) -> Vec<PdaConfiguration> {
        let mut successors: Vec<PdaConfiguration> = Vec::new();
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
                    remaining_input: config.remaining_input.clone(),
                    stack: current_stack.clone(),
                });
                continue;
            }
            if op.pops && current_stack.last() != Some(&op.pop) {
                continue;
            }
            let next_stack_len = current_stack.len() - usize::from(op.pops) + op.push_entries.len();
            if next_stack_len > max_stack_depth {
                continue;
            }
            let consumed = if is_epsilon(&op.read) {
                0
            } else if config.remaining_input.starts_with(&op.read) {
                op.read.len()
            } else {
                continue;
            };
            let mut stack = current_stack.clone();
            if op.pops {
                stack.pop();
            }
            stack.extend(op.push_entries.iter().cloned());
            successors.push(PdaConfiguration {
                state_id: *id,
                remaining_input: config.remaining_input[consumed..].to_string(),
                stack,
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

    fn forget_state(&mut self, state_id: StateID) {
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
        self.check_input(input)
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
                    if let Err(problem) = PdaParsedOp::parse(label) {
                        return Err(format!(
                            "State {} has malformed transition label {:?} ({}).",
                            id, label, problem
                        ));
                    }
                }
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
