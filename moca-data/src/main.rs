pub mod state;
mod finite_automata;
mod pushdown_automata;
mod state_machine;
#[cfg(test)]
pub mod tests;

fn main() {}

/*println!("id \t label \t\t transitions"); to_string of an automata to debug more easily
        for (id, state) in deterministic_automata.get_states_by_id_ref() {
            println!("{} \t {:?} \t\t {:?}", id, state.label, state.iter_by_transition());
        } 
        println!("initial state {:?} and final states {:?}", deterministic_automata.get_initial_state_id(), deterministic_automata.get_final_states()); */
