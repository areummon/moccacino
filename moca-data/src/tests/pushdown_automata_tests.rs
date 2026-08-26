use crate::pushdown_automata::PushdownAutomata;
use crate::state_machine::StateMachine;

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
    // This automaton recognizes the language a^n b^n with n >= 0.
    let mut pushdown_automaton = PushdownAutomata::new("Z".to_string());
    pushdown_automaton.add_n_states(3);
    pushdown_automaton.make_initial(0);
    pushdown_automaton.make_final(2);
    pushdown_automaton.add_transition(0, 0, "a;Z/AZ".to_string());
    pushdown_automaton.add_transition(0, 0, "a;A/AA".to_string());
    pushdown_automaton.add_transition(0, 1, "ε".to_string());
    pushdown_automaton.add_transition(1, 1, "b;A/ε".to_string());
    pushdown_automaton.add_transition(1, 2, "ε;Z/Z".to_string());
    assert_eq!(pushdown_automaton.check_input(&mut "ab".to_string()), true);
    assert_eq!(pushdown_automaton.check_input(&mut "".to_string()), true);
    assert_eq!(pushdown_automaton.check_input(&mut "aabb".to_string()), true);
    assert_eq!(pushdown_automaton.check_input(&mut "aaabbb".to_string()), true);
    assert_eq!(pushdown_automaton.check_input(&mut "aab".to_string()), false);
    assert_eq!(pushdown_automaton.check_input(&mut "ba".to_string()), false);
    assert_eq!(pushdown_automaton.check_input(&mut "abb".to_string()), false);
    assert_eq!(pushdown_automaton.check_input(&mut "sy".to_string()), false);
}
