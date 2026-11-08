use super::{finite, turing, words};
use crate::entity_file::{
    parse_entity_file, write_finite_entity, write_grammar_entity, write_pushdown_entity,
    write_turing_entity, Entity, LLM_PROMPT,
};
use crate::finite_automata::FiniteAutomata;
use crate::grammar::Grammar;
use crate::pushdown_automata::{PdaOp, PushdownAutomata};
use crate::state::{State, StateID};
use crate::state_machine::{Machine, StateMachine};
use crate::turing_machine::TuringMachine;
use std::collections::{BTreeSet, HashMap};

fn finite_of(entity: &Entity) -> &FiniteAutomata {
    match entity {
        Entity::Finite(finite) => finite,
        _ => panic!("expected a finite automaton"),
    }
}

fn pda_of(entity: &Entity) -> &PushdownAutomata {
    match entity {
        Entity::Pushdown(pda) => pda,
        _ => panic!("expected a pushdown automaton"),
    }
}

fn tm_of(entity: &Entity) -> &TuringMachine {
    match entity {
        Entity::Turing(turing) => turing,
        _ => panic!("expected a turing machine"),
    }
}

fn grammar_of(entity: &Entity) -> &Grammar {
    match entity {
        Entity::Grammar(grammar) => grammar,
        _ => panic!("expected a grammar"),
    }
}

fn first_error_message(source: &str) -> String {
    let (entities, errors) = parse_entity_file(source);
    assert!(entities.is_empty(), "expected no entities, got {entities:?}");
    assert_eq!(errors.len(), 1, "expected exactly one error: {errors:?}");
    errors[0].message.clone()
}

#[test]
fn entity_file_multi_kind_test() {
    let source = "\
# computational entities demo
entity: dfa
name: parity
states: s0, s1
transitions: (s0, a) -> s1, (s1, a) -> s0
initial: s0
final: s0

entity: tm
states: a, b
transitions: (a, 0, 1, R) -> b
initial: a
final: b

entity: grammar
productions: S -> a S b | ε
";
    let (entities, errors) = parse_entity_file(source);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(entities.len(), 3);
    assert_eq!(entities[0].name, "parity");
    assert!(finite_of(&entities[0].entity).accepts(""));
    assert!(finite_of(&entities[0].entity).accepts("aa"));
    assert!(!finite_of(&entities[0].entity).accepts("a"));
    assert!(tm_of(&entities[1].entity).accepts("0"));
    assert_eq!(entities[1].name, "TM");
    assert!(grammar_of(&entities[2].entity).generate("aabb"));
    assert_eq!(entities[2].name, "Grammar");
}

#[test]
fn entity_file_pda_push_tail_roundtrip_test() {
    let source = "\
entity: pda
states: q0, q1, q2
transitions: (q0, a, Z, A, Z) -> q1
transitions: (q1, b, A, ε) -> q2
initial: q0
final: q2
";
    let (entities, errors) = parse_entity_file(source);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let pda = pda_of(&entities[0].entity);
    assert!(Machine::accepts(pda, "ab"));
    assert!(!Machine::accepts(pda, "ba"));
    assert!(!Machine::accepts(pda, "a"));
}

#[test]
fn entity_file_pda_embedded_label_semantics_test() {
    let source = "\
entity: pushdown
states: q0, q1
transitions: (q0, a, Z, aZ) -> q1, (q0, ε, Z, Z) -> q1
initial: q0
final: q1
";
    let (entities, errors) = parse_entity_file(source);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let pda = pda_of(&entities[0].entity);
    let labels: Vec<String> = pda
        .get_states_by_id_ref()
        .values()
        .flat_map(|state| state.iter_by_transition().flat_map(|(_, inputs)| inputs.iter().cloned()))
        .collect();
    assert!(labels.contains(&"a;Z/aZ".to_string()), "labels: {labels:?}");
    assert!(labels.contains(&"ε;Z/Z".to_string()), "labels: {labels:?}");
}

#[test]
fn entity_file_tm_multitape_test() {
    let source = "\
entity: tm
states: a, b
transitions: (a, 0, 1, R, _, _, S) -> b
initial: a
final: b
";
    let (entities, errors) = parse_entity_file(source);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(tm_of(&entities[0].entity).initial_configuration("").unwrap().tapes().len(), 2);
}

#[test]
fn entity_file_tm_tape_mismatch_test() {
    let message = first_error_message(
        "\
entity: tm
states: a, b
transitions: (a, 0, 1, R) -> b, (a, 0, 1, R, _, _, S) -> b
initial: a
final: b
",
    );
    assert!(message.contains("inconsistent tape count"), "message: {message}");
}

