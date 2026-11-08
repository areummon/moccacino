use std::collections::{BTreeSet, HashSet};

use super::{finite, words};
use crate::finite_automata::{FiniteAutomata, FiniteConfiguration};
use crate::state::State;
use crate::state_machine::{Machine, StateMachine};

fn labels_of(state: &State) -> Vec<(u64, &str)> {
    state.labels().collect()
}

fn odd_a_dfa() -> FiniteAutomata {
    finite(2, &[1], &[(0, 1, "a"), (0, 0, "b"), (1, 0, "a"), (1, 1, "b")])
}

fn four_state_nfa() -> FiniteAutomata {
    finite(4, &[3], &[
        (0, 1, "0"), (0, 2, "1"), (1, 1, "0"), (1, 2, "1"), (1, 3, "1"),
        (2, 2, "1"), (2, 1, "0"), (2, 3, "0"), (3, 3, "0"), (3, 3, "1"),
    ])
}

fn epsilon_nfa() -> FiniteAutomata {
    finite(6, &[3, 4], &[
        (0, 1, ""), (0, 4, ""), (1, 2, ""), (1, 1, "a"),
        (2, 3, "a"), (2, 2, "b"), (4, 5, "a"), (5, 4, "b"),
    ])
}

#[test]
fn state_transition_crud_test() {
    let mut state = State::new("name".to_string());
    state.add_transition(1, "a".to_string());
    state.add_transition(1, "b".to_string());
    state.add_transition(1, "b".to_string());
    state.add_transition(2, "c".to_string());
    assert_eq!(labels_of(&state), [(1, "a"), (1, "b"), (2, "c")]);
    state.remove_transition(1, "a");
    state.modify_input(2, "c", "d".to_string());
    assert_eq!(labels_of(&state), [(1, "b"), (2, "d")]);
    state.remove_state(1);
    assert_eq!(labels_of(&state), [(2, "d")]);
}

#[test]
fn state_machine_add_transition_test() {
    let mut automata = FiniteAutomata::new();
    automata.add_state();
    automata.add_transition(0, 1, "lovelyz".to_string());
    assert!(labels_of(&automata.get_states_by_id_ref()[&0]).is_empty(), "the target state does not exist yet");
    automata.add_n_states(2);
    automata.add_transition(0, 1, "lovelyz".to_string());
    automata.add_transition(1, 2, "for you".to_string());
    automata.add_transition(5, 0, "ghost".to_string());
    let states = automata.get_states_by_id_ref();
    assert_eq!(states.len(), 3);
    assert_eq!(labels_of(&states[&0]), [(1, "lovelyz")]);
    assert_eq!(labels_of(&states[&1]), [(2, "for you")]);
}

#[test]
fn state_machine_rename_and_modify_input_test() {
    let mut automata = finite(2, &[], &[(0, 1, "fiestar")]);
    automata.modify_name(0, "jiyeon".to_string());
    automata.modify_input(0, 1, "fiestar", "secret".to_string());
    let state = &automata.get_states_by_id_ref()[&0];
    assert_eq!(state.name, "jiyeon");
    assert_eq!(labels_of(state), [(1, "secret")]);
}

#[test]
fn state_machine_remove_state_test() {
    let mut automata = finite(3, &[], &[(0, 1, "badvillain"), (2, 1, "badtitude"), (0, 2, "kept")]);
    automata.remove_state(1);
    let states = automata.get_states_by_id_ref();
    assert_eq!(states.len(), 2);
    assert_eq!(labels_of(&states[&0]), [(2, "kept")]);
    assert!(labels_of(&states[&2]).is_empty());
}

