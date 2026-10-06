use std::collections::{BTreeSet, HashSet};

use crate::state::State;
use crate::finite_automata::{FiniteAutomata, FiniteConfiguration};
use crate::state_machine::{Machine, MachineKind, StateMachine};
use crate::state;

#[test]
fn add_transition_test() {
    let mut state = State::new("name".to_string());
    state.add_transition(1, "a".to_string());
    state.add_transition(1, "b".to_string());
    state.add_transition(2, "c".to_string());
    state.add_transition(2, "d".to_string());
    let mut flag1 = false;
    let mut flag2 = false;
    for (k,v) in state.iter_by_transition() {
        if *k == 1 && v.contains("a") && v.contains("b") {
            flag1 = true;
        }
        if *k == 2 && v.contains("c") && v.contains("d") {
            flag2 = true;
        }
    }
    assert!(flag1 && flag2);
}

fn count_transition(state: &State, in1: &str, in2: &str) -> u64 {
    let mut len = 0;
    for (_,v) in state.iter_by_transition() {
        if v.contains(in1) || v.contains(in2) {
            len += 1;
        }
    }
    len
}

#[test]
fn add_transition_repeated_elements_test() {
    let mut state = state::State::new("name".to_string());
    state.add_transition(2,"input".to_string());
    state.add_transition(2,"input".to_string());
    state.add_transition(2,"input".to_string());
    assert_eq!(state.iter_by_transition().len(), 1);
}

#[test]
fn remove_transition_test() {
    let mut state = state::State::new("assemble".to_string());
    state.add_transition(1, "worst".to_string());
    state.add_transition(1, "best".to_string());
    state.remove_transition(1,"worst");
    assert_eq!(count_transition(&state, "best", "worst"), 1);
    state.remove_transition(1, "best");
    assert_eq!(count_transition(&state, "best", "worst"), 0);
}

#[test]
fn modify_input_test() {
    let mut state = state::State::new("shion".to_string());
    state.add_transition(1, "cuerda".to_string());
    state.modify_input(1, "cuerda", "cuerno".to_string());
    assert_eq!(count_transition(&state, "cuerno", ""),1);
}

#[test]
fn remove_state_test() {
    let mut state = state::State::new("illit".to_string());
    state.add_transition(1, "magnetic".to_string());
    state.remove_state(1);
    assert_eq!(count_transition(&state, "magnetic", ""), 0);
}

#[test]
fn state_machine_add_state_test() {
    let mut automata = FiniteAutomata::new();
    automata.add_state();
    automata.add_state();
    assert_eq!(automata.iter_by_state().len(),2);
}

#[test]
fn state_machine_add_transition_test() {
    let mut automata = FiniteAutomata::new();
    automata.add_state();
    automata.add_transition(0, 1, "lovelyz".to_string());
    for (_k,v) in automata.iter_by_state() {
        assert_ne!(1, v.iter_by_transition().len());
    }
    automata.add_state();
    automata.add_transition(0, 1, "lovelyz".to_string());
    automata.add_state();
    automata.add_transition(1, 2, "for you".to_string());
    let mut len = 0;
    for (_k,v) in automata.iter_by_state() {
        for (_x,y) in v.iter_by_transition() {
            if y.contains("lovelyz") || y.contains("for you") {
                len += 1;
            }
        }
    }
    assert_eq!(len, 2);
}

#[test]
fn modify_name_test() {
    let mut automata = FiniteAutomata::new();
    automata.add_state();
    automata.modify_name(0, "jiyeon".to_string());
    for (_k,v) in automata.iter_by_state() {
        assert_eq!(v.name, "jiyeon");
    }
}

#[test]
fn state_machine_modify_input_test() {
    let mut automata = FiniteAutomata::new();
    automata.add_state();
    automata.add_state();
    automata.add_transition(0,1,"fiestar".to_string());
    automata.modify_input(0,1,"fiestar","secret".to_string());
    for (k,v) in automata.iter_by_state() {
        if *k == 0 {
            assert_eq!(count_transition(v, "secret", ""), 1);
        }
    }
}

#[test]
fn state_machine_remove_transition() {
    let mut automata = FiniteAutomata::new();
    automata.add_state();
    automata.add_state();
    automata.remove_state(1);
    assert_eq!(automata.iter_by_state().len(), 1);
}