#[test]
fn entity_file_tm_direction_handling_test() {
    let source = "\
entity: turing
states: a, b
transitions: (a, 0, 1, r) -> b
initial: a
final: b
";
    let (entities, errors) = parse_entity_file(source);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert!(tm_of(&entities[0].entity).accepts("0"));

    let message = first_error_message(
        "\
entity: turing
states: a, b
transitions: (a, 0, 1, x) -> b
initial: a
final: b
",
    );
    assert!(message.contains("invalid direction"), "message: {message}");
}

#[test]
fn entity_file_finite_arity_test() {
    let message = first_error_message(
        "\
entity: fa
states: a, b
transitions: (a, 0, 1) -> b
initial: a
final: b
",
    );
    assert!(message.contains("exactly 2 parts"), "message: {message}");
}

#[test]
fn entity_file_auto_registration_test() {
    let source = "\
entity: nfa
states: start
transitions: (start, a) -> mid, (mid, ε) -> end
initial: start
final: end
";
    let (entities, errors) = parse_entity_file(source);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(entities[0].name, "NFA");
    let automaton = finite_of(&entities[0].entity);
    assert_eq!(automaton.get_states_by_id_ref().len(), 3);
    assert!(Machine::accepts(automaton, "a"));
}

#[test]
fn entity_file_regex_entity_test() {
    let source = "\
entity: regex
name: abb detector
regex: (a|b)*abb
";
    let (entities, errors) = parse_entity_file(source);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(entities[0].name, "abb detector");
    let automaton = finite_of(&entities[0].entity);
    assert!(Machine::accepts(automaton, "ababb"));
    assert!(!Machine::accepts(automaton, "abba"));
}

#[test]
fn entity_file_invalid_regex_test() {
    let message = first_error_message(
        "\
entity: regex
regex: (a|b
",
    );
    assert!(message.contains("invalid regex"), "message: {message}");
}

#[test]
fn entity_file_grammar_invalid_test() {
    let message = first_error_message(
        "\
entity: grammar
productions: S -> a
productions: S a
",
    );
    assert!(message.contains("invalid grammar"), "message: {message}");
}

#[test]
fn entity_file_wrong_family_keys_test() {
    let message = first_error_message(
        "\
entity: pda
blank: _
states: q0
transitions: (q0, a, Z, Z) -> q0
initial: q0
",
    );
    assert!(message.contains("'blank:' is not valid for Pushdown entities"), "message: {message}");

    let (entities, errors) = parse_entity_file(
        "\
entity: grammar
states: s0
productions: S -> a
",
    );
    assert!(entities.is_empty());
    assert!(errors[0].message.contains("'states:' is not valid"), "{errors:?}");
}

#[test]
fn entity_file_broken_entity_does_not_sink_file_test() {
    let source = "\
entity: dfa
states: a, b
transitions: (a, a) -> b
initial: a
final: b

entity: dfa
states: a, b
transitions: (a, a, b) -> b
initial: a
final: b

entity: regex
regex: a*
";
    let (entities, errors) = parse_entity_file(source);
    assert_eq!(entities.len(), 2, "expected the two healthy entities");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].entity_name, "DFA");
    assert!(finite_of(&entities[0].entity).accepts("a"));
    assert!(!finite_of(&entities[0].entity).accepts("aa"));
    assert!(finite_of(&entities[1].entity).accepts("aaaa"));
}

#[test]
fn entity_file_comments_and_stray_content_test() {
    let source = "\
# leading comment
states: a
entity: fa
states: a, b
transitions: (a, a) -> b
initial: a
final: b
";
    let (entities, errors) = parse_entity_file(source);
    assert_eq!(entities.len(), 1);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].entity_name.is_empty());
    assert!(errors[0].message.contains("before the first 'entity:'"));
}

#[test]
fn entity_file_missing_initial_test() {
    let message = first_error_message(
        "\
entity: fa
states: a, b
transitions: (a, a) -> b
final: b
",
    );
    assert!(message.contains("missing 'initial:'"), "message: {message}");
}

#[test]
fn entity_file_unknown_kind_test() {
    let (entities, errors) = parse_entity_file(
        "\
entity: blender
states: a
",
    );
    assert!(entities.is_empty());
    assert!(errors[0].message.contains("unknown entity kind"), "{errors:?}");
}