#[test]
fn check_input_dfa_test() {
    let automata = odd_a_dfa();
    assert!(!automata.check_input("abbbaabaaba"));
    assert!(automata.check_input("bbbbbbabaaabba"));
    assert!(automata.check_input("aaaaaaaaaaaaa"));

    let mut transitions = vec![(0, 1, "#")];
    for digit in ["0", "1", "2"] {
        transitions.extend([(1, 2, digit), (2, 2, digit)]);
    }
    transitions.extend([
        (2, 3, "a"), (2, 4, "b"), (3, 3, "a"), (3, 4, "b"), (4, 3, "a"),
        (4, 5, "b"), (5, 3, "a"), (5, 6, "b"), (6, 6, "a"), (6, 6, "b"),
    ]);
    let automata = finite(7, &[6], &transitions);
    assert!(!automata.check_input("adsf"));
    assert!(!automata.check_input(""));
    assert!(!automata.check_input("#1010201aabb"));
    assert!(!automata.check_input("1010201abbb"));
    assert!(automata.check_input("#1010abbba"));
    assert!(automata.check_input("#1010bbbbb"));
    assert!(automata.check_input("#2222aaaaaaaaaaabbb"));
}

#[test]
fn check_input_nfa_test() {
    let automata = four_state_nfa();
    assert!(!automata.check_input(""));
    assert!(!automata.check_input("0000000000"));
    assert!(!automata.check_input("111111111"));
    assert!(!automata.check_input("10x"));
    assert!(automata.check_input("10"));
    assert!(automata.check_input("01"));
    assert!(automata.check_input("01111111111110"));
    assert!(automata.check_input("00000000000001"));
    assert!(automata.check_input("010101010101010"));

    let automata = epsilon_nfa();
    assert!(!automata.check_input("abbbbbbbbbb"));
    assert!(!automata.check_input("b"));
    assert!(!automata.check_input("aababababababa"));
    assert!(automata.check_input("a"));
    assert!(automata.check_input(""));
    assert!(automata.check_input("abababababababab"));
    assert!(automata.check_input("aaaaaabbbbbbbbba"));
    assert!(automata.check_input("abbbbbbbbbbba"));
}

#[test]
fn to_dfa_test() {
    let automata = finite(5, &[3, 4], &[(0, 1, ""), (0, 2, ""), (1, 3, "a"), (3, 3, "a"), (2, 4, "b"), (4, 4, "b")]);
    assert!(!automata.is_deterministic());
    let dfa = automata.to_dfa();
    assert!(dfa.is_deterministic());
    assert!(!dfa.check_input(""));
    assert!(!dfa.check_input("ab"));
    assert!(!dfa.check_input("abaaaa"));
    assert!(dfa.check_input("a"));
    assert!(dfa.check_input("b"));
    assert!(dfa.check_input("bbbbbbbb"));
    assert!(dfa.check_input("aaaaaaaa"));

    for (automata, alphabet) in [(epsilon_nfa(), ['a', 'b']), (four_state_nfa(), ['0', '1'])] {
        let dfa = automata.to_dfa();
        assert!(dfa.is_deterministic());
        for word in words(&alphabet, 8) {
            assert_eq!(dfa.check_input(&word), automata.check_input(&word), "to_dfa changed the language on {word:?}");
        }
    }
}

#[test]
fn to_dfa_and_minimize_never_panic_test() {
    let mut no_initial = FiniteAutomata::new();
    no_initial.add_n_states(2);
    no_initial.add_transition(0, 1, "ε".to_string());
    assert!(no_initial.to_dfa().get_states_by_id_ref().is_empty());

    let minimized = epsilon_nfa().minimize();
    assert!(minimized.is_deterministic());
    assert_eq!(minimized.get_states_by_id_ref().len(), epsilon_nfa().to_dfa().minimize().get_states_by_id_ref().len());
    for word in words(&['a', 'b'], 7) {
        assert_eq!(minimized.check_input(&word), epsilon_nfa().check_input(&word), "on {word:?}");
    }

    let dfa = odd_a_dfa().to_dfa();
    for word in words(&['a', 'b'], 6) {
        assert_eq!(dfa.check_input(&word), word.matches('a').count() % 2 == 1, "on {word:?}");
    }
}

#[test]
fn default_matches_new_test() {
    assert!(FiniteAutomata::default().is_deterministic());
    assert!(FiniteAutomata::new().is_deterministic());
}

