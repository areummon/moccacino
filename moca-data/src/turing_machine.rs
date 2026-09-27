/* Turing machine over one or more tapes, with optional nondeterminism.
 *
 * A single-tape transition follows the library-wide convention
 * "read;write/dir" with dir in {L, R, S} (case insensitive). Multitape
 * machines join one such segment per tape with commas, e.g.
 * "a;X/R,b;_/L" for a two-tape machine. The number of tapes of a machine is
 * established by its first well-formed transition; labels whose segment count
 * differs from it are stored (so the GUI can round-trip whatever the user
 * typed) but skipped by the engine and reported by `Machine::validate`.
 *
 * Within one transition all tapes read simultaneously and then write/move, so
 * a tape never observes another tape's write from the same step.
 *
 * Acceptance works on configurations: `initial_configuration` places the input
 * on tape 1 (the remaining tapes start blank), `step`/`step_all` apply
 * transitions matching the scanned symbols, and
 * `run`/`run_nondeterministic` iterate until an accepting state is reached
 * (accepted), every branch halts elsewhere (rejected) or a step budget runs
 * out. When two transitions of a state read the same symbols the machine is
 * flagged as nondeterministic; deterministic runs refuse to pick a branch
 * while nondeterministic ones explore them all. */
use std::collections::{BTreeMap, HashMap, HashSet};
use crate::state::{StateID, Input, State};
use crate::state_machine::{Machine, MachineKind, StateMachine};

/* Step budget used by `Machine::accepts`, which has no budget parameter.
 * Large enough for typical teaching examples, small enough to keep the GUI
 * responsive if a machine loops forever. */
pub const DEFAULT_MAX_STEPS: usize = 10_000;

/* Head movement encoded in a transition label. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Stay,
}

/* Parsed form of a "read;write/dir" transition label. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionOp {
    pub read: char,
    pub write: char,
    pub direction: Direction,
}

/* Parses a transition label into one operation per tape; returns None for
 * malformed labels (wrong field count, multi-character symbols, unknown
 * directions or empty segments). */
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

/* The machine tape: an unbounded strip of cells indexed by integers (negative
 * positions allowed), holding one character each. Unwritten cells read as the
 * blank symbol given at construction time. */
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tape {
    cells: BTreeMap<i64, char>,
    head: i64,
    blank: char,
}

impl Tape {
    /* A tape with the input string written starting at position 0 and the
     * head positioned at its first character. */
    pub fn new_with_input(input: &str, blank: char) -> Self {
        let mut cells = BTreeMap::new();
        for (index, c) in input.chars().enumerate() {
            cells.insert(index as i64, c);
        }
        Tape { cells, head: 0, blank }
    }

    /* Character under the head. */
    pub fn read(&self) -> char {
        self.cells.get(&self.head).copied().unwrap_or(self.blank)
    }

    /* The blank symbol of this tape. */
    pub fn get_blank(&self) -> char {
        self.blank
    }

    /* Writes over the cell under the head. Writing the blank erases the
     * cell instead of materializing an explicit blank entry, which keeps the
     * internal representation canonical (and tape comparisons/hashes exact)
     * and avoids unbounded growth on machines that rewrite blanks. */
    pub fn write(&mut self, c: char) {
        if c == self.blank {
            self.cells.remove(&self.head);
        } else if self.cells.get(&self.head) != Some(&c) {
            self.cells.insert(self.head, c);
        }
    }

    /* Moves the head one cell in the given direction. */
    pub fn move_head(&mut self, direction: Direction) {
        match direction {
            Direction::Left => self.head -= 1,
            Direction::Right => self.head += 1,
            Direction::Stay => (),
        }
    }

    /* Current head position (mainly for debugging/tests). */
    pub fn head_position(&self) -> i64 {
        self.head
    }

    /* Renders a window of `context` cells around the head together with the
     * offset of the head inside the rendered string; unwritten cells render as
     * the blank symbol. This is what the future GUI will use to draw the
     * tape. */
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

/* A snapshot of the machine mid-run: current state plus one tape per machine
 * tape (single-tape machines hold exactly one). */
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Configuration {
    state_id: StateID,
    tapes: Vec<Tape>,
}

impl Configuration {
    pub fn state_id(&self) -> StateID {
        self.state_id
    }