#[test]
fn entity_file_blank_and_stack_defaults_test() {
    let source = "\
entity: tm
states: a, b
transitions: (a, _, _, R) -> b
initial: a
final: b

entity: pda
states: q0, q1
transitions: (q0, a, Z, Z) -> q1
initial: q0
final: q1
";
    let (entities, errors) = parse_entity_file(source);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert!(tm_of(&entities[0].entity).accepts(""));
    assert!(Machine::accepts(pda_of(&entities[1].entity), "a"));
}

fn machine_triples(states: &HashMap<StateID, State>) -> BTreeSet<(String, String, String)> {
    let mut triples = BTreeSet::new();
    for (from_id, state) in states {
        for (to_id, labels) in state.iter_by_transition() {
            for label in labels {
                triples.insert((
                    states[from_id].name.clone(),
                    states[to_id].name.clone(),
                    label.clone(),
                ));
            }
        }
    }
    triples
}

#[test]
fn entity_file_save_load_finite_roundtrip_test() {
    let mut fa = FiniteAutomata::new();
    fa.add_state_with_id_label(0, "q0");
    fa.add_state_with_id_label(1, "q1");
    fa.add_state_with_id_label(2, "q2");
    fa.add_transition(0, 1, "a".to_string());
    fa.add_transition(1, 2, "ε".to_string());
    fa.add_transition(2, 0, "b".to_string());
    fa.make_initial(0);
    fa.make_final(2);

    let text = write_finite_entity("parity", &fa).expect("serializable");
    let (entities, errors) = parse_entity_file(&text);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].name, "parity");
    let loaded = finite_of(&entities[0].entity);
    assert_eq!(
        machine_triples(loaded.get_states_by_id_ref()),
        machine_triples(fa.get_states_by_id_ref())
    );
    assert_eq!(loaded.get_initial_state_id(), fa.get_initial_state_id());
    assert_eq!(loaded.get_final_states(), fa.get_final_states());
    assert_eq!(loaded.is_deterministic(), fa.is_deterministic());
    assert!(Machine::accepts(loaded, "a"));
    assert!(Machine::accepts(loaded, "aba"));
    assert!(!Machine::accepts(loaded, "ab"));
}

#[test]
fn entity_file_save_load_pda_roundtrip_test() {
    let mut pda = PushdownAutomata::new("P0".to_string());
    pda.add_state_with_id_label(0, "q0");
    pda.add_state_with_id_label(1, "q1");
    pda.add_state_with_id_label(2, "q2");
    pda.add_transition(0, 0, "ε;P0/A".to_string());
    pda.add_transition(0, 1, "c;A/X,Y".to_string());
    pda.add_transition(1, 1, "a;X/ε".to_string());
    pda.add_transition(1, 2, "b;Y/ε".to_string());
    pda.make_initial(0);
    pda.make_final(2);

    let text = write_pushdown_entity("bal", &pda).expect("serializable");
    let (entities, errors) = parse_entity_file(&text);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(entities.len(), 1);
    let loaded = pda_of(&entities[0].entity);
    assert_eq!(loaded.get_initial_stack_symbol(), "P0");
    assert_eq!(
        machine_triples(loaded.get_states_by_id_ref()),
        machine_triples(pda.get_states_by_id_ref())
    );
    assert!(Machine::accepts(loaded, "cab"));
    assert!(!Machine::accepts(loaded, "caab"));
    assert!(!Machine::accepts(loaded, "ab"));
}

#[test]
fn entity_file_save_load_tm_roundtrip_test() {
    let mut tm = TuringMachine::new('□');
    tm.add_state_with_id_label(0, "s0");
    tm.add_state_with_id_label(1, "s1");
    tm.add_transition(0, 1, "a;X/R,_;*/L".to_string());
    tm.add_transition(1, 0, "b;_/S,a;X/L".to_string());
    tm.make_initial(0);
    tm.make_final(1);

    let text = write_turing_entity("two-tape", &tm).expect("serializable");
    let (entities, errors) = parse_entity_file(&text);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(entities.len(), 1);
    let loaded = tm_of(&entities[0].entity);
    assert_eq!(loaded.get_blank_symbol(), '□');
    assert_eq!(loaded.initial_configuration("").unwrap().tapes().len(), 2);
    assert_eq!(
        machine_triples(loaded.get_states_by_id_ref()),
        machine_triples(tm.get_states_by_id_ref())
    );
}

