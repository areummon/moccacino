pub mod state;
mod finite_automaton;
mod tests;
mod state_machine;

use crate::state_machine::StateMachine;

fn main() {
    let mut automata = finite_automaton::FiniteAutomaton::new();
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
    automata.nfa_to_dfa();
    if automata.check_input(&mut "aaaaaaaaaaaaabbbbbbbbbbbbbbbbbba".to_string()) {
        println!("La cadena fue aceptada !!");
    } else {
        println!("La cadena NO fue aceptada ㅠㅠ");
    }
}