#[test]
fn state_machine_remove_state() {
    let mut automata = FiniteAutomata::new();
    automata.add_state();
    automata.add_state();
    automata.add_state();
    automata.add_transition(0,1,"badvillain".to_string());
    automata.add_transition(2,1,"badtitude".to_string());
    automata.remove_state(1);
    assert_eq!(automata.iter_by_state().len(),2);
    let mut len = 0;
    for (_k,v) in automata.iter_by_state() {
        for (x,_y) in v.iter_by_transition() {
            if *x == 1 {
                len += 1;
            }
        }
    }
    assert_eq!(len,0);
}

#[test]
fn check_input_dfa_test() {
    let mut automata = FiniteAutomata::new();
    automata.add_state();
    automata.add_state();
    automata.add_transition(0,1, "a".to_string());
    automata.add_transition(0,0, "b".to_string());
    automata.add_transition(1,0, "a".to_string());
    automata.add_transition(1,1, "b".to_string());
    automata.make_final(1);
    automata.make_initial(0);
    assert!(!automata.check_input("abbbaabaaba"));
    assert!(automata.check_input("bbbbbbabaaabba"));
    assert!(automata.check_input("aaaaaaaaaaaaa"));
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(7);
    automata.make_initial(0);
    automata.make_final(6);
    automata.add_transition(0,1,"#".to_string());
    automata.add_transition(1,2,"0".to_string());
    automata.add_transition(1,2,"1".to_string());
    automata.add_transition(1,2,"2".to_string());
    automata.add_transition(2,2,"0".to_string());
    automata.add_transition(2,2,"1".to_string());
    automata.add_transition(2,2,"2".to_string());
    automata.add_transition(2,3,"a".to_string());
    automata.add_transition(2,4,"b".to_string());
    automata.add_transition(3,3,"a".to_string());
    automata.add_transition(3,4,"b".to_string());
    automata.add_transition(4,3,"a".to_string());
    automata.add_transition(4,5,"b".to_string());
    automata.add_transition(5,3,"a".to_string());
    automata.add_transition(5,6,"b".to_string());
    automata.add_transition(6,6,"a".to_string());
    automata.add_transition(6,6,"b".to_string());
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
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(4);
    automata.make_initial(0);
    automata.make_final(3);
    automata.add_transition(0,1, "0".to_string());
    automata.add_transition(0,2, "1".to_string());
    automata.add_transition(1,1, "0".to_string());
    automata.add_transition(1,2, "1".to_string());
    automata.add_transition(1,3, "1".to_string());
    automata.add_transition(2,2, "1".to_string());
    automata.add_transition(2,1, "0".to_string());
    automata.add_transition(2,3, "0".to_string());
    automata.add_transition(3,3, "0".to_string());
    automata.add_transition(3,3, "1".to_string());
    assert!(!automata.check_input(""));
    assert!(!automata.check_input("0000000000"));
    assert!(!automata.check_input("111111111"));
    assert!(!automata.check_input("10x"));
    assert!(automata.check_input("10"));
    assert!(automata.check_input("01"));
    assert!(automata.check_input("01111111111110"));
    assert!(automata.check_input("00000000000001"));
    assert!(automata.check_input("010101010101010"));
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(6);
    automata.make_initial(0);
    automata.make_final(3);
    automata.make_final(4);
    automata.add_transition(0,1, "".to_string());
    automata.add_transition(0,4, "".to_string());
    automata.add_transition(1,2, "".to_string());
    automata.add_transition(1,1, "a".to_string());
    automata.add_transition(2,3, "a".to_string());
    automata.add_transition(2,2, "b".to_string());
    automata.add_transition(4,5, "a".to_string());
    automata.add_transition(5,4, "b".to_string());
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
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(5);
    automata.make_initial(0);
    automata.make_final(3);
    automata.make_final(4);
    automata.add_transition(0,1, "".to_string());
    automata.add_transition(0,2, "".to_string());
    automata.add_transition(1,3, "a".to_string());
    automata.add_transition(3,3, "a".to_string());
    automata.add_transition(2,4, "b".to_string());
    automata.add_transition(4,4, "b".to_string());
    assert!(!automata.is_deterministic());
    let deterministic_automata = automata.to_dfa();
    assert!(deterministic_automata.is_deterministic());
    assert!(!deterministic_automata.check_input(""));
    assert!(!deterministic_automata.check_input("ab"));
    assert!(!deterministic_automata.check_input("abaaaa"));
    assert!(deterministic_automata.check_input("a"));
    assert!(deterministic_automata.check_input("b"));
    assert!(deterministic_automata.check_input("bbbbbbbb"));
    assert!(deterministic_automata.check_input("aaaaaaaa"));
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(6);
    automata.make_initial(0);
    automata.make_final(3);
    automata.make_final(4);
    automata.add_transition(0,1, "".to_string());
    automata.add_transition(0,4, "".to_string());
    automata.add_transition(1,2, "".to_string());
    automata.add_transition(1,1, "a".to_string());
    automata.add_transition(2,3, "a".to_string());
    automata.add_transition(2,2, "b".to_string());
    automata.add_transition(4,5, "a".to_string());
    automata.add_transition(5,4, "b".to_string());
    let deterministic_automata = automata.to_dfa();
    assert!(!deterministic_automata.check_input("abbbbbbbbbb"));
    assert!(!deterministic_automata.check_input("b"));
    assert!(!deterministic_automata.check_input("aababababababa"));
    assert!(deterministic_automata.check_input("a"));
    assert!(deterministic_automata.check_input(""));
    assert!(deterministic_automata.check_input("abababababababab"));
    assert!(deterministic_automata.check_input("aaaaaabbbbbbbbba"));
    assert!(deterministic_automata.check_input("abbbbbbbbbbba"));
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(4);
    automata.make_initial(0);
    automata.make_final(3);
    automata.add_transition(0,1, "0".to_string());
    automata.add_transition(0,2, "1".to_string());
    automata.add_transition(1,1, "0".to_string());
    automata.add_transition(1,2, "1".to_string());
    automata.add_transition(1,3, "1".to_string());
    automata.add_transition(2,2, "1".to_string());
    automata.add_transition(2,1, "0".to_string());
    automata.add_transition(2,3, "0".to_string());
    automata.add_transition(3,3, "0".to_string());
    automata.add_transition(3,3, "1".to_string());
    let deterministic_automata = automata.to_dfa();
    assert!(!deterministic_automata.check_input(""));
    assert!(!deterministic_automata.check_input("0000000000"));
    assert!(!deterministic_automata.check_input("111111111"));
    assert!(!deterministic_automata.check_input("10x"));
    assert!(deterministic_automata.check_input("10"));
    assert!(deterministic_automata.check_input("01"));
    assert!(deterministic_automata.check_input("01111111111110"));
    assert!(deterministic_automata.check_input("00000000000001"));
    assert!(deterministic_automata.check_input("010101010101010"));
}

