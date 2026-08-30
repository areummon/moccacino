/* Loader for `.ce` ("computational entities") files: a small key-based
 * text format declaring any number of machines, regular expressions and
 * grammars in one document. Each `entity:` block is parsed and built
 * independently, so one broken entity never sinks the rest of the file.
 *
 * Transition syntax is structurally distinct per family, which keeps
 * hand-written and LLM-generated files unambiguous:
 *   finite: (from, symbol) -> to
 *   pda:    (from, input, pop, push...) -> to   (push tail may contain commas)
 *   tm:     (from, read, write, dir[, read, write, dir...]) -> to
 * The internal repo labels (`input;pop/push`, `read;write/dir`, tapes
 * comma-joined) are assembled from these parts and then validated. */

use crate::finite_automata::FiniteAutomata;
use crate::grammar::{parse_grammar, Grammar};
use crate::pushdown_automata::PushdownAutomata;
use crate::regex;
use crate::state::{State, StateID};
use crate::state_machine::{Machine, StateMachine};
use crate::turing_machine::TuringMachine;
use std::collections::{HashMap, HashSet};

/* A built computational entity ready to be opened as a tab. */
#[derive(Debug)]
pub enum Entity {
    Finite(FiniteAutomata),
    Pushdown(PushdownAutomata),
    Turing(TuringMachine),
    Grammar(Grammar),
}

/* One successfully loaded entity: a usable display name, the alias the
 * file declared it with, and the object itself. */
#[derive(Debug)]
pub struct NamedEntity {
    pub name: String,
    pub kind_label: String,
    pub entity: Entity,
}

/* One failed entity (or file-level problem). `entity_name` is empty for
 * file-level errors; `line` is 1-based. */
#[derive(Debug)]
pub struct EntityError {
    pub entity_name: String,
    pub line: usize,
    pub message: String,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Turing,
    Pushdown,
    Finite,
    Regex,
    Grammar,
}

/* Maps the declared alias to its canonical kind and default tab name. */
fn resolve_kind(alias: &str) -> Option<(Kind, &'static str)> {
    match alias {
        "tm" | "turing" => Some((Kind::Turing, "TM")),
        "pda" | "pushdown" => Some((Kind::Pushdown, "Pushdown")),
        "dfa" => Some((Kind::Finite, "DFA")),
        "nfa" => Some((Kind::Finite, "NFA")),
        "fa" | "automata" | "automaton" | "finite" => Some((Kind::Finite, "FA")),
        "regex" | "re" => Some((Kind::Regex, "Regex")),
        "grammar" | "cfg" => Some((Kind::Grammar, "Grammar")),
        _ => None,
    }
}

fn normalize_final_key(key: &str) -> bool {
    matches!(key, "final" | "finals" | "halt" | "final/halt" | "final_halt")
}

/* Parses the whole file: valid entities come back in file order, broken
 * ones come back as errors (with the rest of the file unaffected). */
pub fn parse_entity_file(source: &str) -> (Vec<NamedEntity>, Vec<EntityError>) {
    struct Block {
        header_line: usize,
        kind_text: String,
        entries: Vec<(usize, String, String)>,
    }

    let mut blocks: Vec<Block> = Vec::new();
    let mut errors: Vec<EntityError> = Vec::new();
    let mut entities: Vec<NamedEntity> = Vec::new();

    for (index, raw_line) in source.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(kind_text) = strip_entity_header(line) {
            blocks.push(Block {
                header_line: line_number,
                kind_text: kind_text.to_string(),
                entries: Vec::new(),
            });
            continue;
        }
        let entry = match line.split_once(':') {
            Some((key, value)) => (
                line_number,
                key.trim().to_ascii_lowercase(),
                value.trim().to_string(),
            ),
            None => (line_number, String::new(), line.to_string()),
        };
        match blocks.last_mut() {
            Some(block) => block.entries.push(entry),
            None => errors.push(EntityError {
                entity_name: String::new(),
                line: line_number,
                message: "content before the first 'entity:' block".to_string(),
            }),
        }
    }

    for block in &blocks {
        match build_block(block.header_line, &block.kind_text, &block.entries) {
            Ok(entity) => entities.push(entity),
            Err((entity_name, line, message)) => errors.push(EntityError {
                entity_name,
                line,
                message,
            }),
        }
    }

    (entities, errors)
}

