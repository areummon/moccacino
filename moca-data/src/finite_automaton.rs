use std::collections::HashMap;
use std::collections::hash_map::Iter;
use crate::state::{StateID, Input, State};
use crate::state_machine::StateMachine;

/* Structure that represent a finite automaton. */
pub struct FiniteAutomaton {
    states_by_id: HashMap<StateID, State>,
}

impl FiniteAutomaton {
    pub fn new() -> Self {
        FiniteAutomaton { states_by_id: HashMap::new(), }
    }
}

impl StateMachine for FiniteAutomaton {
    fn get_states_by_id(&mut self) -> &mut HashMap<StateID, State> {
        &mut self.states_by_id
    }
}