#[test]
fn minimize_test() {
    let mut bloated_automata = FiniteAutomata::new();
    bloated_automata.add_n_states(6);
    bloated_automata.make_initial(0);
    bloated_automata.make_final(2);
    bloated_automata.make_final(3);
    bloated_automata.make_final(4);
    bloated_automata.add_transition(0,1, "0".to_string());
    bloated_automata.add_transition(0,2, "1".to_string());
    bloated_automata.add_transition(1,0, "0".to_string());
    bloated_automata.add_transition(1,3, "1".to_string());
    bloated_automata.add_transition(3,4, "0".to_string());
    bloated_automata.add_transition(3,5, "1".to_string());
    bloated_automata.add_transition(2,5, "1".to_string());
    bloated_automata.add_transition(2,4, "0".to_string());
    bloated_automata.add_transition(4,4, "0".to_string());
    bloated_automata.add_transition(4,5, "1".to_string());
    bloated_automata.add_transition(5,5, "0".to_string());
    bloated_automata.add_transition(5,5, "1".to_string());
    let debloated_automata = bloated_automata.minimize();
    let states_by_id = debloated_automata.get_states_by_id_ref();
    assert_eq!(states_by_id.len(), 3);
    if let Some(initial_state_id) = debloated_automata.get_initial_state_id() {
        if let Some(state) = states_by_id.get(initial_state_id) {
            assert_eq!(state.label, [0,1].into_iter().collect());
        }
    }
    for state_id in debloated_automata.get_final_states() {
        if let Some(state) = states_by_id.get(state_id) {
            assert_eq!(state.label, [2,3,4].into_iter().collect());
        }
    }
    assert!(!debloated_automata.check_input("0000000000000"));
    assert!(!debloated_automata.check_input("1a0101010"));
    assert!(!debloated_automata.check_input("a"));
    assert!(!debloated_automata.check_input("11"));
    assert!(debloated_automata.check_input("00000000000001"));
    assert!(debloated_automata.check_input("1"));
    assert!(debloated_automata.check_input("00001"));
    assert!(debloated_automata.check_input("100000000000000000000000"));
    let mut bloated_automata = FiniteAutomata::new();
    bloated_automata.add_n_states(6);
    bloated_automata.make_initial(0);
    bloated_automata.make_final(3);
    bloated_automata.make_final(5);
    bloated_automata.add_transition(0,1, "0".to_string());
    bloated_automata.add_transition(0,3, "1".to_string());
    bloated_automata.add_transition(1,0, "0".to_string());
    bloated_automata.add_transition(1,3, "1".to_string());
    bloated_automata.add_transition(2,1, "0".to_string());
    bloated_automata.add_transition(2,4, "1".to_string());
    bloated_automata.add_transition(4,3, "1".to_string());
    bloated_automata.add_transition(4,3, "0".to_string());
    bloated_automata.add_transition(3,5, "0".to_string());
    bloated_automata.add_transition(3,5, "1".to_string());
    bloated_automata.add_transition(5,5, "1".to_string());
    bloated_automata.add_transition(5,5, "0".to_string());
    let debloated_automata = bloated_automata.minimize();
    assert_eq!(debloated_automata.get_states_by_id_ref().len(), 2);
    assert!(!debloated_automata.check_input(""));
    assert!(!debloated_automata.check_input("0"));
    assert!(!debloated_automata.check_input("000000000"));
    assert!(debloated_automata.check_input("1"));
    assert!(debloated_automata.check_input("01010101"));
    let mut bloated_automata = FiniteAutomata::new();
    bloated_automata.add_n_states(5);
    bloated_automata.make_initial(0);
    bloated_automata.make_final(4);
    bloated_automata.add_transition(0,2, "b".to_string());
    bloated_automata.add_transition(0,1, "a".to_string());
    bloated_automata.add_transition(1,1, "a".to_string());
    bloated_automata.add_transition(1,3, "b".to_string());
    bloated_automata.add_transition(2,2, "b".to_string());
    bloated_automata.add_transition(2,1, "a".to_string());
    bloated_automata.add_transition(3,4, "b".to_string());
    bloated_automata.add_transition(3,1, "a".to_string());
    bloated_automata.add_transition(4,2, "b".to_string());
    bloated_automata.add_transition(4,1, "a".to_string());
    let debloated_automata = bloated_automata.minimize();
    let states_by_id = debloated_automata.get_states_by_id_ref();
    assert_eq!(states_by_id.len(), 4);
    if let Some(initial_state_id) = debloated_automata.get_initial_state_id() {
        if let Some(state) = states_by_id.get(initial_state_id) {
            assert_eq!(state.label, [0,2].into_iter().collect());
        }
    }
    for state_id in debloated_automata.get_final_states() {
        if let Some(state) = states_by_id.get(state_id) {
            assert_eq!(state.label, [4].into_iter().collect());
        }
    }
    assert!(!debloated_automata.check_input("ab"));
    assert!(!debloated_automata.check_input("abbaaaaa"));
    assert!(!debloated_automata.check_input("abaaaaaa"));
    assert!(debloated_automata.check_input("abb"));
    assert!(debloated_automata.check_input("abbabb"));
    assert!(debloated_automata.check_input("abbbbaabb"));
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
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(6);
    automata.make_initial(0);
    automata.make_final(0);
    automata.make_final(3);
    for residue in 0..3u64 {
        let next = (residue + 1) % 3;
        automata.add_transition(residue, next, "b".to_string());
        automata.add_transition(residue + 3, next + 3, "b".to_string());
        automata.add_transition(residue, residue, "a".to_string());
        automata.add_transition(residue + 3, residue + 3, "a".to_string());
    }
    automata
}