    /* Every tape of the configuration. */
    pub fn tapes(&self) -> &[Tape] {
        &self.tapes
    }

    /* The first tape; the input tape. Single-tape code can use this instead
     * of `tapes()`. */
    pub fn tape(&self) -> &Tape {
        &self.tapes[0]
    }
}

/* Result of running a Turing machine with a step budget. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunOutcome {
    /* Halted in an accepting state. */
    Accepted,
    /* Halted (no applicable transition) in a non-accepting state. */
    Rejected,
    /* The budget ran out before the machine halted. */
    MaxStepsExceeded,
}

/* Structure that represents a Turing machine over one or more tapes. The
 * machine is deterministic as long as no two transitions of a state read the
 * same symbols; conflicting transitions flag it as nondeterministic.
 * The initial_state_id represents the initial state of the machine, if the
 * value is None, then some algorithms and functions will not work.
 * The blank_symbol is shared by every tape and written/read on cells outside
 * of their used portions. The number of tapes is None until the first
 * well-formed transition establishes it (single-tape machines keep the
 * default of one). Accepting states are the regular final states. */
#[derive(Debug, Clone)]
pub struct TuringMachine {
    states_by_id: HashMap<StateID, State>,
    initial_state_id: Option<StateID>,
    final_states: HashSet<StateID>,
    blank_symbol: char,
    deterministic: bool,
    tape_count: Option<usize>,
    /* Per-source-state transitions pre-parsed into per-tape operations,
     * maintained by every transition-mutating path (add_transition,
     * modify_input, remove_transition, forget_state) so the stepping
     * engine never re-parses label strings. Mirrors exactly what parsing
     * the live State labels would yield. */
    parsed: HashMap<StateID, Vec<(StateID, Vec<TransitionOp>)>>,
}

impl TuringMachine {
    /* Creates an empty single-tape machine over the given blank symbol. The
     * tape count only changes if the first well-formed transition added has
     * several comma-joined segments. */
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

    /* Number of tapes used to build configurations. The first well-formed
     * transition establishes it; before that (or on machines without
     * transitions) it defaults to one tape carrying the input. */
    pub fn get_tape_count(&self) -> usize {
        self.tape_count.unwrap_or(1)
    }

    /* Recomputes the parsed table entry of one source state from its live
     * labels; the single funnel through which every transition mutation
     * refreshes the cache. Malformed labels are absent, exactly as the
     * engine skipped them when parsing per step. */
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

    /* Recomputes the determinism flag from the parsed table: two different
     * transitions of one state reading the same symbols on every tape make
     * the machine nondeterministic. A machine left without any well-formed
     * transition forgets its tape count, so the next one establishes it. */
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

