use std::collections::HashMap;
use indexmap::IndexSet;

use crate::state_machine;

use moca_data::finite_automata::FiniteAutomata;
use moca_data::state_machine::StateMachine;

#[derive(Default)]
pub(crate) struct Tab {
    pub(crate) state_machine: state_machine::State,
    pub(crate) transitions: HashMap<(usize, usize), IndexSet<String>>,
    pub(crate) states: Vec<state_machine::StateNode>,
    pub(crate) state_id_to_index: HashMap<usize, usize>,
    pub(crate) machine: FiniteAutomata,
    pub(crate) initial_state: Option<usize>,
    pub(crate) final_states: std::collections::HashSet<usize>,
    pub(crate) editing_state: Option<usize>,
    pub(crate) editing_transition: Option<usize>,
    pub(crate) edit_text: String,
    pub(crate) operations_menu_open: bool,
    pub(crate) check_input_dialog_open: bool,
    pub(crate) check_input_text: String,
    pub(crate) check_result_popup_open: bool,
    pub(crate) check_input_result: Option<bool>,
    pub(crate) deletion_mode: bool,
    pub(crate) name: String,
    pub(crate) pending_transition: Option<(usize, usize, iced::Point, iced::Point)>,
    pub(crate) pending_transition_label: String,
    pub(crate) pending_transition_dialog_open: bool,
    pub(crate) editing_transition_pair: Option<(usize, usize)>,
    pub(crate) editing_transition_labels: Vec<String>,
    pub(crate) editing_transition_dialog_open: bool,
    pub(crate) editing_transition_label_inputs: Vec<String>,
}

impl Tab {
    pub(crate) fn new() -> Self {
        let mut tab = Self::default();
        tab.state_machine.reset_id_counter();
        tab.machine = FiniteAutomata::default();
        tab.name = "Machine".to_string();
        tab.transitions = HashMap::new();
        tab
    }

    pub(crate) fn new_with_name(name: String) -> Self {
        let mut tab = Self::new();
        tab.name = name;
        tab.transitions = HashMap::new();
        tab
    }

    pub(crate) fn set_initial_state(&mut self, state_id: usize) {
        if self.initial_state == Some(state_id) {
            self.initial_state = None;
        } else {
            self.initial_state = Some(state_id);
        }
        self.state_machine.request_redraw();
    }

    pub(crate) fn toggle_final_state(&mut self, state_id: usize) {
        if self.final_states.contains(&state_id) {
            self.final_states.remove(&state_id);
        } else {
            self.final_states.insert(state_id);
        }
        self.state_machine.request_redraw();
    }

    pub(crate) fn sync_gui_to_finite_automata(&mut self) {
        self.machine.clear();

        // Add all states
        for state_node in &self.states {
            self.machine.add_state_with_id_label(state_node.id as u64, state_node.label);
        }

        // Add all transitions (multi-label)
        for (&(from, to), labels) in &self.transitions {
            for label in labels {
                let label = if label.trim().is_empty() || label == "ε" { "ε" } else { label };
                self.machine.add_transition(from as u64, to as u64, label.to_string());
            }
        }

        // Set final states
        for &state_id in &self.final_states {
            self.machine.make_final(state_id as u64);
        }

        // Set initial state
        if let Some(initial_id) = self.initial_state {
            self.machine.make_initial(initial_id as u64);
        }
    }

    pub(crate) fn load_finite_automata_to_gui(&mut self) {
        self.states.clear();
        self.transitions.clear();
        self.state_id_to_index.clear();
        self.initial_state = None;
        self.final_states.clear();

        let mut max_id_after_load = 0;
        for (id, state) in self.machine.get_states_by_id_ref() {
            let state_node = state_machine::StateNode::new(
                *id as usize,
                iced::Point::new(100.0, 100.0),
                30.0,
                Box::leak(state.name.clone().into_boxed_str())
            );
            let index = self.states.len();
            self.states.push(state_node);
            self.state_id_to_index.insert(*id as usize, index);
            max_id_after_load = max_id_after_load.max(*id as usize);
        }

        self.state_machine.next_id = max_id_after_load + 1;

        // Add all transitions (multi-label)
        for (from_id, state) in self.machine.get_states_by_id_ref() {
            for (to_id, inputs) in state.iter_by_transition() {
                let key = (*from_id as usize, *to_id as usize);
                let entry = self.transitions.entry(key).or_insert_with(indexmap::IndexSet::new);
                for label in inputs {
                    let label = if label.trim().is_empty() || label == "ε" { "ε".to_string() } else { label.clone() };
                    entry.insert(label);
                }
            }
        }

        if let Some(initial_id) = self.machine.get_initial_state_id() {
            if let Some(state) = self.states.iter()
                .find(|s| s.id == *initial_id as usize) {
                self.initial_state = Some(state.id);
            }
        }

        for final_id in self.machine.get_final_states() {
            if let Some(state) = self.states.iter()
                .find(|s| s.id == *final_id as usize) {
                self.final_states.insert(state.id);
            }
        }

        if self.initial_state.is_some() {
            Self::apply_tree_layout_to_tab(self);
        } else {
            Self::apply_grid_layout_to_tab(self);
        }

        self.state_machine.request_redraw();
    }

