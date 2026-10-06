use std::collections::{BTreeMap, HashMap, HashSet};
use crate::state::{StateID, Input, State};
use crate::state_machine::{Machine, MachineKind, StateMachine};

pub const DEFAULT_MAX_STEPS: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Stay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionOp {
    pub read: char,
    pub write: char,
    pub direction: Direction,
}

pub fn parse_transition(label: &str) -> Option<Vec<TransitionOp>> {
    label.split(',').map(parse_segment).collect()
}

fn parse_segment(segment: &str) -> Option<TransitionOp> {
    let mut fields = segment.split(';');
    let read = single_char(fields.next()?)?;
    let rest = fields.next()?;
    if fields.next().is_some() {
        return None;
    }
    let mut halves = rest.split('/');
    let write = single_char(halves.next()?)?;
    let direction = match halves.next()?.to_uppercase().as_str() {
        "L" => Direction::Left,
        "R" => Direction::Right,
        "S" => Direction::Stay,
        _ => return None,
    };
    if halves.next().is_some() {
        return None;
    }
    Some(TransitionOp { read, write, direction })
}

fn single_char(field: &str) -> Option<char> {
    let mut chars = field.chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    Some(c)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tape {
    cells: BTreeMap<i64, char>,
    head: i64,
    blank: char,
}

impl Tape {
    pub fn new_with_input(input: &str, blank: char) -> Self {
        let mut cells = BTreeMap::new();
        for (index, c) in input.chars().enumerate() {
            cells.insert(index as i64, c);
        }
        Tape { cells, head: 0, blank }
    }

    pub fn read(&self) -> char {
        self.cells.get(&self.head).copied().unwrap_or(self.blank)
    }

    pub fn get_blank(&self) -> char {
        self.blank
    }

    pub fn write(&mut self, c: char) {
        if c == self.blank {
            self.cells.remove(&self.head);
        } else if self.cells.get(&self.head) != Some(&c) {
            self.cells.insert(self.head, c);
        }
    }

    pub fn move_head(&mut self, direction: Direction) {
        match direction {
            Direction::Left => self.head -= 1,
            Direction::Right => self.head += 1,
            Direction::Stay => (),
        }
    }

    pub fn head_position(&self) -> i64 {
        self.head
    }

    pub fn snapshot(&self, context: i64) -> (String, usize) {
        let from = self.head - context;
        let to = self.head + context;
        let mut rendered = String::new();
        let mut head_offset = 0;
        for position in from..=to {
            if position == self.head {
                head_offset = rendered.chars().count();
            }
            rendered.push(self.cells.get(&position).copied().unwrap_or(self.blank));
        }
        (rendered, head_offset)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Configuration {
    state_id: StateID,
    tapes: Vec<Tape>,
}

impl Configuration {
    pub fn state_id(&self) -> StateID {
        self.state_id
    }

    pub fn tapes(&self) -> &[Tape] {
        &self.tapes
    }

    pub fn tape(&self) -> &Tape {
        &self.tapes[0]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunOutcome {
    Accepted,
    Rejected,
    MaxStepsExceeded,
}

#[derive(Debug, Clone)]
pub struct TuringMachine {
    states_by_id: HashMap<StateID, State>,
    initial_state_id: Option<StateID>,
    final_states: HashSet<StateID>,
    blank_symbol: char,
    deterministic: bool,
    tape_count: Option<usize>,
    parsed: HashMap<StateID, Vec<(StateID, Vec<TransitionOp>)>>,
}

impl TuringMachine {
    pub fn new(blank_symbol: char) -> Self {
        TuringMachine {
            states_by_id: HashMap::new(),
            initial_state_id: None,
            final_states: HashSet::new(),
            blank_symbol,
            deterministic: true,
            tape_count: None,
            parsed: HashMap::new(),
        }
    }

    pub fn get_blank_symbol(&self) -> char {
        self.blank_symbol
    }

    pub fn get_tape_count(&self) -> usize {
        self.tape_count.unwrap_or(1)
    }

    fn rebuild_parsed_state(&mut self, state_id: StateID) {
        match self.states_by_id.get(&state_id) {
            Some(state) => {
                let mut list: Vec<(StateID, Vec<TransitionOp>)> = Vec::new();
                for (target, labels) in state.iter_by_transition() {
                    for label in labels {
                        if let Some(ops) = parse_transition(label) {
                            list.push((*target, ops));
                        }
                    }
                }
                self.parsed.insert(state_id, list);
            },
            None => {
                self.parsed.remove(&state_id);
            },
        }
        self.refresh_determinism();
    }

    fn refresh_determinism(&mut self) {
        self.deterministic = self.parsed.values().all(|transitions| {
            transitions.iter().enumerate().all(|(index, (target, ops))| {
                transitions[index + 1..].iter().all(|(other_target, other_ops)| {
                    let same_reads = ops.len() == other_ops.len()
                        && ops.iter().zip(other_ops.iter()).all(|(a, b)| a.read == b.read);
                    let identical = target == other_target && ops == other_ops;
                    !same_reads || identical
                })
            })
        });
        if self.parsed.values().all(Vec::is_empty) {
            self.tape_count = None;
        }
    }

    fn transitions_of(&self, state_id: &StateID) -> &[(StateID, Vec<TransitionOp>)] {
        self.parsed
            .get(state_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn initial_configuration(&self, input: &str) -> Option<Configuration> {
        let initial_id = self.initial_state_id?;
        let count = self.get_tape_count();
        let mut tapes = Vec::with_capacity(count);
        tapes.push(Tape::new_with_input(input, self.blank_symbol));
        for _ in 1..count {
            tapes.push(Tape::new_with_input("", self.blank_symbol));
        }
        Some(Configuration {
            state_id: initial_id,
            tapes,
        })
    }

    pub fn step(&self, config: &Configuration) -> Option<Configuration> {
        let mut next_configurations = self.step_all(config);
        if next_configurations.len() == 1 {
            return next_configurations.pop();
        }
        None
    }

    pub fn step_all(&self, config: &Configuration) -> Vec<Configuration> {
        self.transitions_of(&config.state_id)
            .iter()
            .filter(|(_, ops)| {
                ops.len() == config.tapes.len()
                    && ops.iter().zip(config.tapes.iter()).all(|(op, tape)| {
                        op.read == tape.read()
                    })
            })
            .map(|(target, ops)| {
                let mut tapes = config.tapes.clone();
                for (tape, op) in tapes.iter_mut().zip(ops.iter()) {
                    tape.write(op.write);
                    tape.move_head(op.direction);
                }
                Configuration {
                    state_id: *target,
                    tapes,
                }
            })
            .collect()
    }

    pub fn run(&self, input: &str, max_steps: usize) -> Option<RunOutcome> {
        let mut config = self.initial_configuration(input)?;
        for _ in 0..max_steps {
            if self.final_states.contains(&config.state_id) {
                return Some(RunOutcome::Accepted);
            }
            match self.step(&config) {
                Some(next_config) => config = next_config,
                None => return Some(RunOutcome::Rejected),
            }
        }
        if self.final_states.contains(&config.state_id) {
            return Some(RunOutcome::Accepted);
        }
        Some(RunOutcome::MaxStepsExceeded)
    }

    pub fn run_nondeterministic(&self, input: &str, max_steps: usize) -> Option<RunOutcome> {
        let initial_config = self.initial_configuration(input)?;
        let mut frontier = vec![initial_config.clone()];
        let mut visited: HashSet<Configuration> = HashSet::from([initial_config]);

        for _ in 0..max_steps {
            let mut next_frontier: Vec<Configuration> = Vec::new();
            for config in frontier.drain(..) {
                if self.final_states.contains(&config.state_id) {
                    return Some(RunOutcome::Accepted);
                }
                for next_config in self.step_all(&config) {
                    if visited.insert(next_config.clone()) {
                        next_frontier.push(next_config);
                    }
                }
            }
            if next_frontier.is_empty() {
                return Some(RunOutcome::Rejected);
            }
            frontier = next_frontier;
        }
        if frontier.iter().any(|config| self.final_states.contains(&config.state_id)) {
            return Some(RunOutcome::Accepted);
        }
        Some(RunOutcome::MaxStepsExceeded)
    }
}

impl StateMachine for TuringMachine {
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
        if self.tape_count.is_none() {
            if let Some(ops) = parse_transition(&input) {
                self.tape_count = Some(ops.len());
            }
        }
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

impl Machine for TuringMachine {
    fn kind(&self) -> MachineKind {
        MachineKind::Turing
    }

    fn accepts(&self, input: &str) -> bool {
        let outcome = if self.deterministic {
            self.run(input, DEFAULT_MAX_STEPS)
        } else {
            self.run_nondeterministic(input, DEFAULT_MAX_STEPS)
        };
        matches!(outcome, Some(RunOutcome::Accepted))
    }

    fn validate(&self) -> Result<(), String> {
        if self.initial_state_id.is_none() {
            return Err("The Turing machine has no initial state.".to_string());
        }
        let states = self.get_states_by_id_ref();
        for (id, state) in states {
            for (target, labels) in state.iter_by_transition() {
                if !states.contains_key(target) {
                    return Err(format!(
                        "State {id} has a transition to nonexistent state {target}."
                    ));
                }
                for label in labels {
                    match parse_transition(label) {
                        None => {
                            return Err(format!(
                                "State {id} has malformed transition label {label:?} (expected \"read;write/dir\" per tape, joined by commas)."
                            ));
                        },
                        Some(ops) => {
                            if let Some(expected) = self.tape_count {
                                if ops.len() != expected {
                                    return Err(format!(
                                        "Label {:?} has {} tape segment(s), but the machine has {} tape(s).",
                                        label, ops.len(), expected
                                    ));
                                }
                            }
                        },
                    }
                }
            }
        }
        Ok(())
    }
}
