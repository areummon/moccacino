pub mod state;
mod finite_automaton;
mod tests;

use finite_automaton::FiniteAutomaton;

fn main() {
    let mut automata = FiniteAutomaton::new();
    automata.add_state();
    automata.add_state();
    automata.add_state();
    automata.add_state();
    automata.make_initial(0);
    automata.make_final(3);
    automata.add_transition(0, 1, "sticky".to_string());
    automata.add_transition(1, 2, "kiss of life".to_string());
    automata.add_transition(2, 3, "mv".to_string());
    automata.add_transition(3, 0, "jun30".to_string());
    automata.add_transition(0, 4, "prueba".to_string());
    automata.add_transition(4, 4, "prueba".to_string());
    println!("{:?}", automata);
}
