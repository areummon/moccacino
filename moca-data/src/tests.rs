use std::collections::{HashMap, HashSet};
use crate::state::State;
use crate::finite_automaton::FiniteAutomaton;
use crate::state_machine::StateMachine;

#[cfg(test)]
mod tests {
    use super::*;
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
        for (k,v) in state.iter_by_transition() {
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
        let mut automata = FiniteAutomaton::new();
        automata.add_state();
        automata.add_state();
        assert_eq!(automata.iter_by_state().len(),2);
    }

    #[test]
    fn state_machine_add_transition_test() {
        let mut automata = FiniteAutomaton::new();
        automata.add_state();
        automata.add_transition(0, 1, "lovelyz".to_string());
        for (k,v) in automata.iter_by_state() {
            assert_ne!(1, v.iter_by_transition().len());
        }
        automata.add_state();
        automata.add_transition(0, 1, "lovelyz".to_string());
        automata.add_state();
        automata.add_transition(1, 2, "for you".to_string());
        let mut len = 0;
        let mut state_id = 1;
        for (k,v) in automata.iter_by_state() {
            for (x,y) in v.iter_by_transition() {
                if y.contains("lovelyz") || y.contains("for you") {
                    len += 1;
                    state_id += 1;
                }
            }
        }
        assert_eq!(len, 2);
    }

    #[test]
    fn modify_name_test() {
        let mut automata = FiniteAutomaton::new();
        automata.add_state();
        automata.modify_name(0, "jiyeon".to_string());
        for (k,v) in automata.iter_by_state() {
            assert_eq!(v.name, "jiyeon");
        }
    }

    #[test]
    fn state_machine_modify_input_test() {
        let mut automata = FiniteAutomaton::new();
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
        let mut automata = FiniteAutomaton::new();
        automata.add_state();
        automata.add_state();
        automata.remove_state(1);
        assert_eq!(automata.iter_by_state().len(), 1);
    }

    #[test]
    fn state_machine_remove_state() {
        let mut automata = FiniteAutomaton::new();
        automata.add_state();
        automata.add_state();
        automata.add_state();
        automata.add_transition(0,1,"badvillain".to_string());
        automata.add_transition(2,1,"badtitude".to_string());
        automata.remove_state(1);
        assert_eq!(automata.iter_by_state().len(),2);
        let mut len = 0;
        for (k,v) in automata.iter_by_state() {
            for (x,y) in v.iter_by_transition() {
                if *x == 1 {
                    len += 1;
                }
            }
        }
        assert_eq!(len,0);
    }

    /* Tests for the finite_automaton (DFA) module. */
    #[test]
    fn check_input_DFA_test() {
        // automata that recognizes strings with an odd number of 'a'
        let mut automata = FiniteAutomaton::new();
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
        let mut automata = FiniteAutomaton::new();
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
    fn check_input_NFA_test() {
        /* NDA that recognizes strings that contains 01 or 10 */
        let mut automata = FiniteAutomaton::new();
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
        /* NDA that recognizes strings of the form of λ+a(ba)*b+a*b*a */
        let mut automata = FiniteAutomaton::new();
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
        let mut automata = FiniteAutomaton::new();
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
        /* NDA that recognizes strings of the form of λ+a(ba)*b+a*b*a */
        // This should work for the previous reason for the previous automata.
        let mut automata = FiniteAutomaton::new();
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
        let mut automata = FiniteAutomaton::new();
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
        let mut bloated_automata = FiniteAutomaton::new();
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
        let mut states_by_id = debloated_automata.get_states_by_id_ref();
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
        let mut bloated_automata = FiniteAutomaton::new();
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
        let mut bloated_automata = FiniteAutomaton::new();
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
        let mut states_by_id = debloated_automata.get_states_by_id_ref();
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

}
