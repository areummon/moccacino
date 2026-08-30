use std::collections::{BTreeSet, HashSet};

use crate::state::State;
use crate::finite_automata::{FiniteAutomata, FiniteConfiguration};
use crate::state_machine::{Machine, MachineKind, StateMachine};
use crate::state;

/* Tests for the state module. */
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
    assert_eq!(true, flag1 && flag2);
}

/* function to count the number of transitions given inputs. */
fn count_transition(state: &State, in1: &str, in2: &str) -> u64 {
    let mut len = 0;
    for (_,v) in state.iter_by_transition() {
        if v.contains(in1) || v.contains(in2) {
            len += 1;
        }
    }
    len
}

/* Test to check if the hash have repeated transitions. */
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

/* Tests for the state_machine module.
 * This tests will only be for the finite automaton struct because
 * all the state machines implement the same trait and functions. */
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

/* Tests for the finite_automaton (DFA) module. */
#[test]
fn check_input_dfa_test() {
    // automata that recognizes strings with an odd number of 'a'
    let mut automata = FiniteAutomata::new();
    automata.add_state();
    automata.add_state();
    automata.add_transition(0,1, "a".to_string());
    automata.add_transition(0,0, "b".to_string());
    automata.add_transition(1,0, "a".to_string());
    automata.add_transition(1,1, "b".to_string());
    automata.make_final(1);
    automata.make_initial(0);
    assert_eq!(automata.check_input(&mut "abbbaabaaba".to_string()),false);
    assert_eq!(automata.check_input(&mut "bbbbbbabaaabba".to_string()),true);
    assert_eq!(automata.check_input(&mut "aaaaaaaaaaaaa".to_string()),true);
    /* automata that recognizes strings that have an # as the initial symbol
     * followed by numbers between 0,1 or 2 followed by at least three
     * character 'b' aparitions. */
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(7);
    automata.make_initial(0);
    automata.make_final(6);
    // reminder to add another function to add multiple transitions
    // to the same state to sipmplify this mess.
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
    assert_eq!(automata.check_input(&mut "adsf".to_string()), false);
    assert_eq!(automata.check_input(&mut "".to_string()), false);
    assert_eq!(automata.check_input(&mut "#1010201aabb".to_string()), false);
    assert_eq!(automata.check_input(&mut "1010201abbb".to_string()), false);
    assert_eq!(automata.check_input(&mut "#1010abbba".to_string()), true);
    assert_eq!(automata.check_input(&mut "#1010bbbbb".to_string()), true);
    assert_eq!(automata.check_input(&mut "#2222aaaaaaaaaaabbb".to_string()), true);
}

/* Tests for the finite_automaton (NFA) module. */
#[test]
fn check_input_nfa_test() {
    /* NDA that recognizes strings that contains 01 or 10 */
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
    assert_eq!(automata.check_input(&mut "".to_string()),false);
    assert_eq!(automata.check_input(&mut "0000000000".to_string()),false);
    assert_eq!(automata.check_input(&mut "111111111".to_string()),false);
    assert_eq!(automata.check_input(&mut "10x".to_string()),false);
    assert_eq!(automata.check_input(&mut "10".to_string()),true);
    assert_eq!(automata.check_input(&mut "01".to_string()),true);
    assert_eq!(automata.check_input(&mut "01111111111110".to_string()),true);
    assert_eq!(automata.check_input(&mut "00000000000001".to_string()),true);
    assert_eq!(automata.check_input(&mut "010101010101010".to_string()),true);
    /* NDA that recognizes strings of the form of ε+a(ba)*b+a*b*a */
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
    assert_eq!(automata.check_input(&mut "abbbbbbbbbb".to_string()),false);
    assert_eq!(automata.check_input(&mut "b".to_string()),false);
    assert_eq!(automata.check_input(&mut "aababababababa".to_string()),false);
    assert_eq!(automata.check_input(&mut "a".to_string()),true);
    assert_eq!(automata.check_input(&mut "".to_string()),true);
    assert_eq!(automata.check_input(&mut "abababababababab".to_string()),true);
    assert_eq!(automata.check_input(&mut "aaaaaabbbbbbbbba".to_string()),true);
    assert_eq!(automata.check_input(&mut "abbbbbbbbbbba".to_string()),true);
}

