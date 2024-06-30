use std::collections::{HashMap, HashSet};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state;

    /* Tests for the state module. */
    #[test]
    fn add_transition_test() {
        let mut state = state::State::new("name");
        state.add_transition("one", "a".to_string());
        state.add_transition("one", "b".to_string());
        state.add_transition("two", "c".to_string());
        state.add_transition("two", "d".to_string());
        let mut state1_len = 0;
        let mut state2_len = 0;
        match state.get_transitions("one") {
            Some(set) => {state1_len = count_transition(set, "a", "b");}
            None => assert!(false, "The function does not add transitions"),
        }
        assert_eq!(state1_len,2);
        match state.get_transitions("two") {
            Some(set) => {state2_len = count_transition(set, "c", "d");}
            None => assert!(false, "The function does not add transitions"),
        }
        assert_eq!(state2_len,2);
    }

    /* function to count the number of transitions given inputs. */
    fn count_transition(set: &HashSet<String>, in1: &str, in2: &str) -> u32 {
        let mut state_len = 0;
        for i in set {
            if *i == in1.to_string() || *i == in2.to_string() {
                state_len += 1;
            } 
        }
        state_len
    }

    /* Test to check if the hash have repeated transitions. */
    #[test]
    fn add_transition_repeated_elements_test() {
        let mut state = state::State::new("name");
        state.add_transition("q2","input".to_string());
        state.add_transition("q2","input".to_string());
        state.add_transition("q2","input".to_string());
        match state.get_transitions("q2") {
            Some(set) => assert_eq!(set.len(),1),
            None => assert!(false,"The function does not add transitions"),
        }
    }

    #[test]
    fn remove_transition_test() {
        let mut state = state::State::new("assemble");
        state.add_transition("baddest", "worst".to_string());
        state.add_transition("baddest", "best".to_string());
        state.remove_transition("baddest","worst".to_string());
        if let Some(set) = state.get_transitions("baddest") {
            assert_eq!(set.len(),1);
        }
        state.remove_transition("baddest", "best".to_string());
        if let Some(set) = state.get_transitions("baddest") {
            assert_eq!(set.len(),0);
        }
    }

    #[test]
    fn modify_input_test() {
        let mut state = state::State::new("shion");
        state.add_transition("pan", "cuerda".to_string());
        state.modify_input("pan", "cuerda".to_string(), "cuerno".to_string());
        if let Some(set) = state.get_transitions("pan") {
            let transitions_number = count_transition(set, "cuerno", "");
            assert_eq!(transitions_number, 1);
        }
    }

    #[test]
    fn update_test() {
        let mut state = state::State::new("illit");
        state.add_transition("super real me", "magnetic".to_string());
        state.update("super real me", "the red");
        if let Some(set) = state.get_transitions("the red") {
            let transitions_number = count_transition(set, "magnetic", "");
            assert_eq!(transitions_number, 1);
        }
    }
}
