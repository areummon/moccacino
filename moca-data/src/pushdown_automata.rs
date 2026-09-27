use std::collections::{HashMap, HashSet, BTreeSet};
use crate::state::{Input, State, StateID};
use crate::state_machine::{Machine, MachineKind, StateMachine};

/* Structure that represents a pushdown automaton.
 * The inisital_state_id represents the initial state
 * of the automaton, if the value is None, then some 
 * algorithms and functions will not work.
 * The string_transitions field is used to store all
 * the string transitions the automaton has.
 * The stack represents the stack of the pushdown automaton.
 */
#[derive(Debug, Clone)]
pub struct PushdownAutomata {
    states_by_id: HashMap<StateID, State>,
    string_transitions: HashMap<(StateID, String), (StateID, String)>,
    initial_state_id: Option<StateID>,
    final_states: HashSet<StateID>,
    initial_stack_symbol: String,
    deterministic: bool,
    /* Per-source-state transitions pre-parsed out of the
     * "input;pop/push" labels, maintained by every transition-mutating
     * path so the stepping engine never re-splits label strings. Mirrors
     * exactly what parsing the live State labels would yield: malformed
     * labels and bare-ε quirks included (see `PdaParsedOp`). */
    parsed: HashMap<StateID, Vec<(StateID, PdaParsedOp)>>,
}

/* One well-formed transition, decomposed once. The step-time checks that
 * depend on the configuration (stack-top match, input prefix, stack-depth
 * bound) stay in `successors`. */
#[derive(Debug, Clone, PartialEq)]
struct PdaParsedOp {
    /* The label was exactly "ε": fires on every configuration, leaves the
     * input and stack untouched, and is exempt from the depth bound. */
    is_bare_epsilon: bool,
    /* "" or "ε" means the transition consumes no input. */
    read: String,
    /* False when the popped symbol is "ε" (no pop happens). */
    pops: bool,
    pop: String,
    /* The exact stack.push sequence `stack_transition` would perform for
     * this push side (comma-segmented pushes in reverse order with empty
     * and "ε" segments filtered, legacy per-character pushes in reverse
     * order unfiltered); empty means no push. */
    push_entries: Vec<String>,
}

/* A snapshot of the automaton mid-run: current state, the input not yet
 * consumed and the whole stack (bottom first, top last). */
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

    /* Bottom of the stack first, top last. */
    pub fn stack(&self) -> &[String] {
        &self.stack
    }
}

impl PushdownAutomata {
    /* Only the initial symbol is needed because the stack is only used 
     * to operate the automata, otherwise is not necessary. */
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

    /* Recomputes the parsed table entry of one source state from its live
     * labels; the single funnel through which every transition mutation
     * refreshes the cache. It also refreshes the state's rows of the
     * string-transitions table and the determinism flag, so all three stay
     * consistent after additions, edits and removals alike. */
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
                // Empty comma segments cannot be pushed as entries; the
                // whole transition is malformed and skipped.
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

    /* Deterministic when no state has two distinct transitions that could
     * fire on the same configuration: reads overlap (equal, or either reads
     * nothing) and pops overlap (equal, or either pops nothing). A bare "ε"
     * transition overlaps everything. This is the standard DPDA condition. */
    fn refresh_determinism(&mut self) {
        self.deterministic = self.parsed.values().all(|ops| {
            ops.iter().enumerate().all(|(index, first)| {
                ops[index + 1..].iter().all(|second| !ops_conflict(first, second))
            })
        });
    }

    // Getter for the string transitions of the automata,
    pub fn get_string_transitions(&self) -> &HashMap<(StateID, String), (StateID, String)> {
        &self.string_transitions
    }

    pub fn get_initial_stack_symbol(&self) -> &str {
        &self.initial_stack_symbol
    }

    /* Configuration with the input waiting to be read and the initial stack
     * symbol alone on the stack, or None when the automaton has no initial
     * state. */
    pub fn initial_configuration(&self, input: &str) -> Option<PdaConfiguration> {
        let initial_id = self.initial_state_id?;
        Some(PdaConfiguration {
            state_id: initial_id,
            remaining_input: input.to_string(),
            stack: vec![self.initial_stack_symbol.to_string()],
        })
    }

    /* Acceptance works by accepting states: the current state is final and
     * the input has been fully consumed. */
    pub fn is_accepting(&self, config: &PdaConfiguration) -> bool {
        self.final_states.contains(&config.state_id) && config.remaining_input.is_empty()
    }

    /* Every configuration reachable in one step. Non-consuming expansions
     * and input-matching moves both count as steps; the caller drives
     * exploration (and any search bounds) across `step_all` calls, exactly
     * like the Turing engine's step/step_all split. */
    pub fn step_all(&self, config: &PdaConfiguration) -> Vec<PdaConfiguration> {
        self.successors(config, usize::MAX)
    }

