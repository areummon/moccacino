use std::collections::{BTreeMap, HashSet};
use std::fmt;
use crate::state::StateID;
use crate::state_machine::{Changed, Graph, Machine, ParsedLabels, StateMachine};

pub const DEFAULT_MAX_STEPS: usize = 10_000;
pub const DEFAULT_BLANK: char = '_';

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

impl TransitionOp {
    pub fn parse(segment: &str) -> Option<Self> {
        let (read, rest) = segment.split_once(';')?;
        let (write, direction) = rest.split_once('/')?;
        Self::from_fields(read, write, direction).ok()
    }

    pub fn from_fields(read: &str, write: &str, direction: &str) -> Result<Self, &'static str> {
        let direction = match direction.to_uppercase().as_str() {
            "L" => Direction::Left,
            "R" => Direction::Right,
            "S" => Direction::Stay,
            _ => return Err("invalid direction (use L, R or S)"),
        };
        let symbol = |field| single_char(field).ok_or("read/write symbols must be single characters (use the blank symbol for empty cells)");
        Ok(TransitionOp { read: symbol(read)?, write: symbol(write)?, direction })
    }
}

impl Direction {
    pub fn letter(self) -> char {
        match self {
            Direction::Left => 'L',
            Direction::Right => 'R',
            Direction::Stay => 'S',
        }
    }
}

impl fmt::Display for TransitionOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{};{}/{}", self.read, self.write, self.direction.letter())
    }
}

pub fn parse_transition(label: &str) -> Option<Vec<TransitionOp>> {
    label.split(',').map(TransitionOp::parse).collect()
}

pub fn format_transition(ops: &[TransitionOp]) -> String {
    ops.iter().map(TransitionOp::to_string).collect::<Vec<_>>().join(",")
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
    graph: Graph,
    blank_symbol: char,
    tape_count: Option<usize>,
    parsed: ParsedLabels<Vec<TransitionOp>>,
}

impl TuringMachine {
    pub fn new(blank_symbol: char) -> Self {
        TuringMachine {
            graph: Graph::default(),
            blank_symbol,
            tape_count: None,
            parsed: ParsedLabels::default(),
        }
    }

    pub fn get_blank_symbol(&self) -> char {
        self.blank_symbol
    }

    pub fn initial_configuration(&self, input: &str) -> Option<Configuration> {
        let initial_id = self.graph.initial()?;
        let mut tapes = vec![Tape::new_with_input(input, self.blank_symbol)];
        for _ in 1..self.tape_count.unwrap_or(1) {
            tapes.push(Tape::new_with_input("", self.blank_symbol));
        }
        Some(Configuration {
            state_id: initial_id,
            tapes,
        })
    }

    /// The single successor of `config`. `None` covers both a halt (no transition applies) and an
    /// ambiguity (several apply); `run` treats either as rejection, so only call it on machines
    /// whose determinism flag is set, and use `step_all` to tell the two apart.
    pub fn step(&self, config: &Configuration) -> Option<Configuration> {
        let mut next_configurations = self.step_all(config);
        if next_configurations.len() == 1 {
            return next_configurations.pop();
        }
        None
    }

    pub fn step_all(&self, config: &Configuration) -> Vec<Configuration> {
        self.parsed
            .of(config.state_id)
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

    pub fn is_accepting(&self, config: &Configuration) -> bool {
        self.graph.is_final(config.state_id)
    }

    pub fn run(&self, input: &str, max_steps: usize) -> Option<RunOutcome> {
        let mut config = self.initial_configuration(input)?;
        for _ in 0..max_steps {
            if self.is_accepting(&config) {
                return Some(RunOutcome::Accepted);
            }
            match self.step(&config) {
                Some(next_config) => config = next_config,
                None => return Some(RunOutcome::Rejected),
            }
        }
        if self.is_accepting(&config) {
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
                if self.is_accepting(&config) {
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
        if frontier.iter().any(|config| self.is_accepting(config)) {
            return Some(RunOutcome::Accepted);
        }
        Some(RunOutcome::MaxStepsExceeded)
    }
}

impl StateMachine for TuringMachine {
    fn graph(&self) -> &Graph {
        &self.graph
    }

    fn graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }

    fn is_deterministic(&self) -> bool {
        self.parsed.no_conflicts(|(target, ops), (other_target, other_ops)| {
            let same_reads = ops.len() == other_ops.len()
                && ops.iter().zip(other_ops.iter()).all(|(a, b)| a.read == b.read);
            let identical = target == other_target && ops == other_ops;
            same_reads && !identical
        })
    }

    fn transitions_changed(&mut self, changed: Changed) {
        self.parsed.refresh(&self.graph, changed, parse_transition);
        let still_used = |count: usize| self.parsed.all().any(|(_, ops)| ops.len() == count);
        self.tape_count = match self.tape_count {
            Some(count) if still_used(count) => Some(count),
            _ => self.parsed.all().next().map(|(_, ops)| ops.len()),
        };
    }
}

impl Machine for TuringMachine {
    fn accepts(&self, input: &str) -> bool {
        let outcome = if self.is_deterministic() {
            self.run(input, DEFAULT_MAX_STEPS)
        } else {
            self.run_nondeterministic(input, DEFAULT_MAX_STEPS)
        };
        matches!(outcome, Some(RunOutcome::Accepted))
    }

    fn check_label(&self, label: &str) -> Result<(), String> {
        match (parse_transition(label), self.tape_count) {
            (None, _) => Err(format!(
                "malformed transition label {label:?} (expected \"read;write/dir\" per tape, joined by commas)"
            )),
            (Some(ops), Some(expected)) if ops.len() != expected => Err(format!(
                "inconsistent tape count: label {label:?} uses {} tape(s), but the machine has {expected}",
                ops.len()
            )),
            _ => Ok(()),
        }
    }

    fn validate(&self) -> Result<(), String> {
        self.graph.validate_labels("Turing machine", |label| self.check_label(label))
    }
}
