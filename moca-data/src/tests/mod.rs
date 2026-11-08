mod entity_file_tests;
mod finite_automata_tests;
mod grammar_tests;
mod pushdown_automata_tests;
mod regex_tests;
mod turing_machine_tests;

use crate::finite_automata::FiniteAutomata;
use crate::pushdown_automata::{PushdownAutomata, DEFAULT_STACK};
use crate::state::StateID;
use crate::state_machine::StateMachine;
use crate::turing_machine::TuringMachine;

pub fn words(alphabet: &[char], max_len: usize) -> Vec<String> {
    let mut all = vec![String::new()];
    let mut frontier = vec![String::new()];
    for _ in 0..max_len {
        frontier = frontier
            .iter()
            .flat_map(|word| alphabet.iter().map(move |c| format!("{word}{c}")))
            .collect();
        all.extend(frontier.iter().cloned());
    }
    all
}

fn populate<M: StateMachine>(machine: &mut M, states: u64, finals: &[StateID], transitions: &[(StateID, StateID, &str)]) {
    machine.add_n_states(states);
    machine.make_initial(0);
    for id in finals {
        machine.make_final(*id);
    }
    for (from, to, label) in transitions {
        machine.add_transition(*from, *to, label.to_string());
    }
}

pub fn finite(states: u64, finals: &[StateID], transitions: &[(StateID, StateID, &str)]) -> FiniteAutomata {
    let mut automata = FiniteAutomata::new();
    populate(&mut automata, states, finals, transitions);
    automata
}

pub fn pushdown(states: u64, finals: &[StateID], transitions: &[(StateID, StateID, &str)]) -> PushdownAutomata {
    let mut pda = PushdownAutomata::new(DEFAULT_STACK.to_string());
    populate(&mut pda, states, finals, transitions);
    pda
}

pub fn turing(blank: char, states: u64, finals: &[StateID], transitions: &[(StateID, StateID, &str)]) -> TuringMachine {
    let mut tm = TuringMachine::new(blank);
    populate(&mut tm, states, finals, transitions);
    tm
}
