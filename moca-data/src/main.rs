pub mod state;
mod finite_automaton;
mod tests;
mod state_machine;

use crate::state_machine::StateMachine;

fn main() {
    let mut automata = finite_automaton::FiniteAutomaton::new();
}