fn build_partial_dfa() -> FiniteAutomata {
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(3);
    automata.make_initial(0);
    automata.make_final(1);
    automata.add_transition(0, 1, "a".to_string());
    automata.add_transition(1, 1, "a".to_string());
    automata.add_transition(2, 2, "b".to_string());
    automata
}

fn build_empty_language_dfa() -> FiniteAutomata {
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(2);
    automata.make_initial(0);
    automata.add_transition(0, 0, "a".to_string());
    automata.add_transition(0, 1, "b".to_string());
    automata.add_transition(1, 1, "a".to_string());
    automata.add_transition(1, 0, "b".to_string());
    automata
}

fn build_all_final_dfa() -> FiniteAutomata {
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(2);
    automata.make_initial(0);
    automata.make_final(0);
    automata.make_final(1);
    automata.add_transition(0, 1, "a".to_string());
    automata.add_transition(0, 0, "b".to_string());
    automata.add_transition(1, 0, "a".to_string());
    automata.add_transition(1, 1, "b".to_string());
    automata
}

fn reachable_states_of(automata: &FiniteAutomata) -> Vec<u64> {
    let initial = match automata.get_initial_state_id() {
        Some(id) => *id,
        None => return Vec::new(),
    };
    let mut visited: HashSet<u64> = HashSet::new();
    let mut stack = vec![initial];
    while let Some(id) = stack.pop() {
        if !visited.insert(id) {
            continue;
        }
        for string in automata.get_string_transitions() {
            if let Some(target) = automata.transition_function(id, string) {
                stack.push(target);
            }
        }
    }
    let sorted: BTreeSet<u64> = visited.into_iter().collect();
    sorted.into_iter().collect()
}