#[test]
fn entity_file_save_load_grammar_roundtrip_test() {
    let grammar = crate::grammar::parse_grammar("S -> a S b | ε\nS -> b\nA -> c").expect("parses");
    let text = write_grammar_entity("demo", &grammar).expect("serializable");
    let (entities, errors) = parse_entity_file(&text);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(entities.len(), 1);
    let loaded = grammar_of(&entities[0].entity);
    assert_eq!(format!("{loaded}"), format!("{}", grammar));
    assert!(loaded.generate("aabb"));
    assert!(loaded.generate("b"));
    assert!(!loaded.generate("ba"));
}

#[test]
fn entity_file_save_empty_entity_test() {
    assert!(write_finite_entity("e", &FiniteAutomata::default()).is_err());
    assert!(write_pushdown_entity("e", &PushdownAutomata::new("Z".to_string())).is_err());
    assert!(write_turing_entity("e", &TuringMachine::new('_')).is_err());
    assert!(write_grammar_entity("e", &Grammar::default()).is_err());
}

#[test]
fn entity_file_save_unrepresentable_label_test() {
    let mut fa = FiniteAutomata::default();
    fa.add_state_with_id_label(0, "q0");
    fa.add_state_with_id_label(1, "q1");
    fa.make_initial(0);
    fa.add_transition(0, 1, "a,b".to_string());
    assert!(write_finite_entity("e", &fa).is_err());

    let mut pda = PushdownAutomata::new("Z".to_string());
    pda.add_state_with_id_label(0, "q0");
    pda.make_initial(0);
    pda.add_transition(0, 0, "a;Z".to_string());
    assert!(write_pushdown_entity("e", &pda).is_err());

    let mut tm = TuringMachine::new('_');
    tm.add_state_with_id_label(0, "q0");
    tm.make_initial(0);
    tm.add_transition(0, 0, "a;X/Q".to_string());
    assert!(write_turing_entity("e", &tm).is_err());
    tm.add_transition(0, 0, "a;X".to_string());
    assert!(write_turing_entity("e", &tm).is_err());
}

#[test]
fn entity_file_save_missing_initial_test() {
    let mut fa = FiniteAutomata::default();
    fa.add_state_with_id_label(0, "q0");
    assert!(write_finite_entity("e", &fa).is_err());
}

#[test]
fn demo_file_entities_match_their_names_test() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/demo.ce");
    let source = std::fs::read_to_string(path).expect("examples/demo.ce is readable");
    let (entities, errors) = parse_entity_file(&source);
    assert!(errors.is_empty(), "demo.ce parses cleanly: {errors:?}");

    fn is_anbn(word: &str) -> bool {
        let half = word.len() / 2;
        word.len() % 2 == 0 && word[..half].chars().all(|c| c == 'a') && word[half..].chars().all(|c| c == 'b')
    }
    type Spec = (&'static str, &'static [char], fn(&str) -> bool);
    let specs: [Spec; 6] = [
        ("even-number-of-as", &['a', 'b'], |w| w.matches('a').count() % 2 == 0),
        ("ends-with-b", &['a', 'b'], |w| w.ends_with('b')),
        ("anbn", &['a', 'b'], is_anbn),
        ("contains-a-zero", &['0', '1'], |w| w.contains('0')),
        ("abb", &['a', 'b'], |w| w.ends_with("abb")),
        ("anbn", &['a', 'b'], is_anbn),
    ];
    assert_eq!(entities.len(), specs.len());
    for (named, (name, alphabet, expected)) in entities.iter().zip(specs) {
        assert_eq!(named.name, name);
        for word in words(alphabet, 6) {
            let accepted = match &named.entity {
                Entity::Finite(machine) => machine.accepts(&word),
                Entity::Pushdown(machine) => machine.accepts(&word),
                Entity::Turing(machine) => machine.accepts(&word),
                Entity::Grammar(grammar) => grammar.generate(&word),
            };
            assert_eq!(accepted, expected(&word), "{name} on {word:?}");
        }
    }
}

#[test]
fn llm_prompt_example_parses_test() {
    let example = LLM_PROMPT
        .split_once("EXAMPLE FILE\n")
        .and_then(|(_, rest)| rest.split_once("\n\nTASK"))
        .map(|(example, _)| example)
        .expect("the prompt carries an example block");
    let (entities, errors) = parse_entity_file(example);
    assert!(errors.is_empty(), "the prompt's example must load: {errors:?}");
    let names: Vec<&str> = entities.iter().map(|named| named.name.as_str()).collect();
    assert_eq!(names, ["anbn", "abb", "anbn"]);
    let pda = pda_of(&entities[0].entity);
    for (word, expected) in [("ab", true), ("aabb", true), ("aab", false), ("", false), ("ba", false)] {
        assert_eq!(Machine::accepts(pda, word), expected, "pda on {word:?}");
    }
    for word in words(&['a', 'b'], 5) {
        assert_eq!(finite_of(&entities[1].entity).accepts(&word), word.ends_with("abb"), "regex on {word:?}");
    }
}

