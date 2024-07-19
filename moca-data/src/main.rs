pub mod state;
mod finite_automaton;
mod tests;
mod state_machine;

use crate::state_machine::StateMachine;

fn main() {
    let mut b3_automata = finite_automaton::FiniteAutomaton::new();
    b3_automata.add_n_states(5);
    b3_automata.make_initial(0);
    b3_automata.make_final(4);
    b3_automata.add_transition(0,2, "b".to_string());
    b3_automata.add_transition(0,1, "a".to_string());
    b3_automata.add_transition(1,1, "a".to_string());
    b3_automata.add_transition(1,3, "b".to_string());
    b3_automata.add_transition(2,2, "b".to_string());
    b3_automata.add_transition(2,1, "a".to_string());
    b3_automata.add_transition(3,4, "b".to_string());
    b3_automata.add_transition(3,1, "a".to_string());
    b3_automata.add_transition(4,2, "b".to_string());
    b3_automata.add_transition(4,1, "a".to_string());
    let b3_automata = b3_automata.minimize();
    finite_automaton::hopcroft_algorithm(&b3_automata); 
}

/*println!("id \t label \t\t transitions"); to_string of an automata to debug more easily
        for (id, state) in deterministic_automata.get_states_by_id_ref() {
            println!("{} \t {:?} \t\t {:?}", id, state.label, state.iter_by_transition());
        } 
        println!("initial state {:?} and final states {:?}", deterministic_automata.get_initial_state_id(), deterministic_automata.get_final_states()); */