/* Matches an `entity: <kind>` header line and returns the kind text. */
fn strip_entity_header(line: &str) -> Option<&str> {
    let (key, value) = line.split_once(':')?;
    if key.trim().eq_ignore_ascii_case("entity") {
        Some(value.trim())
    } else {
        None
    }
}

type BlockError = (String, usize, String);

/* Assigns integer ids to state labels on first sight (auto-registering
 * labels that only appear in transitions or finals). */
#[derive(Default)]
struct Interner {
    ids: HashMap<String, u64>,
    next_id: u64,
}

impl Interner {
    fn intern(&mut self, label: &str) -> Option<u64> {
        let trimmed = label.trim();
        if trimmed.is_empty() {
            return None;
        }
        let next_id = &mut self.next_id;
        Some(
            *self
                .ids
                .entry(trimmed.to_string())
                .or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                }),
        )
    }
}

/* Splits a `transitions:` value on commas that sit outside parentheses,
 * so multitape TM labels like `_ ; _ / S` groups stay intact. */
fn split_tuples(value: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    for c in value.chars() {
        match c {
            '(' => {
                depth += 1;
                current.push(c);
            }
            ')' => {
                if depth > 0 {
                    depth -= 1;
                }
                current.push(c);
            }
            ',' if depth == 0 => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    parts.push(trimmed);
                }
                current.clear();
            }
            _ => current.push(c),
        }
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        parts.push(trimmed);
    }
    parts
}

/* Parses one `(from, parts...) -> to` tuple. */
fn parse_transition_tuple(tuple: &str) -> Result<(String, Vec<String>, String), String> {
    let trimmed = tuple.trim();
    if !trimmed.starts_with('(') {
        return Err(format!(
            "expected a '(state, ...) -> state' tuple, found '{}'",
            trimmed
        ));
    }
    let close = trimmed
        .rfind(')')
        .ok_or_else(|| format!("missing ')' in '{}'", trimmed))?;
    let inner = trimmed[1..close].trim();
    let rest = trimmed[close + 1..].trim();
    let target = rest
        .strip_prefix("->")
        .ok_or_else(|| format!("missing '->' in '{}'", trimmed))?
        .trim();
    if target.is_empty() {
        return Err(format!("missing target state in '{}'", trimmed));
    }
    if inner.is_empty() {
        return Err(format!("missing states in '{}'", trimmed));
    }
    let parts: Vec<String> = inner.split(',').map(|part| part.trim().to_string()).collect();
    Ok((parts[0].clone(), parts, target.to_string()))
}

/* Blank or whitespace-only components mean ε on finite and PDA labels. */
fn normalize_epsilon(component: &str) -> String {
    let trimmed = component.trim();
    if trimmed.is_empty() {
        "ε".to_string()
    } else {
        trimmed.to_string()
    }
}

fn split_csv(value: &str) -> Vec<String> {
    value.split(',').map(|part| part.trim().to_string()).collect()
}

/* Builds one entity block: distributes keys, enforces family-specific
 * transition arity, assembles internal labels and validates the result.
 * The returned error carries the entity name, the offending line and a
 * human-readable message. */