fn reference_class_count(automata: &FiniteAutomata) -> usize {
    let ids = reachable_states_of(automata);
    if ids.is_empty() {
        return 0;
    }
    let n_real = ids.len();
    let total = n_real + 1;
    let alphabet: Vec<String> = automata.get_string_transitions().iter().cloned().collect();
    let finals = automata.get_final_states();

    let delta = |i: usize, s: &str| -> usize {
        if i == n_real {
            return n_real;
        }
        match automata.transition_function(ids[i], s) {
            Some(target) => ids.iter().position(|&x| x == target).unwrap_or(n_real),
            None => n_real,
        }
    };

    let mut dist = vec![vec![false; total]; total];
    for i in 0..total {
        for j in 0..total {
            let i_final = i < n_real && finals.contains(&ids[i]);
            let j_final = j < n_real && finals.contains(&ids[j]);
            if i_final != j_final {
                dist[i][j] = true;
            }
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
                for s in &alphabet {
                    if dist[delta(i, s)][delta(j, s)] {
                        dist[i][j] = true;
                        changed = true;
                        break;
                    }
                }
            }
        }
    }

    let mut parent: Vec<usize> = (0..total).collect();
    fn find(parent: &mut Vec<usize>, mut x: usize) -> usize {
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
    let mut classes: HashSet<usize> = HashSet::new();
    for i in 0..n_real {
        classes.insert(find(&mut parent, i));
    }
    classes.len()
}

#[test]
fn minimize_matches_reference_test() {
    let fixtures = [
        build_mod3_redundant_dfa(),
        build_partial_dfa(),
        build_empty_language_dfa(),
        build_all_final_dfa(),
    ];
    for automata in fixtures.iter() {
        let minimized = automata.minimize();
        let expected_classes = reference_class_count(automata);
        assert_ne!(expected_classes, 0);
        assert_eq!(
            minimized.get_states_by_id_ref().len(),
            expected_classes,
            "minimized size differs from the table-filling reference"
        );
    }
}

#[test]
fn minimize_random_dfas_match_reference_test() {
    let mut rng = XorShift(0x9E3779B97F4A7C15);
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
        for input in short_ab_inputs() {
            assert_eq!(
                automata.check_input(&input),
                minimized.check_input(&input),
                "minimization changed the language on input {input:?}"
            );
        }
    }
}