#[test]
fn minimize_test() {
    let bloated = finite(6, &[2, 3, 4], &[
        (0, 1, "0"), (0, 2, "1"), (1, 0, "0"), (1, 3, "1"), (3, 4, "0"), (3, 5, "1"),
        (2, 5, "1"), (2, 4, "0"), (4, 4, "0"), (4, 5, "1"), (5, 5, "0"), (5, 5, "1"),
    ]);
    let minimized = bloated.minimize();
    assert_eq!(minimized.get_states_by_id_ref().len(), 3);
    assert!(!minimized.check_input("0000000000000"));
    assert!(!minimized.check_input("1a0101010"));
    assert!(!minimized.check_input("a"));
    assert!(!minimized.check_input("11"));
    assert!(minimized.check_input("00000000000001"));
    assert!(minimized.check_input("1"));
    assert!(minimized.check_input("00001"));
    assert!(minimized.check_input("100000000000000000000000"));

    let bloated = finite(6, &[3, 5], &[
        (0, 1, "0"), (0, 3, "1"), (1, 0, "0"), (1, 3, "1"), (2, 1, "0"), (2, 4, "1"),
        (4, 3, "1"), (4, 3, "0"), (3, 5, "0"), (3, 5, "1"), (5, 5, "1"), (5, 5, "0"),
    ]);
    let minimized = bloated.minimize();
    assert_eq!(minimized.get_states_by_id_ref().len(), 2);
    assert!(!minimized.check_input(""));
    assert!(!minimized.check_input("0"));
    assert!(!minimized.check_input("000000000"));
    assert!(minimized.check_input("1"));
    assert!(minimized.check_input("01010101"));

    let bloated = finite(5, &[4], &[
        (0, 2, "b"), (0, 1, "a"), (1, 1, "a"), (1, 3, "b"), (2, 2, "b"),
        (2, 1, "a"), (3, 4, "b"), (3, 1, "a"), (4, 2, "b"), (4, 1, "a"),
    ]);
    let minimized = bloated.minimize();
    assert_eq!(minimized.get_states_by_id_ref().len(), 4);
    assert!(!minimized.check_input("ab"));
    assert!(!minimized.check_input("abbaaaaa"));
    assert!(!minimized.check_input("abaaaaaa"));
    assert!(minimized.check_input("abb"));
    assert!(minimized.check_input("abbabb"));
    assert!(minimized.check_input("abbbbaabb"));
}

struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

fn build_mod3_redundant_dfa() -> FiniteAutomata {
    let mut transitions = Vec::new();
    for residue in 0..3u64 {
        let next = (residue + 1) % 3;
        transitions.extend([
            (residue, next, "b"),
            (residue + 3, next + 3, "b"),
            (residue, residue, "a"),
            (residue + 3, residue + 3, "a"),
        ]);
    }
    finite(6, &[0, 3], &transitions)
}

fn build_partial_dfa() -> FiniteAutomata {
    finite(3, &[1], &[(0, 1, "a"), (1, 1, "a"), (2, 2, "b")])
}

fn build_empty_language_dfa() -> FiniteAutomata {
    finite(2, &[], &[(0, 0, "a"), (0, 1, "b"), (1, 1, "a"), (1, 0, "b")])
}

fn build_all_final_dfa() -> FiniteAutomata {
    finite(2, &[0, 1], &[(0, 1, "a"), (0, 0, "b"), (1, 0, "a"), (1, 1, "b")])
}

fn delta(automata: &FiniteAutomata, id: u64, symbol: &str) -> Option<u64> {
    labels_of(&automata.get_states_by_id_ref()[&id])
        .into_iter()
        .find(|(_, label)| *label == symbol)
        .map(|(target, _)| target)
}

