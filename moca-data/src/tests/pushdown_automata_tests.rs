use crate::pushdown_automata::PushdownAutomata;
use crate::state_machine::StateMachine;

#[test]
fn check_input_dpa_test() {
    let mut pushdown_automata = PushdownAutomata::new("Z".to_string());
    pushdown_automata.add_n_states(3);
    pushdown_automata.make_initial(0);
    pushdown_automata.make_final(2);
    pushdown_automata.add_transition(0, 0, "0;Z/AZ".to_string());
    pushdown_automata.add_transition(0, 0, "0;A/AA".to_string());
    pushdown_automata.add_transition(0, 1, "ε".to_string());
    pushdown_automata.add_transition(1, 1, "1;A/ε".to_string());
    pushdown_automata.add_transition(1, 2, "ε;Z/Z".to_string());
    assert!(!pushdown_automata.check_input("0"));
    assert!(!pushdown_automata.check_input("001"));
    assert!(!pushdown_automata.check_input("001111"));
    assert!(!pushdown_automata.check_input("0001111"));
    assert!(!pushdown_automata.check_input("sy"));
    assert!(pushdown_automata.check_input("01"));
    assert!(pushdown_automata.check_input("00001111"));
    assert!(pushdown_automata.check_input(""));
    let mut pushdown_automaton = PushdownAutomata::new("Z".to_string());
    pushdown_automaton.add_n_states(3);
    pushdown_automaton.make_initial(0);
    pushdown_automaton.make_final(2);
    pushdown_automaton.add_transition(0, 0, "a;Z/AZ".to_string());
    pushdown_automaton.add_transition(0, 0, "a;A/AA".to_string());
    pushdown_automaton.add_transition(0, 1, "ε".to_string());
    pushdown_automaton.add_transition(1, 1, "b;A/ε".to_string());
    pushdown_automaton.add_transition(1, 2, "ε;Z/Z".to_string());
    assert!(pushdown_automaton.check_input("ab"));
    assert!(pushdown_automaton.check_input(""));
    assert!(pushdown_automaton.check_input("aabb"));
    assert!(pushdown_automaton.check_input("aaabbb"));
    assert!(!pushdown_automaton.check_input("aab"));
    assert!(!pushdown_automaton.check_input("ba"));
    assert!(!pushdown_automaton.check_input("abb"));
    assert!(!pushdown_automaton.check_input("sy"));
}

#[test]
fn remove_state_cleans_bookkeeping_test() {
    let mut pushdown_automata = PushdownAutomata::new("Z".to_string());
    pushdown_automata.add_n_states(3);
    pushdown_automata.make_initial(0);
    pushdown_automata.make_final(1);
    pushdown_automata.add_transition(0, 1, "a;Z/a".to_string());
    pushdown_automata.add_transition(1, 2, "b;a/Z".to_string());
    pushdown_automata.remove_state(1);
    assert!(pushdown_automata.get_final_states().is_empty());
    let start = pushdown_automata.initial_configuration("a").unwrap();
    assert!(pushdown_automata.step_all(&start).is_empty());
}

#[test]
fn machine_validate_test() {
    use crate::state_machine::{Machine, MachineKind};
    let mut pushdown_automata = PushdownAutomata::new("Z".to_string());
    pushdown_automata.add_n_states(2);
    pushdown_automata.make_initial(0);
    pushdown_automata.make_final(1);
    pushdown_automata.add_transition(0, 1, "a;Z/a".to_string());
    assert_eq!(pushdown_automata.kind(), MachineKind::Pushdown);
    assert!(pushdown_automata.validate().is_ok());
    assert!(pushdown_automata.accepts("a"));
    assert!(!pushdown_automata.accepts("aa"));

    pushdown_automata.remove_state(0);
    assert!(pushdown_automata.validate().is_err());
}

use std::collections::{HashSet, VecDeque};
use crate::pushdown_automata::PdaConfiguration;
use crate::state_machine::Machine;

fn build_anbn_pda() -> PushdownAutomata {
    let mut pda = PushdownAutomata::new("Z".to_string());
    pda.add_n_states(3);
    pda.make_initial(0);
    pda.make_final(2);
    pda.add_transition(0, 0, "a;Z/AZ".to_string());
    pda.add_transition(0, 0, "a;A/AA".to_string());
    pda.add_transition(0, 1, "ε".to_string());
    pda.add_transition(1, 1, "b;A/ε".to_string());
    pda.add_transition(1, 2, "ε;Z/Z".to_string());
    pda
}

fn accepts_by_stepping(pda: &PushdownAutomata, input: &str, max_visited: usize, max_stack_depth: usize) -> bool {
    let initial = match pda.initial_configuration(input) {
        Some(config) => config,
        None => return false,
    };
    let mut visited: HashSet<PdaConfiguration> = HashSet::new();
    let mut worklist: VecDeque<PdaConfiguration> = VecDeque::from([initial]);
    while let Some(current) = worklist.pop_front() {
        if !visited.insert(current.clone()) {
            continue;
        }
        if visited.len() > max_visited {
            return false;
        }
        if pda.is_accepting(&current) {
            return true;
        }
        for next in pda.step_all(&current) {
            if next.stack().len() <= max_stack_depth {
                worklist.push_back(next);
            }
        }
    }
    false
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
fn pda_step_check_input_parity_test() {
    let anbn = build_anbn_pda();

    let mut branching = PushdownAutomata::new("Z".to_string());
    branching.add_n_states(4);
    branching.make_initial(0);
    branching.make_final(3);
    branching.add_transition(0, 1, "a;Z/P".to_string());
    branching.add_transition(0, 2, "a;Z/Q".to_string());
    branching.add_transition(1, 1, "b;P/ε".to_string());
    branching.add_transition(2, 2, "c;Q/ε".to_string());
    branching.add_transition(1, 3, "ε;Z/Z".to_string());
    branching.add_transition(2, 3, "ε;Z/Z".to_string());
    assert!(!branching.is_deterministic());

    for pda in [&anbn, &branching] {
        for input in [
            "", "a", "ab", "abb", "ac", "acc", "aa", "aba", "b", "c", "abc",
            "aab", "abbc", "aabb",
        ] {
            let expected = pda.check_input(input);
            assert_eq!(
                accepts_by_stepping(pda, input, 150_000, 4096),
                expected,
                "stepping parity broken on {input:?}"
            );
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
    use crate::state_machine::Machine;
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
