use bimap::hash::BiHashMap;
use std::fmt;
use crate::state::State;

/* Structure that represent a finite automaton. */
pub struct FiniteAutomaton<'a> {
    states: BiHashMap<&'a str, State<'a>>,
}

impl<'a> FiniteAutomaton<'a> {
    pub fn new() -> Self {
        FiniteAutomaton { states: BiHashMap::new(), }
    }
    
    /* The name is assigned automatically. */
    pub fn add_state(&mut self) {
        let state_name = format!("q{}",self.states.len());
        self.states.insert(state_name, State::new(&state_name));
    }
}
