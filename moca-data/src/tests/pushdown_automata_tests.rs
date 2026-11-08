use super::pushdown as pda;
use crate::pushdown_automata::{PdaOp, PushdownAutomata};
use crate::state_machine::{Machine, StateMachine};

fn build_anbn_pda() -> PushdownAutomata {
    pda(3, &[2], &[(0, 0, "a;Z/AZ"), (0, 0, "a;A/AA"), (0, 1, "ε"), (1, 1, "b;A/ε"), (1, 2, "ε;Z/Z")])
}

#[test]
fn check_input_dpa_test() {
    let anbn = build_anbn_pda();
    for (input, expected) in [
        ("", true), ("ab", true), ("aabb", true), ("aaabbb", true),
        ("a", false), ("aab", false), ("ba", false), ("abb", false), ("aaabbbb", false), ("sy", false),
    ] {
        assert_eq!(anbn.check_input(input), expected, "on {input:?}");
    }
}

#[test]
fn remove_state_cleans_bookkeeping_test() {
    let mut pushdown_automata = pda(3, &[1], &[(0, 1, "a;Z/a"), (1, 2, "b;a/Z")]);
    pushdown_automata.remove_state(1);
    assert!(pushdown_automata.get_final_states().is_empty());
    let start = pushdown_automata.initial_configuration("a").unwrap();
    assert!(pushdown_automata.step_all(&start).is_empty());
}

#[test]
fn machine_validate_test() {
    let mut pushdown_automata = pda(2, &[1], &[(0, 1, "a;Z/a")]);
    assert_eq!(pushdown_automata.validate(), Ok(()));
    assert!(pushdown_automata.accepts("a"));
    assert!(!pushdown_automata.accepts("aa"));

    pushdown_automata.remove_state(0);
    assert_eq!(pushdown_automata.validate(), Err("The pushdown automaton has no initial state.".to_string()));
}

#[test]
fn pda_label_codec_round_trip_test() {
    let op = PdaOp::parse("a;Z/AZ").unwrap();
    assert_eq!(op, PdaOp { read: Some("a".into()), pop: Some("Z".into()), push_entries: vec!["Z".into(), "A".into()] });
    assert_eq!(PdaOp::parse("ε").unwrap(), PdaOp::parse("ε;ε/ε").unwrap());
    assert_eq!(PdaOp::parse(";/").unwrap(), PdaOp::parse("ε;ε/ε").unwrap());
    for (label, rendered) in [
        ("a;Z/AZ", "a;Z/AZ"),
        ("a;Z/A,Z", "a;Z/AZ"),
        ("ε", "ε;ε/ε"),
        ("b;Top/Top,ε", "b;Top/Top,ε"),
        ("c;A/S,x,Top", "c;A/S,x,Top"),
        ("x/y;A/ε", "x/y;A/ε"),
    ] {
        let op = PdaOp::parse(label).unwrap();
        assert_eq!(op.to_string(), rendered, "rendering {label:?}");
        assert_eq!(PdaOp::parse(rendered).unwrap(), op, "re-parsing {rendered:?}");
    }
    assert_eq!(PdaOp::parse("a;Z/x,,y"), Err("empty push segment"));
    assert!(PdaOp::parse("a;b;c/d").is_err());
    assert!(PdaOp::is_reserved_symbol("a,b") && PdaOp::is_reserved_symbol("ε") && !PdaOp::is_reserved_symbol("ab"));
}

#[test]
fn pda_equivalent_labels_do_not_conflict_test() {
    let pda = pda(2, &[], &[(0, 1, "ε"), (0, 1, "ε;ε/ε")]);
    assert!(pda.is_deterministic(), "a bare ε and ε;ε/ε are the same move");
}

#[test]
fn pda_configuration_stepping_test() {
    let pda = build_anbn_pda();

    let config = pda.initial_configuration("ab").unwrap();
    assert_eq!(config.state_id(), 0);
    assert_eq!(config.remaining_input(), "ab");
    assert_eq!(config.stack(), ["Z".to_string()]);
    assert!(!pda.is_accepting(&config));

    let successors = pda.step_all(&config);
    assert_eq!(successors.len(), 2);
    let consuming = successors
        .iter()
        .find(|c| c.remaining_input() == "b")
        .expect("the consuming branch must exist");
    assert_eq!(consuming.stack(), ["Z".to_string(), "A".to_string()]);

    let entered_popper = pda
        .step_all(consuming)
        .into_iter()
        .find(|c| c.state_id() == 1 && c.remaining_input() == "b" && c.stack() == ["Z".to_string(), "A".to_string()])
        .unwrap();
    let after_pop = pda
        .step_all(&entered_popper)
        .into_iter()
        .find(|c| c.remaining_input() == "" && c.stack() == ["Z".to_string()])
        .unwrap();
    assert!(!pda.is_accepting(&after_pop));
    let accepted = pda
        .step_all(&after_pop)
        .into_iter()
        .find(|c| c.state_id() == 2)
        .unwrap();
    assert!(pda.is_accepting(&accepted));

    let mut headless = PushdownAutomata::new("Z".to_string());
    headless.add_n_states(1);
    assert!(headless.initial_configuration("ab").is_none());
}

