use crate::state::State;
use crate::pushdown_automata::{self, PushdownAutomata};
use crate::state_machine::StateMachine;
use crate::state;

/* Several methods and functions are the same as the finite automaton
 * So the tests are only for the different methods. */

#[test]
fn check_input_dpa_test() {
    // This automata is use as an example in https://en.wikipedia.org/wiki/Pushdown_automaton#Example
    let mut pushdown_automata = PushdownAutomata::new("Z".to_string());
    pushdown_automata.add_n_states(3);
    pushdown_automata.make_initial(0);
    pushdown_automata.make_final(2);
    pushdown_automata.add_transition(0, 0, "0;Z/AZ".to_string());
    pushdown_automata.add_transition(0, 0, "0;A/AA".to_string());
    pushdown_automata.add_transition(0, 1, "ε".to_string());
    pushdown_automata.add_transition(1, 1, "1;A/ε".to_string());
    pushdown_automata.add_transition(1, 2, "ε;Z/Z".to_string());
    assert_eq!(pushdown_automata.check_input(&mut "0".to_string()), false);
    assert_eq!(pushdown_automata.check_input(&mut "001".to_string()), false);
    assert_eq!(pushdown_automata.check_input(&mut "001111".to_string()), false);
    assert_eq!(pushdown_automata.check_input(&mut "0001111".to_string()), false);
    assert_eq!(pushdown_automata.check_input(&mut "sy".to_string()), false);
    assert_eq!(pushdown_automata.check_input(&mut "01".to_string()), true);
    assert_eq!(pushdown_automata.check_input(&mut "00001111".to_string()), true);
    assert_eq!(pushdown_automata.check_input(&mut "".to_string()), true);
    // This automata is used as an example Q) in https://www.geeksforgeeks.org/construct-pushdown-automata-given-languages/
    let mut pushdown_automaton = PushdownAutomata::new("Z".to_string());
    pushdown_automaton.add_n_states(5);
    pushdown_automaton.make_initial(0);
    pushdown_automaton.make_final(2);
    pushdown_automaton.add_transition(0, 1, "ε;Z/c".to_string());
    pushdown_automaton.add_transition(1, 2, ";c/Z".to_string());
    pushdown_automaton.add_transition(1, 3, "a;Z/a".to_string());
    pushdown_automaton.add_transition(1, 4, "b;Z/b".to_string());
    pushdown_automaton.add_transition(3, 3, "a;Z/a".to_string());
    pushdown_automaton.add_transition(3, 3, "b;a/Z".to_string());
    pushdown_automaton.add_transition(3, 4, "b;c/bc".to_string());
    pushdown_automaton.add_transition(1, 4, "b;Z/b".to_string());
    pushdown_automaton.add_transition(4, 4, "a;b/Z".to_string());
    pushdown_automaton.add_transition(4, 4, "b;Z/b".to_string());
    pushdown_automaton.add_transition(4, 3, "a;c/ac".to_string());
    pushdown_automaton.add_transition(3, 2, ";c/Z".to_string());
    pushdown_automaton.add_transition(4, 2, ";c/Z".to_string());
    assert_eq!(pushdown_automaton.check_input(&mut "ab".to_string()), true);
}
