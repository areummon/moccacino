use std::collections::{HashMap, HashSet};
use indexmap::IndexSet;

use crate::gui::run::Run;
use crate::gui::theme::Family;
use crate::state_machine;

use moca_data::finite_automata::FiniteAutomata;
use moca_data::grammar::Grammar;
use moca_data::pushdown_automata::PushdownAutomata;
use moca_data::state::{State, StateID};
use moca_data::state_machine::{Machine, MachineKind, StateMachine};
use moca_data::turing_machine::TuringMachine;

static EMPTY_STATES: std::sync::OnceLock<HashMap<StateID, State>> = std::sync::OnceLock::new();
static EMPTY_FINALS: std::sync::OnceLock<HashSet<u64>> = std::sync::OnceLock::new();

fn empty_states() -> &'static HashMap<StateID, State> {
    EMPTY_STATES.get_or_init(HashMap::new)
}

fn empty_finals() -> &'static HashSet<u64> {
    EMPTY_FINALS.get_or_init(HashSet::new)
}

#[derive(Debug, Clone)]
pub(crate) enum TabMachine {
    Finite(FiniteAutomata),
    Pushdown(PushdownAutomata),
    Turing(TuringMachine),
    Grammar(Grammar),
}

impl Default for TabMachine {
    fn default() -> Self {
        TabMachine::Finite(FiniteAutomata::default())
    }
}

impl TabMachine {
    pub(crate) fn new_turing() -> Self {
        TabMachine::Turing(TuringMachine::new('_'))
    }

    pub(crate) fn new_pda() -> Self {
        TabMachine::Pushdown(PushdownAutomata::new("Z".to_string()))
    }

    pub(crate) fn machine_kind(&self) -> Option<MachineKind> {
        match self {
            TabMachine::Finite(_) => Some(MachineKind::Finite),
            TabMachine::Pushdown(_) => Some(MachineKind::Pushdown),
            TabMachine::Turing(_) => Some(MachineKind::Turing),
            TabMachine::Grammar(_) => None,
        }
    }

    pub(crate) fn is_grammar(&self) -> bool {
        matches!(self, TabMachine::Grammar(_))
    }

    pub(crate) fn family(&self) -> Family {
        match self {
            TabMachine::Finite(_) => Family::Finite,
            TabMachine::Pushdown(_) => Family::Pushdown,
            TabMachine::Turing(_) => Family::Turing,
            TabMachine::Grammar(_) => Family::Grammar,
        }
    }

    pub(crate) fn clear(&mut self) {
        match self {
            TabMachine::Finite(finite) => finite.clear(),
            TabMachine::Pushdown(pda) => {
                let stack_symbol = pda.get_initial_stack_symbol().to_string();
                *pda = PushdownAutomata::new(stack_symbol);
            },
            TabMachine::Turing(turing) => {
                let blank = turing.get_blank_symbol();
                *turing = TuringMachine::new(blank);
            },
            TabMachine::Grammar(_) => (),
        }
    }

    pub(crate) fn add_state_with_id_label(&mut self, id: u64, label: &str) {
        match self {
            TabMachine::Finite(finite) => finite.add_state_with_id_label(id, label),
            TabMachine::Pushdown(pda) => pda.add_state_with_id_label(id, label),
            TabMachine::Turing(turing) => turing.add_state_with_id_label(id, label),
            TabMachine::Grammar(_) => (),
        }
    }

    pub(crate) fn add_transition(&mut self, from: u64, to: u64, label: String) {
        match self {
            TabMachine::Finite(finite) => finite.add_transition(from, to, label),
            TabMachine::Pushdown(pda) => pda.add_transition(from, to, label),
            TabMachine::Turing(turing) => turing.add_transition(from, to, label),
            TabMachine::Grammar(_) => (),
        }
    }

    pub(crate) fn make_initial(&mut self, id: u64) {
        match self {
            TabMachine::Finite(finite) => finite.make_initial(id),
            TabMachine::Pushdown(pda) => pda.make_initial(id),
            TabMachine::Turing(turing) => turing.make_initial(id),
            TabMachine::Grammar(_) => (),
        }
    }

    pub(crate) fn make_final(&mut self, id: u64) {
        match self {
            TabMachine::Finite(finite) => finite.make_final(id),
            TabMachine::Pushdown(pda) => pda.make_final(id),
            TabMachine::Turing(turing) => turing.make_final(id),
            TabMachine::Grammar(_) => (),
        }
    }