#[test]
fn pda_check_input_language_test() {
    let anbn = build_anbn_pda();

    let branching = pda(4, &[3], &[
        (0, 1, "a;Z/PZ"), (0, 2, "a;Z/QZ"), (1, 1, "b;P/ε"), (2, 2, "c;Q/ε"), (1, 3, "ε;Z/Z"), (2, 3, "ε;Z/Z"),
    ]);
    assert!(!branching.is_deterministic());

    let inputs = ["", "a", "ab", "abb", "ac", "acc", "aa", "aba", "b", "c", "abc", "aab", "abbc", "aabb"];
    for (machine, accepted) in [(&anbn, &["", "ab", "aabb"][..]), (&branching, &["ab", "ac"][..])] {
        for input in inputs {
            let expected = accepted.contains(&input);
            assert_eq!(machine.check_input(input), expected, "check_input on {input:?}");
        }
    }
}

#[test]
fn pda_multisymbol_push_test() {
    let mut pda = PushdownAutomata::new("Z".to_string());
    pda.add_n_states(3);
    pda.make_initial(0);
    pda.make_final(2);
    pda.add_transition(0, 1, "ab;Z/A,Z".to_string());
    pda.add_transition(1, 1, "ε;A/ε".to_string());
    pda.add_transition(1, 2, "ε;Z/Z".to_string());

    let config = pda.initial_configuration("ab").unwrap();
    let stepped = pda.step_all(&config);
    assert_eq!(stepped.len(), 1);
    assert_eq!(stepped[0].remaining_input(), "");
    assert_eq!(stepped[0].stack(), ["Z".to_string(), "A".to_string()]);

    assert!(pda.check_input("ab"));
    assert!(!pda.check_input("a"));
    assert!(!pda.check_input("ba"));
    assert!(!pda.check_input("aba"));
    assert!(pda.validate().is_ok());
}

#[test]
fn pda_comma_epsilon_part_test() {
    let mut pda = PushdownAutomata::new("Z".to_string());
    pda.add_n_states(2);
    pda.make_initial(0);
    pda.add_transition(0, 1, "ε;Z/a,ε,b".to_string());

    let config = pda.initial_configuration("whatever").unwrap();
    let stepped = pda.step_all(&config);
    assert_eq!(stepped.len(), 1);
    assert_eq!(stepped[0].stack(), ["b".to_string(), "a".to_string()]);
}

#[test]
fn pda_malformed_comma_push_test() {
    let mut pda = PushdownAutomata::new("Z".to_string());
    pda.add_n_states(2);
    pda.make_initial(0);
    pda.make_final(1);
    pda.add_transition(0, 1, "a;Z/x,,y".to_string());

    let error = Machine::validate(&pda).expect_err("empty push segment must not validate");
    assert!(error.contains("empty push segment"), "{}", error);

    assert!(!pda.check_input("a"));
}

#[test]
fn pushdown_modify_input_cache_test() {
    let mut pda = PushdownAutomata::new("Z".to_string());
    pda.add_state_with_id_label(0, "q0");
    pda.add_state_with_id_label(1, "qf");
    pda.add_transition(0, 1, "a;Z/A".to_string());
    pda.make_initial(0);
    pda.make_final(1);
    assert!(Machine::accepts(&pda, "a"));
    assert!(!Machine::accepts(&pda, "b"));

    pda.modify_input(0, 1, "a;Z/A", "b;Z/A".to_string());
    assert!(Machine::accepts(&pda, "b"));
    assert!(!Machine::accepts(&pda, "a"));

    pda.remove_transition(0, 1, "b;Z/A");
    assert!(!Machine::accepts(&pda, "b"));
}

#[test]
fn pushdown_remove_state_cache_test() {
    let mut pda = PushdownAutomata::new("Z".to_string());
    pda.add_state_with_id_label(0, "q0");
    pda.add_state_with_id_label(1, "q1");
    pda.add_state_with_id_label(2, "qf");
    pda.add_transition(0, 1, "ε;Z/Z".to_string());
    pda.add_transition(1, 2, "a;Z/ε".to_string());
    pda.make_initial(0);
    pda.make_final(2);
    assert!(Machine::accepts(&pda, "a"));

    pda.remove_state(1);
    assert!(!Machine::accepts(&pda, "a"));

    pda.add_transition(0, 2, "b;Z/ε".to_string());
    assert!(Machine::accepts(&pda, "b"));
}

#[test]
fn pda_epsilon_pop_fires_on_nonempty_stack_test() {
    let mut pda = PushdownAutomata::new("Z".to_string());
    pda.add_n_states(2);
    pda.make_initial(0);
    pda.make_final(1);
    pda.add_transition(0, 1, "a;ε/A".to_string());
    pda.add_transition(1, 1, "a;ε/A".to_string());
    assert!(pda.check_input("a"));
    assert!(pda.check_input("aaa"));
    assert!(!pda.check_input(""));
}

#[test]
fn pda_malformed_label_does_not_panic_test() {
    let mut pda = PushdownAutomata::new("Z".to_string());
    pda.add_n_states(2);
    pda.make_initial(0);
    pda.make_final(1);
    pda.add_transition(0, 1, "a".to_string());
    assert!(!pda.check_input("a"));
    assert!(pda.validate().is_err());
}

#[test]
fn pda_determinism_tracks_every_pair_test() {
    let mut pda = PushdownAutomata::new("Z".to_string());
    pda.add_n_states(3);
    pda.make_initial(0);
    pda.add_transition(0, 1, "a;A/ε".to_string());
    pda.add_transition(0, 1, "a;B/ε".to_string());
    assert!(pda.is_deterministic());
    pda.add_transition(0, 2, "a;A/ε".to_string());
    assert!(!pda.is_deterministic());
    pda.remove_transition(0, 2, "a;A/ε");
    assert!(pda.is_deterministic());
    pda.add_transition(0, 2, "ε;C/ε".to_string());
    assert!(pda.is_deterministic());
    pda.add_transition(0, 2, "ε;A/ε".to_string());
    assert!(!pda.is_deterministic());
}