#[test]
fn minimize_language_equivalence_test() {    let fixtures = [
        build_mod3_redundant_dfa(),
        build_partial_dfa(),
        build_empty_language_dfa(),
        build_all_final_dfa(),
    ];
    let alphabet = ['a', 'b'];
    let mut rng = XorShift(0x2545F4914F6CDD1D);
    let mut inputs: Vec<String> = Vec::new();
    for len in 0..=8usize {
        for mask in 0u64..(1u64 << len) {
            let s: String = (0..len)
                .map(|k| if (mask >> k) & 1 == 0 { 'a' } else { 'b' })
                .collect();
            inputs.push(s);
            if inputs.len() >= 200 {
                break;
            }
        }
        if inputs.len() >= 200 {
            break;
        }
    }
    while inputs.len() < 260 {
        let len = (rng.next() % 10) as usize;
        let s: String = (0..len)
            .map(|_| alphabet[(rng.next() % 2) as usize])
            .collect();
        inputs.push(s);
    }

    for automata in fixtures.iter() {
        let minimized = automata.minimize();
        for input in &inputs {
            let original_result = automata.check_input(input);
            let minimized_result = minimized.check_input(input);
            assert_eq!(
                original_result, minimized_result,
                "minimization changed the language on input {input:?}"
            );
        }
    }
}

#[test]
fn remove_state_cleans_bookkeeping_test() {
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(3);
    automata.make_initial(0);
    automata.make_final(1);
    automata.add_transition(0, 1, "a".to_string());
    automata.remove_state(1);
    assert!(automata.get_final_states().is_empty());
    automata.remove_state(0);
    assert_eq!(automata.get_initial_state_id(), &None);
}

#[test]
fn machine_accepts_matches_check_input_test() {
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(2);
    automata.make_initial(0);
    automata.make_final(1);
    automata.add_transition(0, 1, "a".to_string());
    automata.add_transition(1, 0, "a".to_string());
    assert_eq!(Machine::kind(&automata), MachineKind::Finite);
    for input in ["", "a", "aa", "aaa", "b"] {
        assert_eq!(
            automata.accepts(input),
            automata.check_input(input),
            "disagreement on input {input:?}"
        );
    }
}

#[test]
fn machine_validate_test() {
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(2);
    automata.make_initial(0);
    automata.make_final(1);
    automata.add_transition(0, 1, "a".to_string());
    assert!(automata.validate().is_ok());

    let mut automata = FiniteAutomata::new();
    automata.add_n_states(1);
    assert!(automata.validate().is_err());

    let mut automata = FiniteAutomata::new();
    automata.add_n_states(2);
    automata.make_initial(0);
    automata.add_transition(0, 1, "a".to_string());
    automata.get_states_by_id_mut_ref().remove(&1);
    assert!(automata.validate().is_err());
}

