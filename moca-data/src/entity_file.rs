use crate::finite_automata::FiniteAutomata;
use crate::grammar::{parse_grammar, Grammar};
use crate::pushdown_automata::PushdownAutomata;
use crate::regex;
use crate::state::{State, StateID};
use crate::state_machine::{Machine, StateMachine};
use crate::turing_machine::TuringMachine;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub enum Entity {
    Finite(FiniteAutomata),
    Pushdown(PushdownAutomata),
    Turing(TuringMachine),
    Grammar(Grammar),
}

#[derive(Debug)]
pub struct NamedEntity {
    pub name: String,
    pub entity: Entity,
}

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

fn strip_entity_header(line: &str) -> Option<&str> {
    let (key, value) = line.split_once(':')?;
    if key.trim().eq_ignore_ascii_case("entity") {
        Some(value.trim())
    } else {
        None
    }
}

type BlockError = (String, usize, String);

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
                depth = depth.saturating_sub(1);
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

fn parse_transition_tuple(tuple: &str) -> Result<(String, Vec<String>, String), String> {
    let trimmed = tuple.trim();
    if !trimmed.starts_with('(') {
        return Err(format!(
            "expected a '(state, ...) -> state' tuple, found '{trimmed}'"
        ));
    }
    let close = trimmed
        .rfind(')')
        .ok_or_else(|| format!("missing ')' in '{trimmed}'"))?;
    let inner = trimmed[1..close].trim();
    let rest = trimmed[close + 1..].trim();
    let target = rest
        .strip_prefix("->")
        .ok_or_else(|| format!("missing '->' in '{trimmed}'"))?
        .trim();
    if target.is_empty() {
        return Err(format!("missing target state in '{trimmed}'"));
    }
    if inner.is_empty() {
        return Err(format!("missing states in '{trimmed}'"));
    }
    let parts: Vec<String> = inner.split(',').map(|part| part.trim().to_string()).collect();
    Ok((parts[0].clone(), parts, target.to_string()))
}

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