    // Getter for the transitions of a state parsed into their per-tape
    // operations, served from the pre-parsed table. Malformed labels are
    // skipped by the engine, they are reported by Machine::validate instead.
    fn transitions_of(&self, state_id: &StateID) -> &[(StateID, Vec<TransitionOp>)] {
        self.parsed
            .get(state_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /* Configuration with the input placed on tape 1 and every remaining tape
     * blank, or None when the machine has no initial state. */
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

    /* Applies the unique well-formed transition matching the scanned symbols.
     * Returns None when the machine halts (no transition matches on some
     * tape) or when it is not deterministic (ambiguous next step). Use
     * `step_all` to explore every branch of a nondeterministic machine. */
    pub fn step(&self, config: &Configuration) -> Option<Configuration> {
        let mut next_configurations = self.step_all(config);
        if next_configurations.len() == 1 {
            return next_configurations.pop();
        }
        None
    }

    /* Every configuration reachable in one step by applying a well-formed
     * transition whose read symbols match all tapes simultaneously. Empty
     * means the branch halts; more than one element means the machine
     * branches here. */
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

    /* Runs the machine on the input until it halts or the budget runs out.
     * An accepting state reached at any point means acceptance (the check
     * happens before stepping), halting anywhere else means rejection, and
     * exhausting `max_steps` returns MaxStepsExceeded. Returns None when the
     * machine has no initial state. */
    pub fn run(&self, input: &str, max_steps: usize) -> Option<RunOutcome> {
        let mut config = self.initial_configuration(input)?;
        for _ in 0..max_steps {
            if self.final_states.contains(&config.state_id) {
                return Some(RunOutcome::Accepted);
            }
            match self.step(&config) {
                Some(next_config) => config = next_config,
                // Halted outside an accepting state.
                None => return Some(RunOutcome::Rejected),
            }
        }
        // The last step may have just arrived in an accepting state.
        if self.final_states.contains(&config.state_id) {
            return Some(RunOutcome::Accepted);
        }
        Some(RunOutcome::MaxStepsExceeded)
    }

    /* Nondeterministic run: explores every branch of the computation in
     * breadth-first order over parallel levels (the standard nondeterministic
     * time measure), so `max_steps` counts configuration-tree levels rather
     * than individual transitions. Identical configurations reached through
     * different branches are explored only once.
     *
     * Accepted as soon as any branch reaches an accepting state; Rejected
     * when every branch halts without doing so; MaxStepsExceeded while at
     * least one branch is still running. Returns None when the machine has
     * no initial state. */
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
                // Every branch halted without reaching an accepting state.
                return Some(RunOutcome::Rejected);
            }
            frontier = next_frontier;
        }
        // The last level may have just arrived in an accepting state.
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

    fn is_deterministic(&self) -> bool {
        self.deterministic
    }

    fn get_final_states(&self) -> &HashSet<StateID> {
        &self.final_states
    }

    fn get_initial_state_id(&self) -> &Option<StateID> {
        &self.initial_state_id
    }

    /* The first well-formed transition establishes the tape count of the
     * machine; the determinism flag (two well-formed transitions of a state
     * reading the same symbols but differing in target, writes or moves) is
     * recomputed by `rebuild_parsed_state`, so identical duplicates never
     * flip it and removals restore it. Nothing changes when either endpoint
     * is missing. */
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

    /* Keeps the parsed table in sync when a transition label is edited in
     * place through the trait default. */
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

    /* Keeps the parsed table in sync when a transition is removed through
     * the trait default. */
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

    /* Function to make a state final; unknown ids are ignored, like in
     * make_initial. */
    fn make_final(&mut self, state_id: StateID) {
        if let Some(state) = self.states_by_id.get_mut(&state_id) {
            state.final_flag = true;
            self.final_states.insert(state_id);
        }
    }

    /* Besides the shared registries, the parsed transition table mentions
     * state ids too: drop the state's own entry and scrub it as a target
     * from every other entry (mirroring State::remove_state). */
    fn forget_state(&mut self, state_id: StateID) {
        self.final_states.remove(&state_id);
        if self.initial_state_id == Some(state_id) {
            self.initial_state_id = None;
        }
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

    /* Uses the default step budget; prefer `run`/`run_nondeterministic` when
     * a custom budget is needed (e.g. to detect looping machines). The run
     * style is picked from the determinism flag: ambiguous transitions are
     * refused by `step` on deterministic machines and explored as branches
     * otherwise. */
    fn accepts(&self, input: &str) -> bool {
        let outcome = if self.deterministic {
            self.run(input, DEFAULT_MAX_STEPS)
        } else {
            self.run_nondeterministic(input, DEFAULT_MAX_STEPS)
        };
        matches!(outcome, Some(RunOutcome::Accepted))
    }

    /* Strict validation: besides structural checks, every transition label
     * must parse as comma-joined "read;write/dir" segments and match the
     * tape count established by the machine. */
    fn validate(&self) -> Result<(), String> {
        if self.initial_state_id.is_none() {
            return Err("The Turing machine has no initial state.".to_string());
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
                    match parse_transition(label) {
                        None => {
                            return Err(format!(
                                "State {} has malformed transition label {:?} (expected \"read;write/dir\" per tape, joined by commas).",
                                id, label
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