fn stepping_machine_fixtures() -> Vec<FiniteAutomata> {
    let mut fixtures = Vec::new();

    let mut automata = FiniteAutomata::new();
    automata.add_n_states(2);
    automata.make_initial(0);
    automata.make_final(1);
    automata.add_transition(0, 1, "a".to_string());
    automata.add_transition(0, 0, "b".to_string());
    automata.add_transition(1, 0, "a".to_string());
    automata.add_transition(1, 1, "b".to_string());
    fixtures.push(automata);

    let mut automata = FiniteAutomata::new();
    automata.add_n_states(4);
    automata.make_initial(0);
    automata.make_final(3);
    automata.add_transition(0, 1, "0".to_string());
    automata.add_transition(0, 2, "1".to_string());
    automata.add_transition(1, 1, "0".to_string());
    automata.add_transition(1, 2, "1".to_string());
    automata.add_transition(1, 3, "1".to_string());
    automata.add_transition(2, 2, "1".to_string());
    automata.add_transition(2, 1, "0".to_string());
    automata.add_transition(2, 3, "0".to_string());
    automata.add_transition(3, 3, "0".to_string());
    automata.add_transition(3, 3, "1".to_string());
    fixtures.push(automata);

    let mut automata = FiniteAutomata::new();
    automata.add_n_states(6);
    automata.make_initial(0);
    automata.make_final(3);
    automata.make_final(4);
    automata.add_transition(0, 1, "".to_string());
    automata.add_transition(0, 4, "".to_string());
    automata.add_transition(1, 2, "".to_string());
    automata.add_transition(1, 1, "a".to_string());
    automata.add_transition(2, 3, "a".to_string());
    automata.add_transition(2, 2, "b".to_string());
    automata.add_transition(4, 5, "a".to_string());
    automata.add_transition(5, 4, "b".to_string());
    fixtures.push(automata);

    let mut automata = FiniteAutomata::new();
    automata.add_n_states(3);
    automata.make_initial(0);
    automata.make_final(2);
    automata.add_transition(0, 1, "ε".to_string());
    automata.add_transition(1, 0, "".to_string());
    automata.add_transition(0, 2, "a".to_string());
    fixtures.push(automata);

    let mut automata = FiniteAutomata::new();
    automata.add_n_states(3);
    automata.make_initial(0);
    automata.make_final(1);
    automata.make_final(2);
    automata.add_transition(0, 1, "ab".to_string());
    automata.add_transition(0, 2, "aba".to_string());
    automata.add_transition(1, 1, "a".to_string());
    fixtures.push(automata);

    fixtures.push(build_mod3_redundant_dfa());
    fixtures.push(build_partial_dfa());
    fixtures.push(build_empty_language_dfa());
    fixtures.push(build_all_final_dfa());

    let mut automata = FiniteAutomata::new();
    automata.add_n_states(2);
    automata.make_final(1);
    automata.add_transition(0, 1, "a".to_string());
    fixtures.push(automata);

    fixtures
}

fn frontier_accepts(automata: &FiniteAutomata, input: &str) -> bool {
    let config = match automata.initial_configuration(input) {
        Some(config) => config,
        None => return false,
    };
    let mut visited: HashSet<FiniteConfiguration> = HashSet::from([config.clone()]);
    let mut alive: Vec<FiniteConfiguration> = vec![config];
    loop {
        if alive.iter().any(|config| automata.is_accepting(config)) {
            return true;
        }
        let mut next_alive: Vec<FiniteConfiguration> = Vec::new();
        for config in alive.drain(..) {
            for successor in automata.step_all(&config) {
                if visited.insert(successor.clone()) {
                    next_alive.push(successor);
                }
            }
        }
        if next_alive.is_empty() {
            return false;
        }
        alive = next_alive;
    }
}

fn deterministic_run_outcome(automata: &FiniteAutomata, input: &str) -> Result<bool, ()> {
    let mut config = match automata.initial_configuration(input) {
        Some(config) => config,
        None => return Ok(false),
    };
    let mut visited: HashSet<FiniteConfiguration> = HashSet::from([config.clone()]);
    loop {
        if automata.is_accepting(&config) {
            return Ok(true);
        }
        let successors = automata.step_all(&config);
        match successors.len() {
            0 => return Ok(false),
            1 => {
                let next = successors.into_iter().next().unwrap();
                if !visited.insert(next.clone()) {
                    return Ok(false);
                }
                config = next;
            },
            _ => return Err(()),
        }
    }
}

fn short_ab_inputs() -> Vec<String> {
    let mut inputs = Vec::new();
    for len in 0..=5usize {
        for mask in 0u64..(1u64 << len) {
            inputs.push(
                (0..len)
                    .map(|k| if (mask >> k) & 1 == 0 { 'a' } else { 'b' })
                    .collect(),
            );
        }
    }
    inputs
}