fn build_block(
    header_line: usize,
    kind_text: &str,
    entries: &[(usize, String, String)],
) -> Result<NamedEntity, BlockError> {
    let (kind, fallback_name) =
        resolve_kind(&kind_text.to_ascii_lowercase()).ok_or_else(|| {
            (
                format!("entity at line {header_line}"),
                header_line,
                format!("unknown entity kind '{kind_text}'"),
            )
        })?;

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
                    format!("'states:' is not valid for {fallback_name} entities"),
                ));
            }
            states.extend(split_csv(value));
        } else if key == "transitions" {
            if matches!(kind, Kind::Regex | Kind::Grammar) {
                return Err(err(
                    *line,
                    format!(
                        "'transitions:' is not valid for {fallback_name} entities"
                    ),
                ));
            }
            transitions.push((*line, value.clone()));
        } else if key == "initial" {
            if matches!(kind, Kind::Regex | Kind::Grammar) {
                return Err(err(
                    *line,
                    format!(
                        "'initial:' is not valid for {fallback_name} entities"
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
                    format!("'{key}:' is not valid for {fallback_name} entities"),
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
            return Err(err(*line, format!("unknown key '{key}:'")));
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
                .map_err(|error| err(line, format!("invalid regex: {error}")))?;
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
        entity,
    })
}

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
                    if parts.len() < 4 || !(parts.len() - 1).is_multiple_of(3) {
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
                                    "inconsistent tape count: this transition uses {tapes}, earlier ones use {existing}"
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
                        pieces.push(format!("{read};{write}/{direction}"));
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

    let populate = |machine: &mut dyn StateMachine| {
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
    };
    let entity = match kind {
        Kind::Finite => {
            let mut machine = FiniteAutomata::new();
            populate(&mut machine);
            Machine::validate(&machine).map_err(report_validation)?;
            Entity::Finite(machine)
        }
        Kind::Pushdown => {
            let mut machine = PushdownAutomata::new(stack_symbol);
            populate(&mut machine);
            Machine::validate(&machine).map_err(report_validation)?;
            Entity::Pushdown(machine)
        }
        Kind::Turing => {
            let mut machine = TuringMachine::new(blank_symbol);
            populate(&mut machine);
            Machine::validate(&machine).map_err(report_validation)?;
            Entity::Turing(machine)
        }
        Kind::Regex | Kind::Grammar => unreachable!(),
    };
    Ok(entity)
}

fn entity_header(kind_alias: &str, name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .collect();
    let trimmed = sanitized.trim();
    let name = if trimmed.is_empty() { "entity" } else { trimmed };
    format!("entity: {kind_alias}\nname: {name}")
}

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
                "{role} '{trimmed}' uses '{c}' which the .ce format reserves for its structure"
            ));
        }
    }
    Ok(trimmed.to_string())
}

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
            return Err(format!("state {id} has an empty name"));
        }
        if !seen.insert(label.clone()) {
            return Err(format!("state name '{label}' is used more than once"));
        }
        labels.push(label);
    }
    let name_of: HashMap<StateID, &String> =
        ids.iter().copied().zip(labels.iter()).collect();
    let initial_id = initial.ok_or_else(|| no_initial_message.to_string())?;
    if !states.contains_key(&initial_id) {
        return Err(format!("initial state {initial_id} is missing"));
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
            format!("transition leaves state {from_id}, which is missing")
        })?;
        let to_label = name_of.get(&to_id).ok_or_else(|| {
            format!("transition targets state {to_id}, which is missing")
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
        Some(format!("stack: {stack_symbol}"))
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
            if label == "ε" {
                return Ok(vec![
                    "ε".to_string(),
                    "ε".to_string(),
                    "ε".to_string(),
                ]);
            }
            let malformed =
                || format!("malformed pushdown label '{label}' (expected 'input;pop/push')");
            let (input, rest) = label.split_once(';').ok_or_else(malformed)?;
            let (pop, push) = rest.split_once('/').ok_or_else(malformed)?;
            let input = check_tuple_component(input, "pushdown input symbol", false)?;
            let pop = check_tuple_component(pop, "pushdown popped symbol", false)?;
            let push = check_tuple_component(push, "pushdown pushed symbols", true)?;
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
        Some(format!("blank: {blank}"))
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
            let mut parts = Vec::new();
            for group in label.split(',') {
                let malformed = || {
                    format!(
                        "malformed turing tape group '{group}' in label '{label}' (expected 'read;write/dir')"
                    )
                };
                let (read, rest) = group.split_once(';').ok_or_else(malformed)?;
                let (write, dir) = rest.split_once('/').ok_or_else(malformed)?;
                let read = check_tuple_component(read, "turing read symbol", false)?;
                let write = check_tuple_component(write, "turing write symbol", false)?;
                if read.is_empty() || write.is_empty() {
                    return Err(format!(
                        "turing read/write symbols cannot be empty in label '{label}' (use the blank symbol)"
                    ));
                }
                let dir = dir.trim().to_ascii_uppercase();
                if !matches!(dir.as_str(), "L" | "R" | "S") {
                    return Err(format!(
                        "invalid direction '{dir}' in label '{label}' (use L, R or S)"
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
    if grammar.productions().is_empty() {
        return Err("the grammar has no productions".to_string());
    }
    let mut text = entity_header("grammar", name);
    let start = grammar.start_symbol();
    let others = grammar.nonterminals().iter().filter(|variable| *variable != start);
    for variable in std::iter::once(start).chain(others.map(String::as_str)) {
        let Some(bodies) = grammar.productions_of(variable).filter(|bodies| !bodies.is_empty()) else {
            continue;
        };
        let alternatives: Vec<String> = bodies
            .iter()
            .map(|body| if body.is_empty() { "ε".to_string() } else { body.join(" ") })
            .collect();
        text.push_str(&format!("\nproductions: {} -> {}", variable, alternatives.join(" | ")));
    }
    Ok(text)
}