fn build_block(
    header_line: usize,
    kind_text: &str,
    entries: &[(usize, String, String)],
) -> Result<NamedEntity, BlockError> {
    let (kind, fallback_name) =
        resolve_kind(&kind_text.to_ascii_lowercase()).ok_or_else(|| {
            (
                format!("entity at line {}", header_line),
                header_line,
                format!("unknown entity kind '{}'", kind_text),
            )
        })?;

    // Errors report the declared name when there is one, else the kind's
    // default name, else a positional description.
    let declared_name = entries
        .iter()
        .find(|(_, key, _)| key == "name")
        .map(|(_, _, value)| value.clone())
        .unwrap_or_default();
    let fallback = fallback_name.to_string();
    let error_name = || {
        if !declared_name.is_empty() {
            declared_name.clone()
        } else {
            fallback.clone()
        }
    };
    let err = move |line: usize, message: String| -> BlockError { (error_name(), line, message) };

    let mut name: Option<String> = None;
    let mut states: Vec<String> = Vec::new();
    let mut transitions: Vec<(usize, String)> = Vec::new();
    let mut initial: Option<(usize, String)> = None;
    let mut finals: Vec<String> = Vec::new();
    let mut blank: Option<(usize, String)> = None;
    let mut stack: Option<(usize, String)> = None;
    let mut regex_pattern: Option<(usize, String)> = None;
    let mut productions: Vec<(usize, String)> = Vec::new();

    for (line, key, value) in entries {
        if key == "name" {
            if name.is_none() && !value.is_empty() {
                name = Some(value.clone());
            }
        } else if key == "states" {
            if matches!(kind, Kind::Regex | Kind::Grammar) {
                return Err(err(
                    *line,
                    format!("'states:' is not valid for {} entities", fallback_name),
                ));
            }
            states.extend(split_csv(value));
        } else if key == "transitions" {
            if matches!(kind, Kind::Regex | Kind::Grammar) {
                return Err(err(
                    *line,
                    format!(
                        "'transitions:' is not valid for {} entities",
                        fallback_name
                    ),
                ));
            }
            transitions.push((*line, value.clone()));
        } else if key == "initial" {
            if matches!(kind, Kind::Regex | Kind::Grammar) {
                return Err(err(
                    *line,
                    format!(
                        "'initial:' is not valid for {} entities",
                        fallback_name
                    ),
                ));
            }
            if initial.is_none() {
                initial = Some((*line, value.clone()));
            }
        } else if normalize_final_key(key) {
            if matches!(kind, Kind::Regex | Kind::Grammar) {
                return Err(err(
                    *line,
                    format!("'{}:' is not valid for {} entities", key, fallback_name),
                ));
            }
            finals.extend(split_csv(value));
        } else if key == "blank" {
            if !matches!(kind, Kind::Turing) {
                return Err(err(
                    *line,
                    "'blank:' is only valid for turing machine entities".to_string(),
                ));
            }
            if blank.is_none() {
                blank = Some((*line, value.clone()));
            }
        } else if key == "stack" {
            if !matches!(kind, Kind::Pushdown) {
                return Err(err(
                    *line,
                    "'stack:' is only valid for pushdown entities".to_string(),
                ));
            }
            if stack.is_none() {
                stack = Some((*line, value.clone()));
            }
        } else if key == "regex" {
            if !matches!(kind, Kind::Regex) {
                return Err(err(
                    *line,
                    "'regex:' is only valid for regex entities".to_string(),
                ));
            }
            if regex_pattern.is_none() {
                regex_pattern = Some((*line, value.clone()));
            }
        } else if key == "productions" {
            if !matches!(kind, Kind::Grammar) {
                return Err(err(
                    *line,
                    "'productions:' is only valid for grammar entities".to_string(),
                ));
            }
            productions.push((*line, value.clone()));
        } else if key.is_empty() {
            return Err(err(
                *line,
                "expected 'key: value' lines inside an entity block".to_string(),
            ));
        } else {
            return Err(err(*line, format!("unknown key '{}:'", key)));
        }
    }

    let name = name.unwrap_or_else(|| fallback_name.to_string());

    let entity = match kind {
        Kind::Regex => {
            let (line, pattern) = regex_pattern
                .ok_or_else(|| err(header_line, "missing 'regex:'".to_string()))?;
            if pattern.is_empty() {
                return Err(err(line, "empty 'regex:' value".to_string()));
            }
            let automaton = regex::compile_str(&pattern)
                .map_err(|error| err(line, format!("invalid regex: {}", error)))?;
            Entity::Finite(automaton)
        }
        Kind::Grammar => {
            if productions.is_empty() {
                return Err(err(header_line, "missing 'productions:'".to_string()));
            }
            let first_line = productions[0].0;
            let source = productions
                .iter()
                .map(|(_, value)| value.clone())
                .collect::<Vec<_>>()
                .join("\n");
            let grammar = parse_grammar(&source).map_err(|error| {
                err(
                    first_line + error.line - 1,
                    format!("invalid grammar: {}", error.message),
                )
            })?;
            if grammar.productions().is_empty() {
                return Err(err(first_line, "grammar has no productions".to_string()));
            }
            Entity::Grammar(grammar)
        }
        Kind::Finite | Kind::Pushdown | Kind::Turing => {
            build_machine(kind, header_line, &blank, &stack, &states, &transitions, &initial, &finals, &err)?
        }
    };

    Ok(NamedEntity {
        name,
        kind_label: kind_text.to_string(),
        entity,
    })
}

/* Builds a FiniteAutomata, PushdownAutomata or TuringMachine from the
 * collected block keys. */