fn reference_class_count(automata: &FiniteAutomata) -> usize {
    let alphabet: BTreeSet<&str> = automata
        .get_states_by_id_ref()
        .values()
        .flat_map(|state| state.labels().map(|(_, label)| label))
        .collect();
    let Some(initial) = automata.get_initial_state_id() else {
        return 0;
    };
    let mut visited: BTreeSet<u64> = BTreeSet::new();
    let mut stack = vec![initial];
    while let Some(id) = stack.pop() {
        if visited.insert(id) {
            stack.extend(alphabet.iter().filter_map(|symbol| delta(automata, id, symbol)));
        }
    }
    let ids: Vec<u64> = visited.into_iter().collect();
    let n_real = ids.len();
    let total = n_real + 1;
    let finals = automata.get_final_states();

    let step = |i: usize, s: &str| -> usize {
        if i == n_real {
            return n_real;
        }
        delta(automata, ids[i], s)
            .and_then(|target| ids.iter().position(|&x| x == target))
            .unwrap_or(n_real)
    };

    let mut dist = vec![vec![false; total]; total];
    for i in 0..total {
        for j in 0..total {
            let i_final = i < n_real && finals.contains(&ids[i]);
            let j_final = j < n_real && finals.contains(&ids[j]);
            dist[i][j] = i_final != j_final;
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..total {
            for j in 0..total {
                if dist[i][j] || i == j {
                    continue;
                }
                if alphabet.iter().any(|s| dist[step(i, s)][step(j, s)]) {
                    dist[i][j] = true;
                    changed = true;
                }
            }
        }
    }

    let mut parent: Vec<usize> = (0..total).collect();
    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    for i in 0..total {
        for j in 0..total {
            if !dist[i][j] {
                let a = find(&mut parent, i);
                let b = find(&mut parent, j);
                if a != b {
                    parent[a] = b;
                }
            }
        }
    }
    let classes: HashSet<usize> = (0..n_real).map(|i| find(&mut parent, i)).collect();
    classes.len()
}

#[test]
fn minimize_matches_reference_test() {
    let fixtures = [
        (build_mod3_redundant_dfa(), 3),
        (build_partial_dfa(), 2),
        (build_empty_language_dfa(), 1),
        (build_all_final_dfa(), 1),
    ];
    for (automata, expected_classes) in fixtures.iter() {
        assert_eq!(reference_class_count(automata), *expected_classes);
        assert_eq!(automata.minimize().get_states_by_id_ref().len(), *expected_classes);
    }
}

#[test]
fn minimize_random_dfas_match_reference_test() {
    let mut rng = XorShift(0x9E3779B97F4A7C15);
    let inputs = words(&['a', 'b'], 5);
    for _ in 0..400 {
        let n = 2 + (rng.next() % 7);
        let mut automata = FiniteAutomata::new();
        automata.add_n_states(n);
        automata.make_initial(0);
        for id in 0..n {
            if rng.next().is_multiple_of(3) {
                automata.make_final(id);
            }
            for symbol in ["a", "b"] {
                if !rng.next().is_multiple_of(8) {
                    automata.add_transition(id, rng.next() % n, symbol.to_string());
                }
            }
        }
        let minimized = automata.minimize();
        assert_eq!(
            minimized.get_states_by_id_ref().len(),
            reference_class_count(&automata),
            "Hopcroft disagrees with the table-filling reference"
        );
        for input in &inputs {
            assert_eq!(
                automata.check_input(input),
                minimized.check_input(input),
                "minimization changed the language on input {input:?}"
            );
        }
    }
}

#[test]
fn minimize_language_equivalence_test() {
    let fixtures = [
        build_mod3_redundant_dfa(),
        build_partial_dfa(),
        build_empty_language_dfa(),
        build_all_final_dfa(),
    ];
    for automata in fixtures.iter() {
        let minimized = automata.minimize();
        for input in words(&['a', 'b'], 8) {
            assert_eq!(
                automata.check_input(&input),
                minimized.check_input(&input),
                "minimization changed the language on input {input:?}"
            );
        }
    }
}

#[test]
fn remove_state_cleans_bookkeeping_test() {
    let mut automata = finite(3, &[1], &[(0, 1, "a")]);
    automata.remove_state(1);
    assert!(automata.get_final_states().is_empty());
    automata.remove_state(0);
    assert_eq!(automata.get_initial_state_id(), None);
}

#[test]
fn machine_validate_test() {
    let automata = finite(2, &[1], &[(0, 1, "a")]);
    assert_eq!(automata.validate(), Ok(()));

    let mut automata = FiniteAutomata::new();
    automata.add_n_states(1);
    assert_eq!(automata.validate(), Err("The automaton has no initial state.".to_string()));
}

fn stepping_machine_fixtures() -> Vec<FiniteAutomata> {
    let mut no_initial = FiniteAutomata::new();
    no_initial.add_n_states(2);
    no_initial.make_final(1);
    no_initial.add_transition(0, 1, "a".to_string());
    vec![
        odd_a_dfa(),
        four_state_nfa(),
        epsilon_nfa(),
        finite(3, &[2], &[(0, 1, "ε"), (1, 0, ""), (0, 2, "a")]),
        finite(3, &[1, 2], &[(0, 1, "ab"), (0, 2, "aba"), (1, 1, "a")]),
        build_mod3_redundant_dfa(),
        build_partial_dfa(),
        build_empty_language_dfa(),
        build_all_final_dfa(),
        no_initial,
    ]
}

fn frontier_accepts(automata: &FiniteAutomata, input: &str) -> bool {
    let Some(config) = automata.initial_configuration(input) else {
        return false;
    };
    let mut visited: HashSet<FiniteConfiguration> = HashSet::from([config.clone()]);
    let mut alive: Vec<FiniteConfiguration> = vec![config];
    loop {
        if alive.iter().any(|config| automata.is_accepting(config)) {
            return true;
        }
        let next_alive: Vec<FiniteConfiguration> = alive
            .iter()
            .flat_map(|config| automata.step_all(config))
            .filter(|successor| visited.insert(successor.clone()))
            .collect();
        if next_alive.is_empty() {
            return false;
        }
        alive = next_alive;
    }
}

fn deterministic_run_outcome(automata: &FiniteAutomata, input: &str) -> Result<bool, ()> {
    let Some(mut config) = automata.initial_configuration(input) else {
        return Ok(false);
    };
    let mut visited: HashSet<FiniteConfiguration> = HashSet::from([config.clone()]);
    loop {
        if automata.is_accepting(&config) {
            return Ok(true);
        }
        let mut successors = automata.step_all(&config);
        match successors.len() {
            0 => return Ok(false),
            1 => {
                let next = successors.remove(0);
                if !visited.insert(next.clone()) {
                    return Ok(false);
                }
                config = next;
            },
            _ => return Err(()),
        }
    }
}

#[test]
fn frontier_stepping_matches_check_input_test() {
    let mut inputs = words(&['a', 'b'], 5);
    inputs.extend(["aba", "ababa", "abab", "abaa"].map(String::from));
    for automata in stepping_machine_fixtures().iter() {
        for input in &inputs {
            assert_eq!(
                frontier_accepts(automata, input),
                automata.check_input(input),
                "frontier stepping disagrees with check_input on {input:?}"
            );
        }
    }
}

#[test]
fn deterministic_run_matches_check_input_test() {
    let fixtures: Vec<FiniteAutomata> = stepping_machine_fixtures()
        .into_iter()
        .filter(|automata| automata.is_deterministic())
        .collect();
    assert_eq!(fixtures.len(), 7);
    for automata in fixtures.iter() {
        for input in words(&['a', 'b'], 5) {
            if let Ok(outcome) = deterministic_run_outcome(automata, &input) {
                assert_eq!(outcome, automata.check_input(&input), "deterministic run disagrees on {input:?}");
            }
        }
    }

    let multi_char = finite(3, &[1, 2], &[(0, 1, "ab"), (0, 2, "aba"), (1, 1, "a")]);
    assert!(multi_char.is_deterministic(), "prefix-overlapping labels stay flagged deterministic");
    assert_eq!(deterministic_run_outcome(&multi_char, "aba"), Err(()));
}

#[test]
fn initial_configuration_and_acceptance_test() {
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(2);
    automata.make_final(1);
    automata.add_transition(0, 1, "a".to_string());
    assert!(automata.initial_configuration("a").is_none());

    automata.make_initial(0);
    let config = automata.initial_configuration("ab").unwrap();
    assert_eq!(config.state_id(), 0);
    assert_eq!(config.remaining_input(), "ab");
    assert!(!automata.is_accepting(&config));

    let stepped = automata.step_all(&config);
    assert_eq!(stepped.len(), 1);
    assert_eq!(stepped[0].state_id(), 1);
    assert_eq!(stepped[0].remaining_input(), "b");
    assert!(!automata.is_accepting(&stepped[0]));
    assert!(automata.step_all(&stepped[0]).is_empty(), "'b' is not consumable from state 1");

    let config = automata.initial_configuration("a").unwrap();
    let stepped = automata.step_all(&config);
    assert_eq!(stepped.len(), 1);
    assert!(automata.is_accepting(&stepped[0]), "state 1 is final and the input is consumed");
}

#[test]
fn step_all_epsilon_and_prefixes_test() {
    let automata = finite(4, &[2], &[(0, 1, "ε"), (0, 2, "ab"), (0, 3, "ax"), (1, 2, "b")]);
    let describe = |configs: &[FiniteConfiguration]| -> Vec<(u64, String)> {
        configs.iter().map(|c| (c.state_id(), c.remaining_input().clone())).collect()
    };

    let config = automata.initial_configuration("abc").unwrap();
    assert_eq!(describe(&automata.step_all(&config)), [(1, "abc".to_string()), (2, "c".to_string())]);

    let config = automata.initial_configuration("b").unwrap();
    let successors = automata.step_all(&config);
    assert_eq!(describe(&successors), [(1, "b".to_string())]);
    let successors = automata.step_all(&successors[0]);
    assert_eq!(describe(&successors), [(2, String::new())]);
    assert!(automata.is_accepting(&successors[0]));
}

#[test]
fn determinism_duplicates_and_removal_test() {
    let mut automata = finite(3, &[], &[(0, 1, "a"), (0, 1, "a")]);
    assert!(automata.is_deterministic());
    automata.add_transition(0, 2, "a".to_string());
    assert!(!automata.is_deterministic());
    automata.remove_transition(0, 2, "a");
    assert!(automata.is_deterministic());
    automata.add_transition(1, 2, "".to_string());
    assert!(!automata.is_deterministic(), "a blank label is an ε move");
    automata.modify_input(1, 2, "", "b".to_string());
    assert!(automata.is_deterministic());
    automata.add_transition(2, 0, "ε".to_string());
    automata.remove_state(2);
    assert!(automata.is_deterministic(), "removing a state forgets its ε move");
}

#[test]
fn add_state_after_deletion_keeps_existing_states_test() {
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(3);
    automata.modify_name(2, "kept".to_string());
    automata.remove_state(1);
    let id = automata.add_state();
    assert_ne!(id, 2);
    assert_eq!(automata.get_states_by_id_ref().len(), 3);
    assert_eq!(automata.get_states_by_id_ref()[&2].name, "kept");
}

fn ends_with_ab_nfa(labels_reversed: bool) -> FiniteAutomata {
    let mut loops = vec!["a", "b", "c"];
    if labels_reversed {
        loops.reverse();
    }
    let mut transitions: Vec<(u64, u64, &str)> = loops.into_iter().map(|label| (0, 0, label)).collect();
    transitions.extend([(0, 1, "a"), (1, 2, "b")]);
    finite(3, &[2], &transitions)
}

#[test]
fn derived_machines_are_reproducible_test() {
    use crate::entity_file::write_finite_entity;
    let reference = ends_with_ab_nfa(false);
    let expected_dfa = write_finite_entity("m", &reference.to_dfa()).unwrap();
    let expected_minimized = write_finite_entity("m", &reference.to_dfa().minimize()).unwrap();
    for attempt in 0..32 {
        let automata = ends_with_ab_nfa(attempt % 2 == 1);
        let loop_labels: Vec<&str> = labels_of(&automata.get_states_by_id_ref()[&0])
            .into_iter()
            .filter(|(target, _)| *target == 0)
            .map(|(_, label)| label)
            .collect();
        assert_eq!(loop_labels, ["a", "b", "c"], "labels iterate in sorted order");
        let dfa = automata.to_dfa();
        assert_eq!(write_finite_entity("m", &dfa).unwrap(), expected_dfa, "NFA -> DFA numbering is stable");
        assert_eq!(dfa.get_initial_state_id(), Some(0), "the DFA starts at q0");
        let minimized = dfa.minimize();
        assert_eq!(write_finite_entity("m", &minimized).unwrap(), expected_minimized, "minimized numbering is stable");
        assert_eq!(minimized.get_initial_state_id(), Some(0), "the minimized DFA starts at q0");
    }
}