#[test]
fn entity_file_documented_aliases_test() {
    let (entities, errors) = parse_entity_file(
        "\
entity: re
regex: a
entity: automaton
transitions: (p, a) -> q
initial: p
final: q
",
    );
    assert!(errors.is_empty(), "{errors:?}");
    assert!(finite_of(&entities[0].entity).accepts("a"));
    assert!(finite_of(&entities[1].entity).accepts("a"));
}

#[test]
fn entity_file_grammar_error_reports_its_own_line_test() {
    let (_, errors) = parse_entity_file(
        "\
entity: grammar
productions: S -> a T
# a comment between the production lines
productions: T b
",
    );
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].line, 4);
}

#[test]
fn entity_file_writers_refuse_labels_that_would_drift_test() {
    let tm = turing('_', 1, &[], &[(0, 0, "a;b;c/R")]);
    assert!(write_turing_entity("e", &tm).is_err(), "a;b;c/R is not a turing label");

    let mut pda = PushdownAutomata::new("Z".to_string());
    pda.add_state_with_id_label(0, "q0");
    pda.make_initial(0);
    pda.add_transition(0, 0, "a;Z/a,,b".to_string());
    assert!(write_pushdown_entity("e", &pda).is_err(), "an empty push segment must not become ε");

    let fa = finite(2, &[1], &[(0, 1, " a")]);
    assert!(write_finite_entity("e", &fa).is_err(), "a padded symbol must not be trimmed on save");

    let fa = finite(2, &[1], &[(0, 1, "a(")]);
    assert!(write_finite_entity("e", &fa).is_err());
}

#[test]
fn entity_file_pda_labels_round_trip_by_meaning_test() {
    let mut pda = PushdownAutomata::new("Z".to_string());
    pda.add_n_states(2);
    pda.make_initial(0);
    pda.make_final(1);
    for label in ["ε", ";Z/AB", "a;ε/X,ε", "b;A/AB,C", "ε;Z/Z"] {
        pda.add_transition(0, 1, label.to_string());
    }
    let text = write_pushdown_entity("p", &pda).expect("serializable");
    let (entities, errors) = parse_entity_file(&text);
    assert!(errors.is_empty(), "{errors:?}\n{text}");
    let ops = |machine: &PushdownAutomata| -> BTreeSet<String> {
        machine
            .get_states_by_id_ref()
            .values()
            .flat_map(|state| state.labels().map(|(_, label)| PdaOp::parse(label).unwrap().to_string()))
            .collect()
    };
    assert_eq!(ops(pda_of(&entities[0].entity)), ops(&pda));
}

#[test]
fn entity_file_grammar_writer_preserves_language_test() {
    let mut grammar = Grammar::new("S");
    grammar.add_variable("B");
    grammar.add_production("S", &["a", "B"]);
    grammar.add_production("S", &["c"]);
    let text = write_grammar_entity("g", &grammar).unwrap();
    assert_eq!(text, "entity: grammar\nname: g\nproductions: S -> c");
    let (entities, errors) = parse_entity_file(&text);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(!grammar_of(&entities[0].entity).generate("a"));
}

#[test]
fn entity_file_parentheses_inside_a_tuple_are_rejected_test() {
    let message = first_error_message(
        "\
entity: pda
transitions: (q0, ε, Z, PZ) -> q0, (q0, (, P, PP) -> q0, (q0, ), P, ε) -> q1
initial: q0
",
    );
    assert!(message.contains("reserved"), "message: {message}");
}

#[test]
fn entity_file_reports_machine_label_problems_on_their_line_test() {
    let source = "\
entity: tm
states: a, b
transitions: (a, 0, 1, R) -> b
transitions: (a, 1, 1, R, _, _, S) -> b
initial: a
";
    let (_, errors) = parse_entity_file(source);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].line, 4);
    assert!(errors[0].message.contains("inconsistent tape count"), "{errors:?}");

    let source = "\
entity: pda
states: a
transitions: (a, x;y, Z, A) -> a
initial: a
";
    let (_, errors) = parse_entity_file(source);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].line, 3);
    assert!(errors[0].message.contains("malformed transition label"), "{errors:?}");
}
