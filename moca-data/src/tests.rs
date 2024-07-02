use std::collections::{HashMap, HashSet};
use crate::finite_automaton::FiniteAutomaton;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state;

    /* Tests for the finite_automaton module. */
    #[test]
    fn add_state_test() {
        let mut automata = FiniteAutomaton::new();
        automata.add_state();
        automata.add_state();
        assert_eq!(automata.iter_by_state().len(),2);
    }

    #[test]
    fn add_transition_test() {
        let mut automata = FiniteAutomaton::new();
        automata.add_state();
        automata.add_transition(0, 1, "lovelyz".to_string());
        for (k,v) in automata.iter_by_transition() {
            assert_ne!(1, v.len());
        }
        automata.add_state();
        automata.add_transition(0, 1, "lovelyz".to_string());
        automata.add_state();
        automata.add_transition(1, 2, "for you".to_string());
        let mut len = 0;
        let mut state_id = 1;
        for (k,v) in automata.iter_by_transition() {
            if let Some(transitions) = v.get(&state_id) {
                if transitions.contains("lovelyz") ||
                    transitions.contains("for you") {
                        len += 1;
                    }
                state_id += 1;
            }
        }
        assert_eq!(len, 2);
    }

    #[test]
    fn modify_name_test() {
        let mut automata = FiniteAutomaton::new();
    }
}
