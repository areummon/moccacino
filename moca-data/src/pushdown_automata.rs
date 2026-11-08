use std::collections::{HashSet, VecDeque};
use std::fmt;
use crate::state::{is_epsilon, Input, StateID, EPSILON};
use crate::state_machine::{Changed, Graph, Machine, ParsedLabels, StateMachine};

pub const DEFAULT_STACK: &str = "Z";

#[derive(Debug, Clone)]
pub struct PushdownAutomata {
    graph: Graph,
    initial_stack_symbol: String,
    parsed: ParsedLabels<PdaOp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdaOp {
    pub read: Option<String>,
    pub pop: Option<String>,
    pub push_entries: Vec<String>,
}

impl PdaOp {
    pub fn parse(label: &str) -> Result<Self, &'static str> {
        const SHAPE: &str = "expected \"input;pop/push\"";
        if label == EPSILON {
            return Ok(PdaOp { read: None, pop: None, push_entries: Vec::new() });
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
                .filter(|part| *part != EPSILON)
                .map(str::to_string)
                .collect()
        } else {
            push.chars().rev().map(|symbol| symbol.to_string()).collect()
        };
        let symbol = |text: &str| (!is_epsilon(text)).then(|| text.to_string());
        Ok(PdaOp { read: symbol(read), pop: symbol(pop), push_entries })
    }

    pub fn is_reserved_symbol(symbol: &str) -> bool {
        symbol == EPSILON || symbol.contains([',', ';', '/'])
    }

    fn push_text(&self) -> String {
        if self.push_entries.is_empty() {
            return EPSILON.to_string();
        }
        let top_first = self.push_entries.iter().rev();
        if self.push_entries.iter().all(|entry| entry.chars().count() == 1) {
            return top_first.map(String::as_str).collect();
        }
        let mut text = top_first.map(String::as_str).collect::<Vec<_>>().join(",");
        if self.push_entries.len() == 1 {
            text.push(',');
            text.push_str(EPSILON);
        }
        text
    }
}

impl fmt::Display for PdaOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let read = self.read.as_deref().unwrap_or(EPSILON);
        let pop = self.pop.as_deref().unwrap_or(EPSILON);
        write!(f, "{read};{pop}/{}", self.push_text())
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
    const MAX_VISITED_CONFIGURATIONS: usize = 150_000;
    const STACK_DEPTH_PER_CHAR: usize = 2;
    const STACK_DEPTH_BASE: usize = 16;

    pub fn new(initial_stack_symbol: String) -> Self {
        PushdownAutomata {
            graph: Graph::default(),
            initial_stack_symbol,
            parsed: ParsedLabels::default(),
        }
    }

    pub fn get_initial_stack_symbol(&self) -> &str {
        &self.initial_stack_symbol
    }

    pub fn initial_configuration(&self, input: &str) -> Option<PdaConfiguration> {
        Some(PdaConfiguration {
            state_id: self.graph.initial()?,
            remaining_input: input.to_string(),
            stack: vec![self.initial_stack_symbol.to_string()],
        })
    }

    pub fn is_accepting(&self, config: &PdaConfiguration) -> bool {
        self.graph.accepts_at(config.state_id, &config.remaining_input)
    }

    pub fn step_all(&self, config: &PdaConfiguration) -> Vec<PdaConfiguration> {
        self.successors(config, usize::MAX)
    }

    pub fn check_input(&self, input: &str) -> bool {
        let stack_depth = Self::STACK_DEPTH_BASE
            + Self::STACK_DEPTH_PER_CHAR * input.chars().count();
        self.check_input_with_limit(input, Self::MAX_VISITED_CONFIGURATIONS, stack_depth)
    }

    fn check_input_with_limit(
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
        let current_stack = &config.stack;
        self.parsed
            .of(config.state_id)
            .iter()
            .filter_map(|(target, op)| {
                if op.pop.is_some() && current_stack.last() != op.pop.as_ref() {
                    return None;
                }
                let next_stack_len = current_stack.len() - usize::from(op.pop.is_some()) + op.push_entries.len();
                if next_stack_len > max_stack_depth && next_stack_len > current_stack.len() {
                    return None;
                }
                let consumed = match &op.read {
                    None => 0,
                    Some(read) if config.remaining_input.starts_with(read.as_str()) => read.len(),
                    Some(_) => return None,
                };
                let mut stack = current_stack.clone();
                if op.pop.is_some() {
                    stack.pop();
                }
                stack.extend(op.push_entries.iter().cloned());
                Some(PdaConfiguration {
                    state_id: *target,
                    remaining_input: config.remaining_input[consumed..].to_string(),
                    stack,
                })
            })
            .collect()
    }
}

impl StateMachine for PushdownAutomata {
    fn graph(&self) -> &Graph {
        &self.graph
    }

    fn graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }

    fn is_deterministic(&self) -> bool {
        self.parsed.no_conflicts(|first, second| {
            if first == second {
                return false;
            }
            let ((_, a), (_, b)) = (first, second);
            let reads_overlap = a.read.is_none() || b.read.is_none() || a.read == b.read;
            let pops_overlap = a.pop.is_none() || b.pop.is_none() || a.pop == b.pop;
            reads_overlap && pops_overlap
        })
    }

    fn transitions_changed(&mut self, changed: Changed) {
        self.parsed.refresh(&self.graph, changed, |label| PdaOp::parse(label).ok());
    }
}

impl Machine for PushdownAutomata {
    fn accepts(&self, input: &str) -> bool {
        self.check_input(input)
    }

    fn check_label(&self, label: &str) -> Result<(), String> {
        PdaOp::parse(label)
            .map(drop)
            .map_err(|problem| format!("malformed transition label {label:?} ({problem})"))
    }

    fn validate(&self) -> Result<(), String> {
        self.graph.validate_labels("pushdown automaton", |label| self.check_label(label))
    }
}
