use std::collections::{HashMap, HashSet};
use crate::state::State;

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
}