fn build_machine(
    kind: Kind,
    header_line: usize,
    blank: &Option<(usize, String)>,
    stack: &Option<(usize, String)>,
    states: &[String],
    transitions: &[(usize, String)],
    initial: &Option<(usize, String)>,
    finals: &[String],
    err: &dyn Fn(usize, String) -> BlockError,
) -> Result<Entity, BlockError> {
    let blank_symbol = if matches!(kind, Kind::Turing) {
        let (line, value) = blank
            .as_ref()
            .map(|(line, value)| (*line, value.clone()))
            .unwrap_or((header_line, "_".to_string()));
        let trimmed = value.trim();
        if trimmed.chars().count() != 1 {
            return Err(err(line, "'blank:' must be a single character".to_string()));
        }
        trimmed.chars().next().unwrap()
    } else {
        '_'
    };

    let stack_symbol = if matches!(kind, Kind::Pushdown) {
        let (line, value) = stack
            .as_ref()
            .map(|(line, value)| (*line, value.clone()))
            .unwrap_or((header_line, "Z".to_string()));
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(err(line, "empty 'stack:' value".to_string()));
        }
        trimmed.to_string()
    } else {
        String::new()
    };

    let mut interner = Interner::default();
    for label in states {
        interner
            .intern(label)
            .ok_or_else(|| err(header_line, "empty state name in 'states:'".to_string()))?;
    }

    // Assemble internal labels from the family-specific tuples.
    let mut assembled: Vec<(u64, u64, String)> = Vec::new();
    let mut tape_count: Option<usize> = None;
    for (line, raw) in transitions {
        for tuple in split_tuples(raw) {
            let (from, parts, to) =
                parse_transition_tuple(&tuple).map_err(|message| err(*line, message))?;
            let from_id = interner
                .intern(&from)
                .ok_or_else(|| err(*line, "empty state name in transition".to_string()))?;
            let to_id = interner
                .intern(&to)
                .ok_or_else(|| err(*line, "empty state name in transition".to_string()))?;
            let label = match kind {
                Kind::Finite => {
                    if parts.len() != 2 {
                        return Err(err(
                            *line,
                            format!(
                                "finite transitions take exactly 2 parts (from, symbol), found {} part(s)",
                                parts.len()
                            ),
                        ));
                    }
                    normalize_epsilon(&parts[1])
                }
                Kind::Pushdown => {
                    if parts.len() < 4 {
                        return Err(err(
                            *line,
                            format!(
                                "pushdown transitions take at least 4 parts (from, input, pop, push), found {} part(s)",
                                parts.len()
                            ),
                        ));
                    }
                    let push = parts[3..]
                        .iter()
                        .map(|part| normalize_epsilon(part))
                        .collect::<Vec<_>>()
                        .join(",");
                    format!(
                        "{};{}/{}",
                        normalize_epsilon(&parts[1]),
                        normalize_epsilon(&parts[2]),
                        push
                    )
                }
                Kind::Turing => {
                    if parts.len() < 4 || (parts.len() - 1) % 3 != 0 {
                        return Err(err(
                            *line,
                            format!(
                                "turing transitions take (from, read, write, dir) groups of 4, 7, 10... parts, found {} part(s)",
                                parts.len()
                            ),
                        ));
                    }
                    let tapes = (parts.len() - 1) / 3;
                    match tape_count {
                        Some(existing) if existing != tapes => {
                            return Err(err(
                                *line,
                                format!(
                                    "inconsistent tape count: this transition uses {}, earlier ones use {}",
                                    tapes, existing
                                ),
                            ));
                        }
                        None => tape_count = Some(tapes),
                        _ => {}
                    }
                    let mut pieces = Vec::new();
                    for group in parts[1..].chunks(3) {
                        let read = group[0].trim();
                        let write = group[1].trim();
                        let direction = group[2].trim().to_ascii_uppercase();
                        if read.is_empty() || write.is_empty() {
                            return Err(err(
                                *line,
                                "turing read/write symbols cannot be empty (use the blank symbol)".to_string(),
                            ));
                        }
                        if !matches!(direction.as_str(), "L" | "R" | "S") {
                            return Err(err(
                                *line,
                                format!(
                                    "invalid direction '{}' (use L, R or S)",
                                    group[2].trim()
                                ),
                            ));
                        }
                        pieces.push(format!("{};{}/{}", read, write, direction));
                    }
                    pieces.join(",")
                }
                Kind::Regex | Kind::Grammar => unreachable!(),
            };
            assembled.push((from_id, to_id, label));
        }
    }

    let (initial_line, initial_value) = initial
        .as_ref()
        .map(|(line, value)| (*line, value.clone()))
        .ok_or_else(|| err(header_line, "missing 'initial:'".to_string()))?;
    let initial_trimmed = initial_value.trim();
    if initial_trimmed.is_empty() || initial_trimmed.contains(',') {
        return Err(err(
            initial_line,
            "'initial:' must be a single state name".to_string(),
        ));
    }
    let initial_id = interner
        .intern(initial_trimmed)
        .ok_or_else(|| err(initial_line, "empty state name in 'initial:'".to_string()))?;
    let mut final_ids = Vec::new();
    for label in finals {
        if label.trim().is_empty() {
            continue;
        }
        let id = interner
            .intern(label)
            .ok_or_else(|| err(header_line, "empty state name in finals".to_string()))?;
        final_ids.push(id);
    }

    let report_validation = |machine_problem: String| -> BlockError {
        let line = transitions.first().map(|(line, _)| *line).unwrap_or(header_line);
        err(line, machine_problem)
    };

    let entity = match kind {
        Kind::Finite => {
            // `new()`, not `default()`: the derived Default leaves the
            // determinism flag false, which would mislabel the entity on
            // a subsequent save.
            let mut machine = FiniteAutomata::new();
            for (label, &id) in &interner.ids {
                machine.add_state_with_id_label(id, label);
            }
            for (from, to, label) in assembled {
                machine.add_transition(from, to, label);
            }
            machine.make_initial(initial_id);
            for id in final_ids {
                machine.make_final(id);
            }
            Machine::validate(&machine).map_err(report_validation)?;
            Entity::Finite(machine)
        }
        Kind::Pushdown => {
            let mut machine = PushdownAutomata::new(stack_symbol);
            for (label, &id) in &interner.ids {
                machine.add_state_with_id_label(id, label);
            }
            for (from, to, label) in assembled {
                machine.add_transition(from, to, label);
            }
            machine.make_initial(initial_id);
            for id in final_ids {
                machine.make_final(id);
            }
            Machine::validate(&machine).map_err(report_validation)?;
            Entity::Pushdown(machine)
        }
        Kind::Turing => {
            let mut machine = TuringMachine::new(blank_symbol);
            for (label, &id) in &interner.ids {
                machine.add_state_with_id_label(id, label);
            }
            for (from, to, label) in assembled {
                machine.add_transition(from, to, label);
            }
            machine.make_initial(initial_id);
            for id in final_ids {
                machine.make_final(id);
            }
            Machine::validate(&machine).map_err(report_validation)?;
            Entity::Turing(machine)
        }
        Kind::Regex | Kind::Grammar => unreachable!(),
    };
    Ok(entity)
}