#[test]
fn to_dfa_test() {
    // The automata accepts any string of the form (a+ + b+)
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
    assert_eq!(automata.is_deterministic(), false);
    let deterministic_automata = automata.to_dfa();
    assert_eq!(deterministic_automata.is_deterministic(), true);
    // The language recognized by the dfa and the nfa must be the same.
    assert_eq!(deterministic_automata.check_input(&mut "".to_string()),false);
    assert_eq!(deterministic_automata.check_input(&mut "ab".to_string()),false);
    assert_eq!(deterministic_automata.check_input(&mut "abaaaa".to_string()),false);
    assert_eq!(deterministic_automata.check_input(&mut "a".to_string()),true);
    assert_eq!(deterministic_automata.check_input(&mut "b".to_string()),true);
    assert_eq!(deterministic_automata.check_input(&mut "bbbbbbbb".to_string()),true);
    assert_eq!(deterministic_automata.check_input(&mut "aaaaaaaa".to_string()),true);
    /* NDA that recognizes strings of the form of ε+a(ba)*b+a*b*a */
    // This should work for the previous reason for the previous automata.
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
    assert_eq!(deterministic_automata.check_input(&mut "abbbbbbbbbb".to_string()),false);
    assert_eq!(deterministic_automata.check_input(&mut "b".to_string()),false);
    assert_eq!(deterministic_automata.check_input(&mut "aababababababa".to_string()),false);
    assert_eq!(deterministic_automata.check_input(&mut "a".to_string()),true);
    assert_eq!(deterministic_automata.check_input(&mut "".to_string()),true);
    assert_eq!(deterministic_automata.check_input(&mut "abababababababab".to_string()),true);
    assert_eq!(deterministic_automata.check_input(&mut "aaaaaabbbbbbbbba".to_string()),true);
    assert_eq!(deterministic_automata.check_input(&mut "abbbbbbbbbbba".to_string()),true);
    /* NDA that recognizes strings that contains 01 or 10 */
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
    assert_eq!(deterministic_automata.check_input(&mut "".to_string()),false);
    assert_eq!(deterministic_automata.check_input(&mut "0000000000".to_string()),false);
    assert_eq!(deterministic_automata.check_input(&mut "111111111".to_string()),false);
    assert_eq!(deterministic_automata.check_input(&mut "10x".to_string()),false);
    assert_eq!(deterministic_automata.check_input(&mut "10".to_string()),true);
    assert_eq!(deterministic_automata.check_input(&mut "01".to_string()),true);
    assert_eq!(deterministic_automata.check_input(&mut "01111111111110".to_string()),true);
    assert_eq!(deterministic_automata.check_input(&mut "00000000000001".to_string()),true);
    assert_eq!(deterministic_automata.check_input(&mut "010101010101010".to_string()),true);
}

#[test]
fn minimize_test() {
    // This automata is used as an example in https://en.wikipedia.org/wiki/DFA_minimization
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
    assert_eq!(debloated_automata.check_input(&mut "0000000000000".to_string()),false);
    assert_eq!(debloated_automata.check_input(&mut "1a0101010".to_string()),false);
    assert_eq!(debloated_automata.check_input(&mut "a".to_string()),false);
    assert_eq!(debloated_automata.check_input(&mut "11".to_string()),false);
    assert_eq!(debloated_automata.check_input(&mut "00000000000001".to_string()),true);
    assert_eq!(debloated_automata.check_input(&mut "1".to_string()),true);
    assert_eq!(debloated_automata.check_input(&mut "00001".to_string()),true);
    assert_eq!(debloated_automata.check_input(&mut "100000000000000000000000".to_string()),true);
    // This automata is used as an example in https://www.javatpoint.com/minimization-of-dfa
    // The example in the webpage has a useless state q1, therefore only 2 states are needed.
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
    assert_eq!(debloated_automata.check_input(&mut "".to_string()),false);
    assert_eq!(debloated_automata.check_input(&mut "0".to_string()),false);
    assert_eq!(debloated_automata.check_input(&mut "000000000".to_string()),false);
    assert_eq!(debloated_automata.check_input(&mut "1".to_string()),true);
    assert_eq!(debloated_automata.check_input(&mut "01010101".to_string()),true);
    // This automata is used as an example in https://www.gatevidyalay.com/minimization-of-dfa-minimize-dfa-example/
    // problem 01
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
    assert_eq!(debloated_automata.check_input(&mut "ab".to_string()),false);
    assert_eq!(debloated_automata.check_input(&mut "abbaaaaa".to_string()),false);
    assert_eq!(debloated_automata.check_input(&mut "abaaaaaa".to_string()),false);
    assert_eq!(debloated_automata.check_input(&mut "abb".to_string()),true);
    assert_eq!(debloated_automata.check_input(&mut "abbabb".to_string()),true);
    assert_eq!(debloated_automata.check_input(&mut "abbbbaabb".to_string()),true);
}

