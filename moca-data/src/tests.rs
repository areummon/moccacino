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

    /* Tests for the finite_automaton module. */
    #[test]
    fn check_input_test() {
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
}