/* ---------- serialization: entities -> .ce text ---------- */

/* Writes one `entity:` block per call, mirroring the keys `build_block`
 * consumes. Every writer rejects empty entities and labels the tuple
 * format cannot round-trip, so saved files always reload. */

fn entity_header(kind_alias: &str, name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .collect();
    let trimmed = sanitized.trim();
    let name = if trimmed.is_empty() { "entity" } else { trimmed };
    format!("entity: {}\nname: {}", kind_alias, name)
}

/* Trims a component of a transition tuple and rejects the characters the
 * tuple layer itself parses: commas segment tuple parts and the states
 * list, parentheses bracket tuples (and steer `split_tuples` depth
 * counting), line breaks would splice the line-oriented format. Push
 * tails pass `allow_comma` because the loader segments them on commas. */
fn check_tuple_component(component: &str, role: &str, allow_comma: bool) -> Result<String, String> {
    let trimmed = component.trim();
    for c in trimmed.chars() {
        let reserved = match c {
            ',' => !allow_comma,
            '(' | ')' | '\n' | '\r' => true,
            _ => false,
        };
        if reserved {
            return Err(format!(
                "{} '{}' uses '{}' which the .ce format reserves for its structure",
                role, trimmed, c
            ));
        }
    }
    Ok(trimmed.to_string())
}