/* ---------- Minimization verification (Hopcroft vs an independent reference) ---------- */

/* Deterministic pseudo-random generator (xorshift64), so the generated input
 * battery is reproducible without external dependencies. */
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

/* Builds the language { w in {a,b}* : number of b's mod 3 == 0 } with every
 * residue state duplicated, so the minimal DFA has 3 states but 4 classes
 * would be found if duplicates were wrongly kept apart. */
fn build_mod3_redundant_dfa() -> FiniteAutomata {
    // ids: r0=0, r1=1, r2=2, d0=3, d1=4, d2=5 (the d* copy mirrors the r* one)
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

/* Partial DFA: b is simply not defined anywhere. The reachable part accepts a+,
 * and an unreachable junk state exists to exercise unreachable removal. */
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

/* Returns the states reachable from the initial state, sorted by id. */
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

/* Independent reference implementation: Moore's table-filling algorithm over
 * the reachable states, with undefined transitions compared against each other
 * and against real targets via an implicit sink index (the same convention the
 * Hopcroft implementation uses). Returns the number of equivalence classes. */
fn reference_class_count(automata: &FiniteAutomata) -> usize {
    let ids = reachable_states_of(automata);
    if ids.is_empty() {
        return 0;
    }
    let n_real = ids.len();
    let total = n_real + 1; // last index is the implicit sink
    let alphabet: Vec<String> = automata.get_string_transitions().iter().cloned().collect();
    let finals = automata.get_final_states();

    let delta = |i: usize, s: &str| -> usize {
        if i == n_real {
            return n_real; // the implicit sink loops to itself on every symbol
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

    // Union-find over the indistinguishable pairs to count classes.
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
fn minimize_language_equivalence_test() {    let fixtures = [
        build_mod3_redundant_dfa(),
        build_partial_dfa(),
        build_empty_language_dfa(),
        build_all_final_dfa(),
    ];
    // A fixed battery of hand-picked strings plus a reproducible random one.
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
            let original_result = automata.check_input(&mut input.clone());
            let minimized_result = minimized.check_input(&mut input.clone());
            assert_eq!(
                original_result, minimized_result,
                "minimization changed the language on input {:?}",
                input
            );
        }
    }
}

/* Removing a state must also clean every registry that mentions it, otherwise
 * ghost ids leak into final_states/initial_state_id and corrupt later
 * transformations (this is what used to break minimize on partial automata). */
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

/* ---------- Machine behavioral trait ---------- */

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
        let mut owned = input.to_string();
        assert_eq!(
            automata.accepts(input),
            automata.check_input(&mut owned),
            "disagreement on input {:?}",
            input
        );
    }
}

#[test]
fn machine_validate_test() {
    // Valid automaton.
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(2);
    automata.make_initial(0);
    automata.make_final(1);
    automata.add_transition(0, 1, "a".to_string());
    assert!(automata.validate().is_ok());

    // Missing initial state.
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(1);
    assert!(automata.validate().is_err());

    // Dangling transition target.
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(2);
    automata.make_initial(0);
    automata.add_transition(0, 1, "a".to_string());
    automata.get_states_by_id_mut_ref().remove(&1);
    assert!(automata.validate().is_err());
}

/* ---------- Configuration stepping (mirrors the GUI run panel) ---------- */

fn stepping_machine_fixtures() -> Vec<FiniteAutomata> {
    let mut fixtures = Vec::new();

    // DFA over {a,b}: odd number of 'a's.
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(2);
    automata.make_initial(0);
    automata.make_final(1);
    automata.add_transition(0, 1, "a".to_string());
    automata.add_transition(0, 0, "b".to_string());
    automata.add_transition(1, 0, "a".to_string());
    automata.add_transition(1, 1, "b".to_string());
    fixtures.push(automata);

    // NFA: strings containing 01 or 10.
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

    // ε-NFA: ε + a(ba)*b + a*b*a.
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

    // ε-cycle with a consuming escape: 0 ⇄ 1 via ε, 0 -a-> 2 (final).
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(3);
    automata.make_initial(0);
    automata.make_final(2);
    automata.add_transition(0, 1, "ε".to_string());
    automata.add_transition(1, 0, "".to_string());
    automata.add_transition(0, 2, "a".to_string());
    fixtures.push(automata);

    // Multi-character labels with a shared prefix: "ab" and "aba" both lead
    // to final states, so every matching prefix must be explored.
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(3);
    automata.make_initial(0);
    automata.make_final(1);
    automata.make_final(2);
    automata.add_transition(0, 1, "ab".to_string());
    automata.add_transition(0, 2, "aba".to_string());
    automata.add_transition(1, 1, "a".to_string());
    fixtures.push(automata);

    // Partial DFA (b undefined) and the other minimization fixtures.
    fixtures.push(build_mod3_redundant_dfa());
    fixtures.push(build_partial_dfa());
    fixtures.push(build_empty_language_dfa());
    fixtures.push(build_all_final_dfa());

    // Automaton without an initial state.
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(2);
    automata.make_final(1);
    automata.add_transition(0, 1, "a".to_string());
    fixtures.push(automata);

    fixtures
}

