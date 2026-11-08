use crate::finite_automata::FiniteAutomata;
use crate::grammar::{parse_grammar, Grammar};
use crate::pushdown_automata::{PdaOp, PushdownAutomata, DEFAULT_STACK};
use crate::regex;
use crate::state::{is_epsilon, StateID, EPSILON};
use crate::state_machine::{Machine, StateMachine};
use crate::turing_machine::{parse_transition, TransitionOp, TuringMachine, DEFAULT_BLANK};
use std::collections::{HashMap, HashSet};

pub const LLM_PROMPT: &str = r#"You are given an image of one or more computational models (a state diagram of an automaton, a Turing machine, a pushdown automaton, a regular expression, or a context-free grammar).

Transcribe what you see into a `.ce` file with the exact syntax below, then output ONLY the file contents (no explanations, no code fences).

SYNTAX
- One entity per block, each block starts with `entity: <kind>` where kind is one of:
  dfa | nfa | fa | automata | automaton | finite | tm | turing | pda | pushdown | regex | re | grammar | cfg
- Optional `name: <label>` line right after `entity:`.
- Lines starting with `#` are comments. Blank lines are ignored.
- State lists and finals are comma-separated. Multiple `transitions:` / `productions:` lines accumulate.
- State names and transition symbols cannot contain `(`, `)` or `,`: the file format reserves them.

TRANSITION SYNTAX (different per family - use the right one!)
- dfa/nfa/fa/automata/finite:  (from, symbol) -> to
    symbols may be single characters; use ε for epsilon moves.
- pda/pushdown:  (from, input, pop, push...) -> to
    input read, stack symbol popped, then the stack symbols pushed; when several are pushed, the FIRST listed one ends on top. Use ε for no-ops.
    Example: (q0, a, Z, A, Z) -> q1   (read a, pop Z, push Z back and A on top of it)
- tm/turing:  (from, read, write, dir[, read, write, dir...]) -> to
    one (read, write, dir) group per tape; dir is L, R or S.
    Example single tape: (q0, 0, 1, R) -> q1
    Example two tapes:   (q0, 0, 1, R, _, _, S) -> q1
- regex: a line `regex: <pattern>` with operators | * + ? and ε.
- grammar: `productions: <lhs> -> body | body` lines; ε is an empty body.

COMMON KEYS
- `states: a, b, c` (machines only, optional: states named in transitions are added automatically)
- `initial: <state>` (machines only)
- `final: <state>, <state>` also accepts `finals:` / `halt:` / `final/halt:` (machines only)
- `blank: <char>` (turing only, default _)
- `stack: <symbol>` (pushdown only, default Z)

EXAMPLE FILE
# A pushdown automaton, a regex and a grammar, together in one file
entity: pda
name: anbn
states: q0, q1, q2
transitions: (q0, a, Z, A, Z) -> q0, (q0, a, A, A, A) -> q0, (q0, b, A, ε) -> q1, (q1, b, A, ε) -> q1, (q1, ε, Z, Z) -> q2
initial: q0
final: q2

entity: regex
name: abb
regex: (a|b)*abb

entity: grammar
name: anbn
productions: S -> a S b | ε

TASK
Look at the attached image carefully (states, arrows, labels, initial arrow, double circles for accepting states, stack/tape annotations) and produce the matching `.ce` file. Use short state names exactly as drawn or numbered q0, q1, ... if unlabeled. Output only the .ce file contents."#;

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
enum Family {
    Finite,
    Pushdown,
    Turing,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Machine(Family),
    Regex,
    Grammar,
}

fn resolve_kind(alias: &str) -> Option<(Kind, &'static str)> {
    match alias {
        "tm" | "turing" => Some((Kind::Machine(Family::Turing), "TM")),
        "pda" | "pushdown" => Some((Kind::Machine(Family::Pushdown), "Pushdown")),
        "dfa" => Some((Kind::Machine(Family::Finite), "DFA")),
        "nfa" => Some((Kind::Machine(Family::Finite), "NFA")),
        "fa" | "automata" | "automaton" | "finite" => Some((Kind::Machine(Family::Finite), "FA")),
        "regex" | "re" => Some((Kind::Regex, "Regex")),
        "grammar" | "cfg" => Some((Kind::Grammar, "Grammar")),
        _ => None,
    }
}