/* Shared machine-block core: states/initial/finals scaffolding plus the
 * sorted `transitions:` lines, delegating per-family label decomposition
 * to `label_to_parts` (the components after the source state name). */
fn write_machine_block<F>(
    kind_alias: &str,
    name: &str,
    empty_message: &str,
    no_initial_message: &str,
    states: &HashMap<StateID, State>,
    initial: Option<StateID>,
    finals: &HashSet<u64>,
    extra_header_key: Option<String>,
    label_to_parts: F,
) -> Result<String, String>
where
    F: Fn(&str) -> Result<Vec<String>, String>,
{
    if states.is_empty() {
        return Err(empty_message.to_string());
    }
    let mut ids: Vec<StateID> = states.keys().copied().collect();
    ids.sort_unstable();
    let mut labels: Vec<String> = Vec::with_capacity(ids.len());
    let mut seen: HashSet<String> = HashSet::new();
    for id in &ids {
        let label = check_tuple_component(&states[id].name, "state name", false)?;
        if label.is_empty() {
            return Err(format!("state {} has an empty name", id));
        }
        if !seen.insert(label.clone()) {
            return Err(format!("state name '{}' is used more than once", label));
        }
        labels.push(label);
    }
    let name_of: HashMap<StateID, &String> =
        ids.iter().copied().zip(labels.iter()).collect();
    let initial_id = initial.ok_or_else(|| no_initial_message.to_string())?;
    if !states.contains_key(&initial_id) {
        return Err(format!("initial state {} is missing", initial_id));
    }

    let mut text = entity_header(kind_alias, name);
    text.push_str("\nstates: ");
    text.push_str(&labels.join(", "));

    let mut triples: Vec<(StateID, StateID, String)> = Vec::new();
    for (from_id, state) in states {
        for (to_id, inputs) in state.iter_by_transition() {
            for label in inputs {
                triples.push((*from_id, *to_id, label.clone()));
            }
        }
    }
    triples.sort();
    for (from_id, to_id, label) in triples {
        let parts = label_to_parts(&label)?;
        let from_label = name_of.get(&from_id).ok_or_else(|| {
            format!("transition leaves state {}, which is missing", from_id)
        })?;
        let to_label = name_of.get(&to_id).ok_or_else(|| {
            format!("transition targets state {}, which is missing", to_id)
        })?;
        text.push_str(&format!(
            "\ntransitions: ({}, {}) -> {}",
            from_label,
            parts.join(", "),
            to_label
        ));
    }
    if let Some(extra) = extra_header_key {
        text.push('\n');
        text.push_str(&extra);
    }
    text.push_str(&format!("\ninitial: {}", name_of[&initial_id]));
    let mut final_ids: Vec<StateID> = finals
        .iter()
        .filter(|id| states.contains_key(id))
        .copied()
        .collect();
    final_ids.sort_unstable();
    if !final_ids.is_empty() {
        let final_labels: Vec<String> = final_ids
            .iter()
            .map(|id| name_of[id].clone())
            .collect();
        text.push_str(&format!("\nfinal: {}", final_labels.join(", ")));
    }
    Ok(text)
}

pub fn write_finite_entity(name: &str, fa: &FiniteAutomata) -> Result<String, String> {
    let alias = if fa.is_deterministic() { "dfa" } else { "nfa" };
    write_machine_block(
        alias,
        name,
        "the finite automaton has no states",
        "the finite automaton has no initial state",
        fa.get_states_by_id_ref(),
        *fa.get_initial_state_id(),
        fa.get_final_states(),
        None,
        |label| {
            // A blank symbol is the loader's ε spelling too.
            let symbol = check_tuple_component(label, "transition symbol", false)?;
            Ok(vec![if symbol.is_empty() {
                "ε".to_string()
            } else {
                symbol
            }])
        },
    )
}

