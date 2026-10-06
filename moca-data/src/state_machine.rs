use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Iter;
use crate::state::{StateID, Input, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineKind {
    Finite,
    Pushdown,
    Turing,
}

pub trait Machine {
    fn kind(&self) -> MachineKind;

    fn accepts(&self, input: &str) -> bool;

    fn validate(&self) -> Result<(), String>;
}

pub trait StateMachine {
    fn get_states_by_id_mut_ref(&mut self) -> &mut HashMap<StateID, State>;

    fn get_states_by_id_ref(&self) -> &HashMap<StateID, State>;

    fn markers_mut(&mut self) -> (&mut HashMap<StateID, State>, &mut Option<StateID>, &mut HashSet<StateID>);

    fn is_deterministic(&self) -> bool;

    fn get_final_states(&self) -> &HashSet<StateID>;

    fn get_initial_state_id(&self) -> &Option<StateID>;

    fn add_state(&mut self) -> StateID {
        let states_by_id = self.get_states_by_id_mut_ref();
        let mut id = states_by_id.len() as StateID;
        while states_by_id.contains_key(&id) {
            id += 1;
        }
        states_by_id.insert(id, State::new(format!("q{}", id)));
        id
    }

    fn add_state_with_id_label(&mut self, id: u64, label: &str) {
        self.get_states_by_id_mut_ref().insert(id, State::new(label.to_string()));
    }

    fn add_n_states(&mut self, n: u64) {
        for _ in 0..n {
            self.add_state();
        }
    }

    fn add_transition(&mut self, state_id1: StateID, state_id2: StateID, input: Input);

    fn modify_name(&mut self, state_id: StateID, new_name: String) {
        if let Some(state) = self.get_states_by_id_mut_ref().get_mut(&state_id) {
            state.name = new_name;
        }
    }

    fn modify_input(&mut self, state_id: StateID, state_transition_id: StateID,
                        old_input: &str, new_input: Input) {
        if let Some(state) = self.get_states_by_id_mut_ref().get_mut(&state_id) {
            state.modify_input(state_transition_id, old_input, new_input);
        }
    }

    fn forget_state(&mut self, _state_id: StateID) {}

    fn remove_state(&mut self, state_id: StateID) {
        let (states_by_id, initial, finals) = self.markers_mut();
        if states_by_id.remove(&state_id).is_none() {
            return;
        }
        for state in states_by_id.values_mut() {
            state.remove_state(state_id);
        }
        finals.remove(&state_id);
        if *initial == Some(state_id) {
            *initial = None;
        }
        self.forget_state(state_id);
    }

    fn remove_transition(&mut self, state_id: StateID, state_transition_id: StateID, input: &str) {
        if let Some(state) = self.get_states_by_id_mut_ref().get_mut(&state_id) {
            state.remove_transition(state_transition_id, input);
        }
    }

    fn make_initial(&mut self, state_id: StateID) {
        let (states_by_id, initial, _) = self.markers_mut();
        if !states_by_id.contains_key(&state_id) {
            return;
        }
        if let Some(old) = initial.and_then(|old_id| states_by_id.get_mut(&old_id)) {
            old.initial_flag = false;
        }
        if let Some(state) = states_by_id.get_mut(&state_id) {
            state.initial_flag = true;
        }
        *initial = Some(state_id);
    }

    fn make_final(&mut self, state_id: StateID) {
        let (states_by_id, _, finals) = self.markers_mut();
        if let Some(state) = states_by_id.get_mut(&state_id) {
            state.final_flag = true;
            finals.insert(state_id);
        }
    }

    fn iter_by_state(&mut self) -> Iter<'_, StateID, State> {
        self.get_states_by_id_mut_ref().iter()
    }
}
