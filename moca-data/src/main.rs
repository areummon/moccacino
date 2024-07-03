pub mod state;
mod finite_automaton;
mod tests;
mod state_machine;

use crate::state_machine::StateMachine;

fn main() {
    let mut automata = finite_automaton::FiniteAutomaton::new();
    automata.add_state();
    automata.add_state();
    automata.add_transition(0,1, "a".to_string());
    automata.add_transition(0,0, "b".to_string());
    automata.add_transition(1,0, "a".to_string());
    automata.add_transition(1,1, "b".to_string());
    automata.make_final(1);
    automata.make_initial(0);
    let cadena = "baabaabbbbbbabbb";
    if automata.check_input(&mut "baabaabbbbabbbbb".to_string()) {
        println!("La cadena {} fue aceptada!", cadena);
    } else {
        println!("La cadena {} fue rechazada!", cadena);
    }
}