#[test]
fn frontier_stepping_matches_check_input_test() {
    let fixtures = stepping_machine_fixtures();
    let mut inputs = short_ab_inputs();
    inputs.extend([
        "aba".to_string(),
        "ababa".to_string(),
        "abab".to_string(),
        "abaa".to_string(),
    ]);
    for automata in fixtures.iter() {
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
    assert!(fixtures.len() >= 5, "expected several deterministic fixtures");
    let inputs = short_ab_inputs();
    for automata in fixtures.iter() {
        for input in &inputs {
            if let Ok(outcome) = deterministic_run_outcome(automata, input) { assert_eq!(
                outcome,
                automata.check_input(input),
                "deterministic run disagrees with check_input on {input:?}"
            ) }
        }
    }

    let multi_char = fixtures
        .iter()
        .find(|automata| {
            automata
                .get_string_transitions()
                .contains(&"aba".to_string())
        })
        .expect("the multi-character fixture is deterministic-flagged");
    assert!(multi_char.is_deterministic());
    assert_eq!(deterministic_run_outcome(multi_char, "aba"), Err(()));
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
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(4);
    automata.make_initial(0);
    automata.make_final(2);
    automata.add_transition(0, 1, "ε".to_string());
    automata.add_transition(0, 2, "ab".to_string());
    automata.add_transition(0, 3, "ax".to_string());
    automata.add_transition(1, 2, "b".to_string());

    let config = automata.initial_configuration("abc").unwrap();
    let successors = automata.step_all(&config);
    let described: Vec<(u64, &str)> = successors
        .iter()
        .map(|c| (c.state_id(), c.remaining_input().as_str()))
        .collect();
    assert_eq!(described, vec![(1, "abc"), (2, "c")]);

    let config = automata.initial_configuration("b").unwrap();
    let successors = automata.step_all(&config);
    let described: Vec<(u64, &str)> = successors
        .iter()
        .map(|c| (c.state_id(), c.remaining_input().as_str()))
        .collect();
    assert_eq!(described, vec![(1, "b")]);
    let successors = automata.step_all(&successors[0]);
    let described: Vec<(u64, &str)> = successors
        .iter()
        .map(|c| (c.state_id(), c.remaining_input().as_str()))
        .collect();
    assert_eq!(described, vec![(2, "")]);
    assert!(automata.is_accepting(&successors[0]));
}

#[test]
fn determinism_duplicates_and_removal_test() {
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(3);
    automata.make_initial(0);
    automata.add_transition(0, 1, "a".to_string());
    automata.add_transition(0, 1, "a".to_string());
    assert!(automata.is_deterministic());
    automata.add_transition(0, 2, "a".to_string());
    assert!(!automata.is_deterministic());
    automata.remove_transition(0, 2, "a");
    assert!(automata.is_deterministic());
    automata.add_transition(1, 2, "".to_string());
    assert!(!automata.is_deterministic(), "a blank label is an ε move");
    assert!(!automata.get_string_transitions().contains(""));
    automata.remove_transition(1, 2, "");
    assert!(automata.is_deterministic());
    automata.add_transition(1, 2, "b".to_string());
    automata.remove_state(2);
    assert!(!automata.get_string_transitions().contains("b"));
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
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(3);
    automata.make_initial(0);
    automata.make_final(2);
    let mut loops = vec!["a", "b", "c"];
    if labels_reversed {
        loops.reverse();
    }
    for label in loops {
        automata.add_transition(0, 0, label.to_string());
    }
    automata.add_transition(0, 1, "a".to_string());
    automata.add_transition(1, 2, "b".to_string());
    automata
}

#[test]
fn derived_machines_are_reproducible_test() {
    use crate::entity_file::write_finite_entity;
    let reference = ends_with_ab_nfa(false);
    let expected_dfa = write_finite_entity("m", &reference.to_dfa()).unwrap();
    let expected_minimized = write_finite_entity("m", &reference.to_dfa().minimize()).unwrap();
    for attempt in 0..32 {
        let automata = ends_with_ab_nfa(attempt % 2 == 1);
        let loop_labels: Vec<&String> = automata.get_states_by_id_ref()[&0]
            .iter_by_transition()
            .find(|(target, _)| **target == 0)
            .map(|(_, labels)| labels.iter().collect())
            .unwrap();
        assert_eq!(loop_labels, ["a", "b", "c"], "labels iterate in sorted order");
        let dfa = automata.to_dfa();
        assert_eq!(write_finite_entity("m", &dfa).unwrap(), expected_dfa, "NFA -> DFA numbering is stable");
        assert_eq!(dfa.get_initial_state_id(), &Some(0), "the DFA starts at q0");
        let minimized = dfa.minimize();
        assert_eq!(write_finite_entity("m", &minimized).unwrap(), expected_minimized, "minimized numbering is stable");
        assert_eq!(minimized.get_initial_state_id(), &Some(0), "the minimized DFA starts at q0");
    }
}
