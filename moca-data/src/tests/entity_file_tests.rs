use crate::entity_file::{
    parse_entity_file, write_finite_entity, write_grammar_entity, write_pushdown_entity,
    write_turing_entity, Entity,
};
use crate::finite_automata::FiniteAutomata;
use crate::grammar::Grammar;
use crate::pushdown_automata::PushdownAutomata;
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
    assert_eq!(tm_of(&entities[0].entity).get_tape_count(), 2);
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
    assert!(message.contains("'blank:' is only valid"), "message: {message}");

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
    assert_eq!(loaded.get_tape_count(), 2);
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
