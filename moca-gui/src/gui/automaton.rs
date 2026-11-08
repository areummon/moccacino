use moca_data::entity_file::{write_finite_entity, write_pushdown_entity, write_turing_entity};
use moca_data::finite_automata::FiniteAutomata;
use moca_data::pushdown_automata::PushdownAutomata;
use moca_data::state_machine::{Machine, StateMachine};
use moca_data::turing_machine::TuringMachine;

use crate::family::Family;

#[derive(Debug, Clone)]
pub(crate) enum Automaton {
    Finite(FiniteAutomata),
    Pushdown(PushdownAutomata),
    Turing(TuringMachine),
}

impl Automaton {
    pub(crate) fn family(&self) -> Family {
        match self {
            Automaton::Finite(_) => Family::Finite,
            Automaton::Pushdown(_) => Family::Pushdown,
            Automaton::Turing(_) => Family::Turing,
        }
    }

    pub(crate) fn structure(&self) -> &dyn StateMachine {
        match self {
            Automaton::Finite(finite) => finite,
            Automaton::Pushdown(pda) => pda,
            Automaton::Turing(turing) => turing,
        }
    }

    pub(crate) fn structure_mut(&mut self) -> &mut dyn StateMachine {
        match self {
            Automaton::Finite(finite) => finite,
            Automaton::Pushdown(pda) => pda,
            Automaton::Turing(turing) => turing,
        }
    }

    pub(crate) fn behavior(&self) -> &dyn Machine {
        match self {
            Automaton::Finite(finite) => finite,
            Automaton::Pushdown(pda) => pda,
            Automaton::Turing(turing) => turing,
        }
    }

    pub(crate) fn is_deterministic(&self) -> bool {
        self.structure().is_deterministic()
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        self.behavior().validate()
    }

    pub(crate) fn clear(&mut self) {
        match self {
            Automaton::Finite(finite) => *finite = FiniteAutomata::default(),
            Automaton::Pushdown(pda) => *pda = PushdownAutomata::new(pda.get_initial_stack_symbol().to_string()),
            Automaton::Turing(turing) => *turing = TuringMachine::new(turing.get_blank_symbol()),
        }
    }

    pub(crate) fn finite(&self, verb: &str) -> Result<&FiniteAutomata, String> {
        let holder = match self {
            Automaton::Finite(finite) => return Ok(finite),
            Automaton::Pushdown(_) => "a pushdown automaton",
            Automaton::Turing(_) => "a Turing machine",
        };
        Err(format!("Cannot {verb}: the tab holds {holder}, not a finite automaton."))
    }

    pub(crate) fn entity_text(&self, name: &str) -> Result<String, String> {
        match self {
            Automaton::Finite(finite) => write_finite_entity(name, finite),
            Automaton::Pushdown(pda) => write_pushdown_entity(name, pda),
            Automaton::Turing(turing) => write_turing_entity(name, turing),
        }
    }
}