/* The exact frontier exploration the GUI's nondeterministic view runs:
 * verdict before expanding, visited dedup over configurations, empty
 * frontier = rejected. */
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

/* The exact single-branch run the GUI's deterministic view runs: acceptance
 * on arrival, one successor per step, revisiting a configuration is the same
 * ε-cycle dead end `check_input`'s visited set reports. An ambiguity (several
 * successors) surfaces as Err instead of guessing a branch. */
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

/* All strings over {a,b} up to length 5, in a deterministic order. */
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
            let mut owned = input.clone();
            assert_eq!(
                frontier_accepts(automata, input),
                automata.check_input(&mut owned),
                "frontier stepping disagrees with check_input on {:?}",
                input
            );
        }
    }
}

#[test]
fn deterministic_run_matches_check_input_test() {
    // Only unambiguous machines can drive the deterministic view. The
    // determinism flag reacts to duplicate labels but not to prefix-
    // overlapping multi-character ones, so genuinely ambiguous machines can
    // stay flagged deterministic; the run must surface them as Err rather
    // than silently pick a branch.
    let fixtures: Vec<FiniteAutomata> = stepping_machine_fixtures()
        .into_iter()
        .filter(|automata| automata.is_deterministic())
        .collect();
    assert!(fixtures.len() >= 5, "expected several deterministic fixtures");
    let inputs = short_ab_inputs();
    for automata in fixtures.iter() {
        for input in &inputs {
            let mut owned = input.clone();
            match deterministic_run_outcome(automata, input) {
                Ok(outcome) => assert_eq!(
                    outcome,
                    automata.check_input(&mut owned),
                    "deterministic run disagrees with check_input on {:?}",
                    input
                ),
                // Ambiguous step: the deterministic view refuses to guess a
                // branch instead of lying about the outcome.
                Err(()) => {},
            }
        }
    }

    // The multi-character fixture stays flagged deterministic (the flag only
    // reacts to duplicate labels) but must surface the "ab"/"aba" overlap as
    // an ambiguity instead of picking a branch.
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
    // Without an initial state no configuration exists.
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

    // Consuming "a" leaves "b": a final state alone is not acceptance.
    let stepped = automata.step_all(&config);
    assert_eq!(stepped.len(), 1);
    assert_eq!(stepped[0].state_id(), 1);
    assert_eq!(stepped[0].remaining_input(), "b");
    assert!(!automata.is_accepting(&stepped[0]));
    assert!(automata.step_all(&stepped[0]).is_empty(), "'b' is not consumable from state 1");

    // With the input fully consumed, the final state accepts.
    let config = automata.initial_configuration("a").unwrap();
    let stepped = automata.step_all(&config);
    assert_eq!(stepped.len(), 1);
    assert!(automata.is_accepting(&stepped[0]), "state 1 is final and the input is consumed");
}

#[test]
fn step_all_epsilon_and_prefixes_test() {
    // 0 -ε-> 1, 0 -"ab"-> 2, 0 -"ax"-> 3, 1 -"b"-> 2 (final).
    let mut automata = FiniteAutomata::new();
    automata.add_n_states(4);
    automata.make_initial(0);
    automata.make_final(2);
    automata.add_transition(0, 1, "ε".to_string());
    automata.add_transition(0, 2, "ab".to_string());
    automata.add_transition(0, 3, "ax".to_string());
    automata.add_transition(1, 2, "b".to_string());

    // ε keeps the input; matching prefixes are consumed; non-matching
    // labels are dropped. Sorted by (state id, remaining input).
    let config = automata.initial_configuration("abc").unwrap();
    let successors = automata.step_all(&config);
    let described: Vec<(u64, &str)> = successors
        .iter()
        .map(|c| (c.state_id(), c.remaining_input().as_str()))
        .collect();
    assert_eq!(described, vec![(1, "abc"), (2, "c")]);

    // With input "b" only the ε move applies; the next step consumes the
    // matching label from the ε-reached state and lands on the final state.
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