pub fn write_pushdown_entity(name: &str, pda: &PushdownAutomata) -> Result<String, String> {
    let stack_symbol = pda.get_initial_stack_symbol();
    let extra = if stack_symbol == "Z" {
        None
    } else {
        Some(format!("stack: {}", stack_symbol))
    };
    write_machine_block(
        "pda",
        name,
        "the pushdown automaton has no states",
        "the pushdown automaton has no initial state",
        pda.get_states_by_id_ref(),
        *pda.get_initial_state_id(),
        pda.get_final_states(),
        extra,
        |label| {
            // The state layer stores bare "ε" for no-op transitions; the
            // tuple spelling of the same behavior is (ε, ε, ε).
            if label == "ε" {
                return Ok(vec![
                    "ε".to_string(),
                    "ε".to_string(),
                    "ε".to_string(),
                ]);
            }
            let malformed =
                || format!("malformed pushdown label '{}' (expected 'input;pop/push')", label);
            let (input, rest) = label.split_once(';').ok_or_else(malformed)?;
            let (pop, push) = rest.split_once('/').ok_or_else(malformed)?;
            let input = check_tuple_component(input, "pushdown input symbol", false)?;
            let pop = check_tuple_component(pop, "pushdown popped symbol", false)?;
            let push = check_tuple_component(push, "pushdown pushed symbols", true)?;
            // Blank components are the loader's ε (no-op) spelling.
            let to_epsilon = |component: String| {
                if component.is_empty() {
                    "ε".to_string()
                } else {
                    component
                }
            };
            Ok(vec![to_epsilon(input), to_epsilon(pop), to_epsilon(push)])
        },
    )
}

pub fn write_turing_entity(name: &str, tm: &TuringMachine) -> Result<String, String> {
    let blank = tm.get_blank_symbol();
    let extra = if blank == '_' {
        None
    } else {
        Some(format!("blank: {}", blank))
    };
    write_machine_block(
        "tm",
        name,
        "the turing machine has no states",
        "the turing machine has no initial state",
        tm.get_states_by_id_ref(),
        *tm.get_initial_state_id(),
        tm.get_final_states(),
        extra,
        |label| {
            // The label is tape groups joined by commas, each 'read;write/dir'.
            let mut parts = Vec::new();
            for group in label.split(',') {
                let malformed = || {
                    format!(
                        "malformed turing tape group '{}' in label '{}' (expected 'read;write/dir')",
                        group, label
                    )
                };
                let (read, rest) = group.split_once(';').ok_or_else(malformed)?;
                let (write, dir) = rest.split_once('/').ok_or_else(malformed)?;
                let read = check_tuple_component(read, "turing read symbol", false)?;
                let write = check_tuple_component(write, "turing write symbol", false)?;
                if read.is_empty() || write.is_empty() {
                    return Err(format!(
                        "turing read/write symbols cannot be empty in label '{}' (use the blank symbol)",
                        label
                    ));
                }
                let dir = dir.trim().to_ascii_uppercase();
                if !matches!(dir.as_str(), "L" | "R" | "S") {
                    return Err(format!(
                        "invalid direction '{}' in label '{}' (use L, R or S)",
                        dir, label
                    ));
                }
                parts.push(read);
                parts.push(write);
                parts.push(dir);
            }
            Ok(parts)
        },
    )
}

pub fn write_grammar_entity(name: &str, grammar: &Grammar) -> Result<String, String> {
    let productions = grammar.productions();
    if productions.is_empty() {
        return Err("the grammar has no productions".to_string());
    }
    let mut text = entity_header("grammar", name);
    // The .ce format derives the start symbol from the first production
    // line, so the start variable's alternatives are emitted first; the
    // remaining variables follow in `productions()`'s sorted order (the
    // same order Display uses, so the text also matches Display output).
    let start = grammar.start_symbol().to_string();
    let (start_pairs, other_pairs): (Vec<_>, Vec<_>) =
        productions.into_iter().partition(|(variable, _)| *variable == start);
    // `productions()` yields (variable, body) pairs grouped by variable;
    // fold consecutive pairs back into one line per variable.
    let mut current: Option<(String, Vec<String>)> = None;
    for (variable, body) in start_pairs.into_iter().chain(other_pairs) {
        let alternative = if body.is_empty() {
            "ε".to_string()
        } else {
            body.join(" ")
        };
        match &mut current {
            Some((var, alternatives)) if *var == variable => {
                alternatives.push(alternative);
            }
            _ => {
                if let Some((var, alternatives)) = current.take() {
                    text.push_str(&format!(
                        "\nproductions: {} -> {}",
                        var,
                        alternatives.join(" | ")
                    ));
                }
                current = Some((variable, vec![alternative]));
            }
        }
    }
    if let Some((var, alternatives)) = current {
        text.push_str(&format!(
            "\nproductions: {} -> {}",
            var,
            alternatives.join(" | ")
        ));
    }
    Ok(text)
}