    pub(crate) fn is_deterministic(&self) -> bool {
        match self {
            TabMachine::Finite(finite) => finite.is_deterministic(),
            TabMachine::Pushdown(pda) => pda.is_deterministic(),
            TabMachine::Turing(turing) => turing.is_deterministic(),
            TabMachine::Grammar(_) => false,
        }
    }

    pub(crate) fn states_ref(&self) -> &HashMap<StateID, State> {
        match self {
            TabMachine::Finite(finite) => finite.get_states_by_id_ref(),
            TabMachine::Pushdown(pda) => pda.get_states_by_id_ref(),
            TabMachine::Turing(turing) => turing.get_states_by_id_ref(),
            TabMachine::Grammar(_) => empty_states(),
        }
    }

    pub(crate) fn initial_id(&self) -> Option<StateID> {
        match self {
            TabMachine::Finite(finite) => *finite.get_initial_state_id(),
            TabMachine::Pushdown(pda) => *pda.get_initial_state_id(),
            TabMachine::Turing(turing) => *turing.get_initial_state_id(),
            TabMachine::Grammar(_) => None,
        }
    }

    pub(crate) fn final_states_ref(&self) -> &HashSet<u64> {
        match self {
            TabMachine::Finite(finite) => finite.get_final_states(),
            TabMachine::Pushdown(pda) => pda.get_final_states(),
            TabMachine::Turing(turing) => turing.get_final_states(),
            TabMachine::Grammar(_) => empty_finals(),
        }
    }

