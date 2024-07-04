pub mod state;
mod finite_automaton;
mod tests;
mod state_machine;

use crate::state_machine::StateMachine;

fn main() {
    let mut automata = finite_automaton::FiniteAutomaton::new();
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
    if automata.check_input(&mut "100000000000000000000000".to_string()) {
        println!("La cadena fue aceptada !!");
    } else {
        println!("La cadena NO fue aceptada ㅠㅠ");
    }
}