    /* Function to check if a given input string is accepted by the automata,
     * i.e. the final state is final and the input is consumed. 
     * This implementation works with acceptting states (final states).
     * The traversal runs under the default safety bounds; see
     * check_input_with_limit when explicit control is needed. */
    pub fn check_input(&self, input: &mut Input) -> bool {
        let stack_depth = Self::STACK_DEPTH_BASE
            + Self::STACK_DEPTH_PER_CHAR * input.chars().count();
        self.check_input_with_limit(input, Self::MAX_VISITED_CONFIGURATIONS, stack_depth)
    }

    /* Like `check_input` but with explicit exploration bounds: unique
     * configurations visited and maximum stack depth considered. Exceeding
     * either bound reports rejection, so callers needing exact answers on
     * machines that explore deep stacks must raise the limits. */
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
            // Without an initial state no input can be accepted.
            None => false,
        }
    }

    // Function to add a label to a state given by it's id.
    pub fn add_label(&mut self, state_id: StateID, label: BTreeSet<StateID>) {
        if let Some(state) = self.states_by_id.get_mut(&state_id) {
            state.label = label;
        }
    }

    /* Function to traverse the automaton and checking the input string, i.e.
     * Check if the automaton accepts the input string. This function is very
     * similar to the implementation in the finite_automaton function with 
     * the same name, with the stack added.
     * Transitions have the form "input;pop/push". An input part of "" or "ε"
     * does not consume input (it can still operate the stack), and a plain
     * "ε" transition neither consumes input nor operates the stack. Every
     * branch works on its own clone of the input and the stack, so sibling
     * branches cannot corrupt each other on nondeterministic choices.
     *
     * The search is iterative over an explicit worklist of configurations,
     * and the visited set memoizes already explored (state, remaining input,
     * stack) configurations: ε-heavy machines (e.g. grammar-recognizer
     * constructions where expansion rules keep growing the stack) would
     * otherwise overflow the call stack through deep recursion. Any
     * exploration order yields the same acceptance answer, because reaching
     * an accepting state with empty input is order-independent. */
    /* Default safety bounds for `check_input`, shared philosophy with the
     * Turing engine's DEFAULT_MAX_STEPS: pathological machines report
     * rejection instead of grinding forever. Verified fast paths (grammar
     * recognizers on teaching-scale words) stay orders of magnitude below
     * these bounds.
     * The stack-depth guard scales with the input length: expansions that
     * pile up far more pending symbols than the remaining input could ever
     * consume cannot contribute to acceptance. */
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
        // Breadth-first over configuration levels: an accepting path is
        // always found after examining only configurations shallower than
        // its step count, whereas depth-first search can drown in unrelated
        // ε-expansion subtrees before ever backtracking to a short match
        // sequence.
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

    /* One-step successor generation shared by `step_all` and the bounded
     * search inside `traverse`.
     * Transitions have the form "input;pop/push". An input part of "" or "ε"
     * does not consume input (it can still operate the stack), and a plain
     * "ε" transition neither consumes input nor operates the stack. A push
     * side containing commas segments into atomic stack entries (leftmost on
     * top); comma-less pushes keep the legacy per-character decomposition.
     * Branches whose stack would outgrow `max_stack_depth` are pruned —
     * `step_all` passes no bound — and empty comma segments are malformed and
     * skip the transition. */
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
                // A bare "ε" transition consumes nothing, leaves the stack
                // alone and is exempt from the depth bound.
                successors.push(PdaConfiguration {
                    state_id: *id,
                    remaining_input: current_input.clone(),
                    stack: current_stack.clone(),
                });
                continue;
            }
            // A popping transition needs its symbol on top of the stack; a
            // non-popping one ("ε" pop) fires whatever the stack holds.
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
            // Applies the pop and the precomputed push sequence, exactly
            // what `stack_transition` performs for this label.
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

    /* Stores the label and refreshes the parsed table, the string-transitions
     * rows and the determinism flag through `rebuild_parsed_state`. Malformed
     * labels (e.g. no ';') are kept on the state so `validate` can report
     * them, but never reach the engine. Nothing changes when either endpoint
     * is missing. */
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

    /* Cleans the bookkeeping of the automaton when a state is deleted: the
     * final state registry, the initial state id, every entry of the
     * string transitions table that mentions the removed state, and the
     * parsed table (the state's own entry plus the forgotten id as a
     * target anywhere else). */
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
                    // Label shape: bare "ε" or "input;pop/push" with no empty
                    // comma segments on the push side.
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

/* "" and "ε" both mean "nothing" for the input read and the pop. */
fn is_epsilon(symbol: &str) -> bool {
    symbol.is_empty() || symbol == "ε"
}

/* Whether two transitions of one state could both fire on some
 * configuration (identical transitions never conflict). */
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