    fn apply_tree_layout_to_tab(active_tab: &mut Tab) {
        use std::collections::{HashMap, HashSet};

        let mut children_map: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut parent_map: HashMap<usize, usize> = HashMap::new();
        let mut all_ids: HashSet<usize> = HashSet::new();
        for state in &active_tab.states {
            all_ids.insert(state.id);
        }
        for (&(from, to), _) in &active_tab.transitions {
            if !parent_map.contains_key(&to) {
                children_map.entry(from).or_default().push(to);
                parent_map.insert(to, from);
            }
        }

        let root_id = match active_tab.initial_state {
            Some(id) => id,
            None => return,
        };

        let mut x_counter = 0.0;
        let x_spacing = 90.0;
        let y_spacing = 120.0;
        let start_x = 100.0;
        let start_y = 100.0;

        fn assign_positions(
            node_id: usize,
            depth: usize,
            children_map: &HashMap<usize, Vec<usize>>,
            state_map: &mut HashMap<usize, &mut state_machine::StateNode>,
            x_counter: &mut f32,
            x_spacing: f32,
            y_spacing: f32,
            start_x: f32,
            start_y: f32,
        ) -> f32 {
            let children = children_map.get(&node_id);
            let x;
            if let Some(children) = children {
                let mut child_xs = Vec::new();
                for &child_id in children {
                    let cx = assign_positions(child_id, depth + 1, children_map, state_map, x_counter, x_spacing, y_spacing, start_x, start_y);
                    child_xs.push(cx);
                }
                if !child_xs.is_empty() {
                    x = (child_xs[0] + child_xs[child_xs.len() - 1]) / 2.0;
                } else {
                    x = *x_counter;
                    *x_counter += x_spacing;
                }
            } else {
                x = *x_counter;
                *x_counter += x_spacing;
            }
            if let Some(state) = state_map.get_mut(&node_id) {
                state.position = iced::Point::new(start_x + x, start_y + (depth as f32) * y_spacing);
            }
            x
        }

        let mut state_map: HashMap<usize, &mut state_machine::StateNode> =
            active_tab.states.iter_mut().map(|s| (s.id, s)).collect();
        assign_positions(
            root_id,
            0,
            &children_map,
            &mut state_map,
            &mut x_counter,
            x_spacing,
            y_spacing,
            start_x,
            start_y,
        );

        let placed: HashSet<usize> = state_map.keys().copied().collect();
        let unreachable: Vec<usize> = all_ids.difference(&placed).copied().collect();
        let unreachable_y = start_y + 4.0 * y_spacing;
        for (i, id) in unreachable.iter().enumerate() {
            if let Some(state) = state_map.get_mut(id) {
                state.position = iced::Point::new(start_x + (i as f32) * x_spacing, unreachable_y);
            }
        }

        // No-op: no from_point/to_point to update
    }

    fn apply_grid_layout_to_tab(active_tab: &mut Tab) {
        let states = &mut active_tab.states;

        if states.is_empty() {
            return;
        }

        let grid_size = (states.len() as f32).sqrt().ceil() as usize;
        let spacing = 150.0;
        let start_x = 100.0;
        let start_y = 100.0;

        for (i, state) in states.iter_mut().enumerate() {
            let row = i / grid_size;
            let col = i % grid_size;
            state.position = iced::Point::new(
                start_x + (col as f32 * spacing),
                start_y + (row as f32 * spacing)
            );
        }

        // No-op: no from_point/to_point to update
    }
}