    pub(crate) fn accepts(&self, input: &str) -> bool {
        match self {
            TabMachine::Finite(finite) => Machine::accepts(finite, input),
            TabMachine::Pushdown(pda) => Machine::accepts(pda, input),
            TabMachine::Turing(turing) => Machine::accepts(turing, input),
            TabMachine::Grammar(grammar) => grammar.generate(input),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        match self {
            TabMachine::Finite(finite) => Machine::validate(finite),
            TabMachine::Pushdown(pda) => Machine::validate(pda),
            TabMachine::Turing(turing) => Machine::validate(turing),
            TabMachine::Grammar(grammar) => {
                if grammar.productions().is_empty() {
                    Err("The grammar has no productions.".to_string())
                } else {
                    Ok(())
                }
            },
        }
    }

    pub(crate) fn to_dfa(&self) -> Option<TabMachine> {
        match self {
            TabMachine::Finite(finite) if !finite.is_deterministic() => {
                Some(TabMachine::Finite(finite.to_dfa()))
            },
            _ => None,
        }
    }

    pub(crate) fn minimized(&self) -> Option<TabMachine> {
        match self {
            TabMachine::Finite(finite) if finite.is_deterministic() => {
                Some(TabMachine::Finite(finite.minimize()))
            },
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TabInsight {
    pub(crate) deterministic: bool,
    pub(crate) problem: Option<String>,
    pub(crate) production_count: usize,
    pub(crate) unsaved: bool,
}

#[derive(Default)]
pub(crate) struct Tab {
    pub(crate) state_machine: state_machine::State,
    pub(crate) transitions: HashMap<(usize, usize), IndexSet<String>>,
    pub(crate) states: Vec<state_machine::StateNode>,
    pub(crate) state_id_to_index: HashMap<usize, usize>,
    pub(crate) machine: TabMachine,
    pub(crate) initial_state: Option<usize>,
    pub(crate) final_states: std::collections::HashSet<usize>,
    pub(crate) editing_state: Option<usize>,
    pub(crate) editing_transition: Option<usize>,
    pub(crate) edit_text: String,
    pub(crate) check_input_dialog_open: bool,
    pub(crate) check_input_text: String,
    pub(crate) dock_collapsed: bool,
    pub(crate) insight: TabInsight,
    pub(crate) run_highlight: HashSet<usize>,
    pub(crate) saved_fingerprint: Option<u64>,
    synced_fingerprint: Option<u64>,
    insight_fingerprint: Option<u64>,
    pub(crate) regex_dialog_open: bool,
    pub(crate) regex_text: String,
    pub(crate) run_input: String,
    pub(crate) run: Option<Run>,
    pub(crate) playing: bool,
    pub(crate) grammar_content: iced::widget::text_editor::Content,
    pub(crate) grammar_text: String,
    pub(crate) grammar_word: String,
    pub(crate) grammar_output: Option<String>,
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
        tab.name = "Automaton".to_string();
        tab
    }

    pub(crate) fn new_turing() -> Self {
        let mut tab = Self::new();
        tab.machine = TabMachine::new_turing();
        tab.name = "Turing".to_string();
        tab
    }

    pub(crate) fn new_pda() -> Self {
        let mut tab = Self::new();
        tab.machine = TabMachine::new_pda();
        tab.name = "Pushdown".to_string();
        tab
    }

    pub(crate) fn new_grammar() -> Self {
        let mut tab = Self::new();
        tab.machine = TabMachine::Grammar(Grammar::default());
        tab.name = "Grammar".to_string();
        tab.grammar_text = "S -> a S b | ε\nS -> ε".to_string();
        tab.grammar_content = iced::widget::text_editor::Content::with_text(&tab.grammar_text);
        tab
    }

    pub(crate) fn new_with_name(name: String) -> Self {
        let mut tab = Self::new();
        tab.name = name;
        tab
    }

    pub(crate) fn new_of(family: Family) -> Self {
        let mut tab = match family {
            Family::Finite => Self::new(),
            Family::Pushdown => Self::new_pda(),
            Family::Turing => Self::new_turing(),
            Family::Grammar => Self::new_grammar(),
        };
        tab.mark_saved();
        tab
    }

    pub(crate) fn active_run_states(&self) -> HashSet<usize> {
        self.run.as_ref().map(Run::active_states).unwrap_or_default()
    }

    pub(crate) fn run_state(&self) -> (bool, bool, bool) {
        let finished = self.run.as_ref().is_some_and(|run| run.finished().is_some());
        (self.run.is_some(), finished, self.playing && !finished)
    }

    pub(crate) fn refresh_insight(&mut self) {
        let fingerprint = self.content_fingerprint();
        if self.insight_fingerprint != Some(fingerprint) {
            self.insight_fingerprint = Some(fingerprint);
            self.insight = self.compute_insight();
        }
        self.insight.unsaved = self.unsaved_given(fingerprint);
    }

    fn compute_insight(&mut self) -> TabInsight {
        if self.machine.is_grammar() {
            let parsed = moca_data::grammar::parse_grammar(self.grammar_text.trim());
            return match parsed {
                Ok(grammar) => TabInsight {
                    deterministic: false,
                    problem: grammar.productions().is_empty().then(|| "No productions yet".to_string()),
                    production_count: grammar.productions().len(),
                    unsaved: false,
                },
                Err(error) if self.grammar_text.trim().is_empty() => TabInsight {
                    problem: Some(format!("Empty grammar ({error})")),
                    ..TabInsight::default()
                },
                Err(error) => TabInsight {
                    problem: Some(error.to_string()),
                    ..TabInsight::default()
                },
            };
        }
        self.sync_gui_to_machine();
        TabInsight {
            deterministic: self.machine.is_deterministic(),
            problem: if self.states.is_empty() {
                None
            } else {
                self.machine.validate().err()
            },
            production_count: 0,
            unsaved: false,
        }
    }

    pub(crate) fn content_fingerprint(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.machine.family().title().hash(&mut hasher);
        if self.machine.is_grammar() {
            self.grammar_text.trim().hash(&mut hasher);
            return hasher.finish();
        }
        let mut states: Vec<(usize, &str)> =
            self.states.iter().map(|node| (node.id, node.label.as_str())).collect();
        states.sort_unstable();
        states.hash(&mut hasher);
        let mut transitions: Vec<((usize, usize), Vec<&str>)> = self
            .transitions
            .iter()
            .map(|(pair, labels)| {
                let mut labels: Vec<&str> = labels.iter().map(String::as_str).collect();
                labels.sort_unstable();
                (*pair, labels)
            })
            .collect();
        transitions.sort_unstable();
        transitions.hash(&mut hasher);
        self.initial_state.hash(&mut hasher);
        let mut finals: Vec<usize> = self.final_states.iter().copied().collect();
        finals.sort_unstable();
        finals.hash(&mut hasher);
        hasher.finish()
    }

    fn is_empty_content(&self) -> bool {
        if self.machine.is_grammar() {
            self.grammar_text.trim().is_empty()
        } else {
            self.states.is_empty() && self.transitions.is_empty()
        }
    }

    pub(crate) fn has_unsaved_changes(&self) -> bool {
        self.unsaved_given(self.content_fingerprint())
    }

    fn unsaved_given(&self, fingerprint: u64) -> bool {
        match self.saved_fingerprint {
            Some(saved) => saved != fingerprint,
            None => !self.is_empty_content(),
        }
    }

    pub(crate) fn mark_saved(&mut self) {
        self.saved_fingerprint = Some(self.content_fingerprint());
        self.insight.unsaved = false;
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

    pub(crate) fn active_tool(&self) -> crate::state_machine::EditorTool {
        self.state_machine.active_tool()
    }

    pub(crate) fn set_active_tool(&mut self, tool: crate::state_machine::EditorTool) {
        self.state_machine.set_tool(tool);
        self.state_machine.request_redraw();
    }

    pub(crate) fn sync_gui_to_machine(&mut self) {
        let fingerprint = self.content_fingerprint();
        if self.synced_fingerprint == Some(fingerprint) {
            return;
        }
        self.synced_fingerprint = Some(fingerprint);
        self.machine.clear();

        for state_node in &self.states {
            self.machine.add_state_with_id_label(state_node.id as u64, &state_node.label);
        }

        for (&(from, to), labels) in &self.transitions {
            for label in labels {
                let normalized = if !matches!(self.machine, TabMachine::Turing(_))
                    && (label.trim().is_empty() || label == "ε")
                {
                    "ε"
                } else {
                    label.as_str()
                };
                self.machine.add_transition(from as u64, to as u64, normalized.to_string());
            }
        }

        for &state_id in &self.final_states {
            self.machine.make_final(state_id as u64);
        }

        if let Some(initial_id) = self.initial_state {
            self.machine.make_initial(initial_id as u64);
        }
    }

    pub(crate) fn load_machine_to_gui(&mut self) {
        self.states.clear();
        self.transitions.clear();
        self.state_id_to_index.clear();
        self.initial_state = None;
        self.final_states.clear();

        let mut max_id_after_load = 0;
        let mut loaded: Vec<_> = self.machine.states_ref().iter().collect();
        loaded.sort_unstable_by_key(|(id, _)| **id);
        for (id, state) in loaded {
            let state_node = state_machine::StateNode::new(
                *id as usize,
                iced::Point::new(100.0, 100.0),
                30.0,
                state.name.clone()
            );
            let index = self.states.len();
            self.states.push(state_node);
            self.state_id_to_index.insert(*id as usize, index);
            max_id_after_load = max_id_after_load.max(*id as usize);
        }

        self.state_machine.next_id = max_id_after_load + 1;

        for (from_id, state) in self.machine.states_ref() {
            for (to_id, inputs) in state.iter_by_transition() {
                let key = (*from_id as usize, *to_id as usize);
                let entry = self.transitions.entry(key).or_default();
                for label in inputs {
                    let label = if label.trim().is_empty() || label == "ε" { "ε".to_string() } else { label.clone() };
                    entry.insert(label);
                }
            }
        }

        if let Some(initial_id) = self.machine.initial_id() {
            if let Some(state) = self.states.iter()
                .find(|s| s.id == initial_id as usize) {
                self.initial_state = Some(state.id);
            }
        }

        for final_id in self.machine.final_states_ref() {
            if let Some(state) = self.states.iter()
                .find(|s| s.id == *final_id as usize) {
                self.final_states.insert(state.id);
            }
        }

        if self.initial_state.is_some() {
            Self::apply_layered_layout_to_tab(self);
        } else {
            Self::apply_grid_layout_to_tab(self);
        }

        self.state_machine.set_scroll(iced::Vector::new(0.0, 0.0));
        self.state_machine.request_redraw();
    }

    fn apply_layered_layout_to_tab(active_tab: &mut Tab) {
        use std::collections::BTreeSet;

        let node_count = active_tab.states.len();
        if node_count == 0 {
            return;
        }
        let initial = match active_tab.initial_state {
            Some(id) => id,
            None => return,
        };
        let index_of: HashMap<usize, usize> = active_tab
            .states
            .iter()
            .enumerate()
            .map(|(index, state)| (state.id, index))
            .collect();
        let root = match index_of.get(&initial) {
            Some(&root) => root,
            None => {
                Self::apply_grid_layout_to_tab(active_tab);
                return;
            }
        };

        let neighbors: Vec<Vec<usize>> = {
            let mut adjacency: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); node_count];
            for &(from, to) in active_tab.transitions.keys() {
                if from != to {
                    if let (Some(&fi), Some(&ti)) = (index_of.get(&from), index_of.get(&to)) {
                        adjacency[fi].insert(ti);
                    }
                }
            }
            adjacency
                .into_iter()
                .map(|set| set.into_iter().collect())
                .collect()
        };

        let mut color = vec![0u8; node_count];
        let mut back_edge = vec![BTreeSet::new(); node_count];
        let mut preorder: Vec<usize> = Vec::new();
        {
            let mut iter_pos = vec![0usize; node_count];
            let mut stack: Vec<usize> = vec![root];
            color[root] = 1;
            preorder.push(root);
            while let Some(&u) = stack.last() {
                if iter_pos[u] < neighbors[u].len() {
                    let v = neighbors[u][iter_pos[u]];
                    iter_pos[u] += 1;
                    match color[v] {
                        0 => {
                            color[v] = 1;
                            preorder.push(v);
                            stack.push(v);
                        }
                        1 => {
                            back_edge[u].insert(v);
                        }
                        _ => {}
                    }
                } else {
                    color[u] = 2;
                    stack.pop();
                }
            }
        }
        let reachable: Vec<bool> = color.iter().map(|&c| c != 0).collect();

        let mut indegree = vec![0usize; node_count];
        let mut kept: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); node_count];
        for u in 0..node_count {
            if !reachable[u] {
                continue;
            }
            for &v in &neighbors[u] {
                if reachable[v] && !back_edge[u].contains(&v) {
                    kept[u].insert(v);
                    indegree[v] += 1;
                }
            }
        }
        let mut layer = vec![0usize; node_count];
        {
            let mut queue: std::collections::VecDeque<usize> = (0..node_count)
                .filter(|&u| reachable[u] && indegree[u] == 0)
                .collect();
            while let Some(u) = queue.pop_front() {
                for &v in &kept[u] {
                    layer[v] = layer[v].max(layer[u] + 1);
                    indegree[v] -= 1;
                    if indegree[v] == 0 {
                        queue.push_back(v);
                    }
                }
            }
        }

        let max_layer = reachable
            .iter()
            .enumerate()
            .filter(|&(_, &r)| r)
            .map(|(u, _)| layer[u])
            .max()
            .unwrap_or(0);
        let mut layers: Vec<Vec<usize>> = vec![Vec::new(); max_layer + 1];
        for &u in &preorder {
            layers[layer[u]].push(u);
        }

        let mut undirected: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); node_count];
        for u in 0..node_count {
            if !reachable[u] {
                continue;
            }
            for &v in &neighbors[u] {
                if reachable[v] && u != v {
                    undirected[u].insert(v);
                    undirected[v].insert(u);
                }
            }
        }
        let mut pos = vec![0usize; node_count];
        let sync_positions = |layers: &[Vec<usize>], pos: &mut [usize]| {
            for layer_nodes in layers {
                for (index, &u) in layer_nodes.iter().enumerate() {
                    pos[u] = index;
                }
            }
        };
        sync_positions(&layers, &mut pos);
        for sweep in 0..4 {
            let order: Vec<usize> = if sweep % 2 == 0 {
                (1..=max_layer).collect()
            } else {
                (0..max_layer).rev().collect()
            };
            for l in order {
                let mut keys: Vec<(usize, f32)> = layers[l]
                    .iter()
                    .map(|&u| {
                        let neighbor_set = &undirected[u];
                        let key = if neighbor_set.is_empty() {
                            pos[u] as f32
                        } else {
                            let total: f32 = neighbor_set.iter().map(|&v| pos[v] as f32).sum();
                            total / neighbor_set.len() as f32
                        };
                        (u, key)
                    })
                    .collect();
                keys.sort_by(|a, b| {
                    a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
                });
                layers[l] = keys.into_iter().map(|(u, _)| u).collect();
                for (index, &u) in layers[l].iter().enumerate() {
                    pos[u] = index;
                }
            }
        }

        const START_X: f32 = 150.0;
        const START_Y: f32 = 150.0;
        const Y_SPACING: f32 = 150.0;
        const TARGET_WIDTH: f32 = 1500.0;
        let widest = layers
            .iter()
            .map(|layer_nodes| layer_nodes.len())
            .max()
            .unwrap_or(1)
            .max(1) as f32;
        let x_spacing = (TARGET_WIDTH / widest).clamp(120.0, 260.0);
        for (layer_index, layer_nodes) in layers.iter().enumerate() {
            let count = layer_nodes.len() as f32;
            for (i, &u) in layer_nodes.iter().enumerate() {
                let x = START_X + (i as f32 - (count - 1.0) / 2.0) * x_spacing;
                let y = START_Y + layer_index as f32 * Y_SPACING;
                active_tab.states[u].position = iced::Point::new(x, y);
            }
        }

        let unreachable: Vec<usize> = (0..node_count).filter(|&u| !reachable[u]).collect();
        if !unreachable.is_empty() {
            let per_row = ((TARGET_WIDTH / x_spacing).floor() as usize).max(1);
            for (row, chunk) in unreachable.chunks(per_row).enumerate() {
                let count = chunk.len() as f32;
                for (i, &u) in chunk.iter().enumerate() {
                    let x = START_X + (i as f32 - (count - 1.0) / 2.0) * x_spacing;
                    let y = START_Y + (max_layer + 1 + row) as f32 * Y_SPACING;
                    active_tab.states[u].position = iced::Point::new(x, y);
                }
            }
        }
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
    }
}