fn canonical_key(key: &str) -> Option<&'static str> {
    match key {
        "name" => Some("name"),
        "states" => Some("states"),
        "transitions" => Some("transitions"),
        "initial" => Some("initial"),
        "final" | "finals" | "halt" | "final/halt" | "final_halt" => Some("final"),
        "blank" => Some("blank"),
        "stack" => Some("stack"),
        "regex" => Some("regex"),
        "productions" => Some("productions"),
        _ => None,
    }
}

fn allows(kind: Kind, key: &str) -> bool {
    match key {
        "states" | "transitions" | "initial" | "final" => matches!(kind, Kind::Machine(_)),
        "blank" => kind == Kind::Machine(Family::Turing),
        "stack" => kind == Kind::Machine(Family::Pushdown),
        "regex" => kind == Kind::Regex,
        "productions" => kind == Kind::Grammar,
        _ => true,
    }
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
    ids: HashMap<String, StateID>,
    next_id: StateID,
}

impl Interner {
    fn intern(&mut self, label: &str) -> Option<StateID> {
        let trimmed = label.trim();
        if trimmed.is_empty() {
            return None;
        }
        let next_id = &mut self.next_id;
        Some(*self.ids.entry(trimmed.to_string()).or_insert_with(|| {
            let id = *next_id;
            *next_id += 1;
            id
        }))
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
    if inner.contains(['(', ')']) || target.contains(['(', ')']) {
        return Err(format!("'(' and ')' are reserved by the .ce format, found in '{trimmed}'"));
    }
    let mut parts = split_csv(inner);
    let from = parts.remove(0);
    Ok((from, parts, target.to_string()))
}

fn normalize_epsilon(component: &str) -> &str {
    if component.is_empty() {
        EPSILON
    } else {
        component
    }
}

fn split_csv(value: &str) -> Vec<String> {
    value.split(',').map(|part| part.trim().to_string()).collect()
}

fn label_from_parts(family: Family, parts: &[String]) -> Result<String, String> {
    match family {
        Family::Finite => match parts {
            [symbol] => Ok(normalize_epsilon(symbol).to_string()),
            _ => Err(format!(
                "finite transitions take exactly 2 parts (from, symbol), found {} part(s)",
                parts.len() + 1
            )),
        },
        Family::Pushdown => {
            let (read, pop, push) = match parts {
                [read, pop, push @ ..] if !push.is_empty() => (read, pop, push),
                _ => {
                    return Err(format!(
                        "pushdown transitions take at least 4 parts (from, input, pop, push), found {} part(s)",
                        parts.len() + 1
                    ))
                },
            };
            let push: Vec<&str> = push.iter().map(|part| normalize_epsilon(part)).collect();
            Ok(format!("{};{}/{}", normalize_epsilon(read), normalize_epsilon(pop), push.join(",")))
        }
        Family::Turing => {
            if parts.is_empty() || !parts.len().is_multiple_of(3) {
                return Err(format!(
                    "turing transitions take (from, read, write, dir) groups of 4, 7, 10... parts, found {} part(s)",
                    parts.len() + 1
                ));
            }
            let ops = parts
                .chunks(3)
                .map(|group| {
                    let (read, write, direction) = (&group[0], &group[1], &group[2]);
                    TransitionOp::from_fields(read, write, direction)
                        .map_err(|problem| format!("turing group ({read}, {write}, {direction}): {problem}"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(crate::turing_machine::format_transition(&ops))
        }
    }
}

fn single_value(entry: &Option<(usize, String)>, header_line: usize, default: &str) -> (usize, String) {
    entry
        .as_ref()
        .map(|(line, value)| (*line, value.trim().to_string()))
        .unwrap_or((header_line, default.to_string()))
}

#[derive(Default)]
struct BlockFields {
    name: Option<String>,
    states: Vec<String>,
    transitions: Vec<(usize, String)>,
    initial: Option<(usize, String)>,
    finals: Vec<String>,
    blank: Option<(usize, String)>,
    stack: Option<(usize, String)>,
    regex: Option<(usize, String)>,
    productions: Vec<(usize, String)>,
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
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| fallback_name.to_string());
    let err = |line: usize, message: String| -> BlockError { (declared_name.clone(), line, message) };

    let mut fields = BlockFields::default();
    for (line, key, value) in entries {
        if key.is_empty() {
            return Err(err(*line, "expected 'key: value' lines inside an entity block".to_string()));
        }
        let canonical = canonical_key(key).ok_or_else(|| err(*line, format!("unknown key '{key}:'")))?;
        if !allows(kind, canonical) {
            return Err(err(*line, format!("'{key}:' is not valid for {fallback_name} entities")));
        }
        let first = |slot: &mut Option<(usize, String)>| {
            slot.get_or_insert_with(|| (*line, value.clone()));
        };
        match canonical {
            "name" => {
                if fields.name.is_none() && !value.is_empty() {
                    fields.name = Some(value.clone());
                }
            },
            "states" => fields.states.extend(split_csv(value)),
            "transitions" => fields.transitions.push((*line, value.clone())),
            "initial" => first(&mut fields.initial),
            "final" => fields.finals.extend(split_csv(value)),
            "blank" => first(&mut fields.blank),
            "stack" => first(&mut fields.stack),
            "regex" => first(&mut fields.regex),
            _ => fields.productions.push((*line, value.clone())),
        }
    }

    let entity = match kind {
        Kind::Regex => {
            let (line, pattern) = fields
                .regex
                .clone()
                .ok_or_else(|| err(header_line, "missing 'regex:'".to_string()))?;
            if pattern.is_empty() {
                return Err(err(line, "empty 'regex:' value".to_string()));
            }
            let automaton = regex::compile_str(&pattern)
                .map_err(|error| err(line, format!("invalid regex: {error}")))?;
            Entity::Finite(automaton)
        }
        Kind::Grammar => {
            let Some(&(first_line, _)) = fields.productions.first() else {
                return Err(err(header_line, "missing 'productions:'".to_string()));
            };
            let source = fields
                .productions
                .iter()
                .map(|(_, value)| value.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            let grammar = parse_grammar(&source).map_err(|error| {
                let line = error
                    .line
                    .checked_sub(1)
                    .and_then(|index| fields.productions.get(index))
                    .map_or(first_line, |(line, _)| *line);
                err(line, format!("invalid grammar: {}", error.message))
            })?;
            Entity::Grammar(grammar)
        }
        Kind::Machine(Family::Finite) => {
            Entity::Finite(build_machine(FiniteAutomata::new(), Family::Finite, header_line, &fields, &err)?)
        }
        Kind::Machine(Family::Pushdown) => {
            let (line, symbol) = single_value(&fields.stack, header_line, DEFAULT_STACK);
            if symbol.is_empty() {
                return Err(err(line, "empty 'stack:' value".to_string()));
            }
            let machine = PushdownAutomata::new(symbol);
            Entity::Pushdown(build_machine(machine, Family::Pushdown, header_line, &fields, &err)?)
        }
        Kind::Machine(Family::Turing) => {
            let (line, blank) = single_value(&fields.blank, header_line, &DEFAULT_BLANK.to_string());
            let mut chars = blank.chars();
            let (Some(blank), None) = (chars.next(), chars.next()) else {
                return Err(err(line, "'blank:' must be a single character".to_string()));
            };
            Entity::Turing(build_machine(TuringMachine::new(blank), Family::Turing, header_line, &fields, &err)?)
        }
    };

    Ok(NamedEntity {
        name: fields.name.unwrap_or_else(|| fallback_name.to_string()),
        entity,
    })
}

fn build_machine<M: StateMachine + Machine>(
    mut machine: M,
    family: Family,
    header_line: usize,
    fields: &BlockFields,
    err: &dyn Fn(usize, String) -> BlockError,
) -> Result<M, BlockError> {
    let mut interner = Interner::default();
    for label in &fields.states {
        interner
            .intern(label)
            .ok_or_else(|| err(header_line, "empty state name in 'states:'".to_string()))?;
    }

    let mut assembled: Vec<(usize, StateID, StateID, String)> = Vec::new();
    for (line, raw) in &fields.transitions {
        for tuple in split_tuples(raw) {
            let (from, parts, to) =
                parse_transition_tuple(&tuple).map_err(|message| err(*line, message))?;
            let from_id = interner
                .intern(&from)
                .ok_or_else(|| err(*line, "empty state name in transition".to_string()))?;
            let to_id = interner
                .intern(&to)
                .ok_or_else(|| err(*line, "empty state name in transition".to_string()))?;
            let label = label_from_parts(family, &parts).map_err(|message| err(*line, message))?;
            assembled.push((*line, from_id, to_id, label));
        }
    }

    let (initial_line, initial_value) = fields
        .initial
        .as_ref()
        .ok_or_else(|| err(header_line, "missing 'initial:'".to_string()))?;
    let initial_id = interner
        .intern(initial_value)
        .filter(|_| !initial_value.contains(','))
        .ok_or_else(|| err(*initial_line, "'initial:' must be a single state name".to_string()))?;
    let final_ids: Vec<StateID> = fields.finals.iter().filter_map(|label| interner.intern(label)).collect();

    for (label, &id) in &interner.ids {
        machine.add_state_with_id_label(id, label);
    }
    for (_, from, to, label) in &assembled {
        machine.add_transition(*from, *to, label.clone());
    }
    for (line, _, _, label) in &assembled {
        machine.check_label(label).map_err(|problem| err(*line, problem))?;
    }
    machine.make_initial(initial_id);
    for id in final_ids {
        machine.make_final(id);
    }
    Ok(machine)
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

fn has_reserved_char(text: &str) -> Option<char> {
    text.chars().find(|c| matches!(c, ',' | '(' | ')' | '\n' | '\r'))
}

fn storable_parts(family: Family, label: &str) -> Result<Vec<String>, String> {
    let parts = label_parts(family, label)?;
    let unstorable = |reason: String| format!("transition label '{label}' cannot be saved: {reason}");
    if let Some(c) = parts.iter().find_map(|part| has_reserved_char(part).filter(|c| *c != ',')) {
        return Err(unstorable(format!("'{c}' is reserved by the .ce format")));
    }
    let reread = split_csv(&parts.join(", "));
    let rebuilt = label_from_parts(family, &reread).map_err(unstorable)?;
    let same_meaning = match family {
        Family::Finite => rebuilt == label || (is_epsilon(label) && is_epsilon(&rebuilt)),
        Family::Pushdown => PdaOp::parse(label).ok() == PdaOp::parse(&rebuilt).ok(),
        Family::Turing => parse_transition(label) == parse_transition(&rebuilt),
    };
    if !same_meaning {
        return Err(unstorable(format!("it would load back as '{rebuilt}'")));
    }
    Ok(parts)
}

fn label_parts(family: Family, label: &str) -> Result<Vec<String>, String> {
    let malformed = |shape: &str| format!("malformed transition label '{label}' (expected {shape})");
    match family {
        Family::Finite => Ok(vec![normalize_epsilon(label).to_string()]),
        Family::Pushdown => {
            PdaOp::parse(label).map_err(|_| malformed("'input;pop/push'"))?;
            if label == EPSILON {
                return Ok(vec![EPSILON.to_string(); 3]);
            }
            let (read, stack) = label.split_once(';').ok_or_else(|| malformed("'input;pop/push'"))?;
            let (pop, push) = stack.split_once('/').ok_or_else(|| malformed("'input;pop/push'"))?;
            Ok([read, pop]
                .into_iter()
                .chain(push.split(','))
                .map(|part| normalize_epsilon(part).to_string())
                .collect())
        }
        Family::Turing => {
            let shape = "'read;write/dir' per tape, joined by commas";
            let ops = parse_transition(label).ok_or_else(|| malformed(shape))?;
            Ok(ops
                .iter()
                .flat_map(|op| [op.read.to_string(), op.write.to_string(), op.direction.letter().to_string()])
                .collect())
        }
    }
}

fn write_machine_block<M: StateMachine>(
    family: Family,
    kind_alias: &str,
    noun: &str,
    name: &str,
    machine: &M,
    extra_header_key: Option<String>,
) -> Result<String, String> {
    let states = machine.get_states_by_id_ref();
    if states.is_empty() {
        return Err(format!("the {noun} has no states"));
    }
    let mut ids: Vec<StateID> = states.keys().copied().collect();
    ids.sort_unstable();
    let mut name_of: HashMap<StateID, String> = HashMap::new();
    let mut seen: HashSet<String> = HashSet::new();
    for id in &ids {
        let label = states[id].name.trim().to_string();
        if let Some(c) = has_reserved_char(&label) {
            return Err(format!("state name '{label}' uses '{c}' which the .ce format reserves for its structure"));
        }
        if label.is_empty() {
            return Err(format!("state {id} has an empty name"));
        }
        if !seen.insert(label.clone()) {
            return Err(format!("state name '{label}' is used more than once"));
        }
        name_of.insert(*id, label);
    }
    let initial_id = machine.get_initial_state_id().ok_or_else(|| format!("the {noun} has no initial state"))?;

    let mut text = entity_header(kind_alias, name);
    text.push_str("\nstates: ");
    text.push_str(&ids.iter().map(|id| name_of[id].as_str()).collect::<Vec<_>>().join(", "));

    let mut triples: Vec<(StateID, StateID, &str)> = machine.graph().labels().collect();
    triples.sort();
    for (from_id, to_id, label) in triples {
        let parts = storable_parts(family, label)?;
        text.push_str(&format!(
            "\ntransitions: ({}, {}) -> {}",
            name_of[&from_id],
            parts.join(", "),
            name_of[&to_id]
        ));
    }
    if let Some(extra) = extra_header_key {
        text.push('\n');
        text.push_str(&extra);
    }
    text.push_str(&format!("\ninitial: {}", name_of[&initial_id]));
    let mut final_ids: Vec<StateID> = machine.get_final_states().iter().copied().collect();
    final_ids.sort_unstable();
    if !final_ids.is_empty() {
        let final_labels: Vec<&str> = final_ids.iter().map(|id| name_of[id].as_str()).collect();
        text.push_str(&format!("\nfinal: {}", final_labels.join(", ")));
    }
    Ok(text)
}

pub fn write_finite_entity(name: &str, fa: &FiniteAutomata) -> Result<String, String> {
    let alias = if fa.is_deterministic() { "dfa" } else { "nfa" };
    write_machine_block(Family::Finite, alias, "finite automaton", name, fa, None)
}

pub fn write_pushdown_entity(name: &str, pda: &PushdownAutomata) -> Result<String, String> {
    let stack_symbol = pda.get_initial_stack_symbol();
    let extra = (stack_symbol != DEFAULT_STACK).then(|| format!("stack: {stack_symbol}"));
    write_machine_block(Family::Pushdown, "pda", "pushdown automaton", name, pda, extra)
}

pub fn write_turing_entity(name: &str, tm: &TuringMachine) -> Result<String, String> {
    let blank = tm.get_blank_symbol();
    let extra = (blank != DEFAULT_BLANK).then(|| format!("blank: {blank}"));
    write_machine_block(Family::Turing, "tm", "turing machine", name, tm, extra)
}

pub fn write_grammar_entity(name: &str, grammar: &Grammar) -> Result<String, String> {
    if grammar.productions().is_empty() {
        return Err("the grammar has no productions".to_string());
    }
    let mut text = entity_header("grammar", name);
    for line in grammar.to_string().lines() {
        text.push_str("\nproductions: ");
        text.push_str(line);
    }
    Ok(text)
}
