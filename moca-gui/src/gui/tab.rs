use std::collections::{HashMap, HashSet};
use indexmap::IndexSet;

use crate::gui::theme::Family;
use crate::state_machine;

use moca_data::finite_automata::{FiniteAutomata, FiniteConfiguration};
use moca_data::grammar::Grammar;
use moca_data::pushdown_automata::{PdaConfiguration, PushdownAutomata};
use moca_data::state::{State, StateID};
use moca_data::state_machine::{Machine, MachineKind, StateMachine};
use moca_data::turing_machine::{Configuration, RunOutcome, TuringMachine};

/* Shared empty maps so grammar tabs can answer structural queries without
 * per-call allocations. */
static EMPTY_STATES: std::sync::OnceLock<HashMap<StateID, State>> = std::sync::OnceLock::new();
static EMPTY_FINALS: std::sync::OnceLock<HashSet<u64>> = std::sync::OnceLock::new();

fn empty_states() -> &'static HashMap<StateID, State> {
    EMPTY_STATES.get_or_init(HashMap::new)
}

fn empty_finals() -> &'static HashSet<u64> {
    EMPTY_FINALS.get_or_init(HashSet::new)
}

/* Snapshot of a running Turing machine shown in the tab's run panel. */
#[derive(Debug, Clone)]
pub(crate) struct TmRun {
    pub(crate) config: Configuration,
    pub(crate) steps: u64,
    /* Set once the machine accepted or rejected; some branches never do. */
    pub(crate) finished: Option<RunOutcome>,
}

/* Frontier of a running nondeterministic Turing machine: every live branch's
 * configuration after `level` parallel steps. Mirrors
 * `run_nondeterministic`'s level loop and dedup, one level per Step press. */
#[derive(Debug, Clone)]
pub(crate) struct NdFrontier {
    pub(crate) level: u64,
    pub(crate) alive: Vec<Configuration>,
    pub(crate) visited: HashSet<Configuration>,
    pub(crate) finished: Option<RunOutcome>,
}

/* Snapshot of a running pushdown automaton shown in the tab's run panel.
 * The original input is kept so the ribbon can dim the consumed prefix. */
#[derive(Debug, Clone)]
pub(crate) struct PdaRun {
    pub(crate) config: PdaConfiguration,
    pub(crate) input: String,
    pub(crate) steps: u64,
    /* Some(true) = accepted, Some(false) = rejected (halt or bounds). */
    pub(crate) finished: Option<bool>,
}

/* Snapshot of a running finite automaton shown in the tab's run panel. The
 * original input is kept so the ribbon can dim the consumed prefix; the
 * visited set mirrors `check_input`'s memoization: revisiting a (state,
 * remaining input) pair is an ε-cycle dead end. */
#[derive(Debug, Clone)]
pub(crate) struct FiniteRun {
    pub(crate) config: FiniteConfiguration,
    pub(crate) input: String,
    pub(crate) steps: u64,
    pub(crate) visited: HashSet<FiniteConfiguration>,
    /* Some(true) = accepted, Some(false) = rejected. */
    pub(crate) finished: Option<bool>,
}

/* Frontier of a running nondeterministic machine: every live branch's
 * configuration after `level` parallel transitions, deduped against the
 * visited set. Mirrors the library traversals (arrival acceptance before
 * expanding, empty frontier = rejected), one level per Step press. */
#[derive(Debug, Clone)]
pub(crate) struct FiniteNdFrontier {
    pub(crate) level: u64,
    pub(crate) alive: Vec<FiniteConfiguration>,
    pub(crate) visited: HashSet<FiniteConfiguration>,
    pub(crate) finished: Option<bool>,
}

#[derive(Debug, Clone)]
pub(crate) struct PdaNdFrontier {
    pub(crate) level: u64,
    pub(crate) alive: Vec<PdaConfiguration>,
    pub(crate) visited: HashSet<PdaConfiguration>,
    pub(crate) finished: Option<bool>,
}

/* The formal object behind a tab: machine families dispatch through the
 * shared traits, grammars use their own surface (they are not state
 * machines), and finite-only transformations stay explicit optional
 * methods. */
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

    /* Machine families implementing the shared traits; grammars are not
     * machines, so their tabs answer None here. */
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

    // ---- Structural API (StateMachine trait; grammar variant ignores it) ----

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
            // The canvas has nothing to show for grammar tabs; an empty map
            // keeps load paths uniform.
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

    // ---- Behavioral API ----

    /* Grammars interpret "accepts" as CYK membership. */
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

    // ---- Finite-only transformations ----

    /* Subset construction into a DFA, for nondeterministic finite automata. */
    pub(crate) fn into_dfa(&self) -> Option<TabMachine> {
        match self {
            TabMachine::Finite(finite) if !finite.is_deterministic() => {
                Some(TabMachine::Finite(finite.to_dfa()))
            },
            _ => None,
        }
    }

    /* Hopcroft minimization, for deterministic finite automata. */
    pub(crate) fn minimized(&self) -> Option<TabMachine> {
        match self {
            TabMachine::Finite(finite) if finite.is_deterministic() => {
                Some(TabMachine::Finite(finite.minimize()))
            },
            _ => None,
        }
    }
}

/* Facts the status bar shows about a tab, recomputed after edits that can
 * change the machine (never per frame, never on plain drags or scrolls). */
#[derive(Debug, Clone, Default)]
pub(crate) struct TabInsight {
    pub(crate) deterministic: bool,
    /* Why the machine cannot run yet, if anything. */
    pub(crate) problem: Option<String>,
    pub(crate) production_count: usize,
    /* The content differs from what was last saved or loaded (or, for a
     * tab that was never saved, it is not empty). */
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
    // Run dock collapsed to its header row.
    pub(crate) dock_collapsed: bool,
    // Cached status-bar facts, refreshed after structural edits.
    pub(crate) insight: TabInsight,
    // States the loaded run occupies, mirrored after every update so the
    // canvas can glow them (and redraw only when they change).
    pub(crate) run_highlight: HashSet<usize>,
    // Fingerprint of the content as last saved or loaded; None for tabs
    // that never touched a file (generated results), which count as
    // unsaved as soon as they hold anything.
    pub(crate) saved_fingerprint: Option<u64>,
    pub(crate) regex_dialog_open: bool,
    pub(crate) regex_text: String,
    // Turing run panel state.
    pub(crate) tm_input_text: String,
    pub(crate) tm_playing: bool,
    // Pushdown run panel state.
    pub(crate) pda_input_text: String,
    pub(crate) pda_run: Option<PdaRun>,
    pub(crate) pda_playing: bool,
    pub(crate) pda_frontier: Option<PdaNdFrontier>,
    // Finite run panel state.
    pub(crate) finite_input_text: String,
    pub(crate) finite_playing: bool,
    pub(crate) finite_run: Option<FiniteRun>,
    pub(crate) finite_frontier: Option<FiniteNdFrontier>,
    // Grammar panel state: editor content, parsed result mirror and last
    // outputs. The editor is the source of truth while typing; grammar_text
    // mirrors it as a plain string for the parser calls.
    pub(crate) grammar_content: iced::widget::text_editor::Content,
    pub(crate) grammar_text: String,
    pub(crate) grammar_word: String,
    pub(crate) grammar_output: Option<String>,
    pub(crate) tm_run: Option<TmRun>,
    pub(crate) tm_frontier: Option<NdFrontier>,
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
        tab.transitions = HashMap::new();
        tab
    }

    /* A fresh tab holding an empty single-tape Turing machine. */
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

    /* A fresh tab whose panel edits a context-free grammar. */
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
        tab.transitions = HashMap::new();
        tab
    }

    /* A fresh, empty tab of the given family. It starts clean (the default
     * grammar template included), so closing it right away never asks. */
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

    /* States the loaded run currently occupies (every live branch for
     * nondeterministic frontiers); highlighted on the canvas. */
    pub(crate) fn active_run_states(&self) -> HashSet<usize> {
        let mut active = HashSet::new();
        match &self.machine {
            TabMachine::Finite(_) => {
                if let Some(run) = &self.finite_run {
                    active.insert(run.config.state_id() as usize);
                }
                if let Some(frontier) = &self.finite_frontier {
                    active.extend(frontier.alive.iter().map(|config| config.state_id() as usize));
                }
            }
            TabMachine::Pushdown(_) => {
                if let Some(run) = &self.pda_run {
                    active.insert(run.config.state_id() as usize);
                }
                if let Some(frontier) = &self.pda_frontier {
                    active.extend(frontier.alive.iter().map(|config| config.state_id() as usize));
                }
            }
            TabMachine::Turing(_) => {
                if let Some(run) = &self.tm_run {
                    active.insert(run.config.state_id() as usize);
                }
                if let Some(frontier) = &self.tm_frontier {
                    active.extend(frontier.alive.iter().map(|config| config.state_id() as usize));
                }
            }
            TabMachine::Grammar(_) => {}
        }
        active
    }

    /* Whether a run is loaded, has finished, and is auto-playing. A finished
     * run never plays, whatever its play flag still says. */
    pub(crate) fn run_state(&self) -> (bool, bool, bool) {
        let (loaded, finished, playing) = match &self.machine {
            TabMachine::Turing(_) => (
                self.tm_run.is_some() || self.tm_frontier.is_some(),
                self.tm_run.as_ref().is_some_and(|run| run.finished.is_some())
                    || self.tm_frontier.as_ref().is_some_and(|frontier| frontier.finished.is_some()),
                self.tm_playing,
            ),
            TabMachine::Pushdown(_) => (
                self.pda_run.is_some() || self.pda_frontier.is_some(),
                self.pda_run.as_ref().is_some_and(|run| run.finished.is_some())
                    || self.pda_frontier.as_ref().is_some_and(|frontier| frontier.finished.is_some()),
                self.pda_playing,
            ),
            TabMachine::Finite(_) => (
                self.finite_run.is_some() || self.finite_frontier.is_some(),
                self.finite_run.as_ref().is_some_and(|run| run.finished.is_some())
                    || self.finite_frontier.as_ref().is_some_and(|frontier| frontier.finished.is_some()),
                self.finite_playing,
            ),
            TabMachine::Grammar(_) => (false, false, false),
        };
        (loaded, finished, playing && !finished)
    }

    /* Syncs the drawing into the machine and caches what the status bar
     * shows. Grammar tabs parse their editor text instead. */
    pub(crate) fn refresh_insight(&mut self) {
        if self.machine.is_grammar() {
            let parsed = moca_data::grammar::parse_grammar(self.grammar_text.trim());
            self.insight = match parsed {
                Ok(grammar) => TabInsight {
                    deterministic: false,
                    problem: grammar.productions().is_empty().then(|| "No productions yet".to_string()),
                    production_count: grammar.productions().len(),
                    unsaved: false,
                },
                Err(error) if self.grammar_text.trim().is_empty() => TabInsight {
                    problem: Some(format!("Empty grammar ({})", error)),
                    ..TabInsight::default()
                },
                Err(error) => TabInsight {
                    problem: Some(error.to_string()),
                    ..TabInsight::default()
                },
            };
            self.insight.unsaved = self.has_unsaved_changes();
            return;
        }
        self.sync_gui_to_machine();
        self.insight = TabInsight {
            deterministic: self.machine.is_deterministic(),
            problem: if self.states.is_empty() {
                None
            } else {
                self.machine.validate().err()
            },
            production_count: 0,
            unsaved: self.has_unsaved_changes(),
        };
    }

    /* Hash of everything a .ce save persists: names, transitions (labels as
     * a set), initial and accepting states, or the grammar text. Layout,
     * zoom and runs are not part of the file, so they never count as
     * changes. */
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
        match self.saved_fingerprint {
            Some(saved) => saved != self.content_fingerprint(),
            None => !self.is_empty_content(),
        }
    }

    /* Records the current content as saved (after a save or a load). */
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

    /* The active canvas editing tool, stored on the canvas state. */
    pub(crate) fn active_tool(&self) -> crate::state_machine::EditorTool {
        self.state_machine.active_tool()
    }

    pub(crate) fn set_active_tool(&mut self, tool: crate::state_machine::EditorTool) {
        self.state_machine.set_tool(tool);
        self.state_machine.request_redraw();
    }

    pub(crate) fn sync_gui_to_machine(&mut self) {
        self.machine.clear();

        // Add all states
        for state_node in &self.states {
            self.machine.add_state_with_id_label(state_node.id as u64, &state_node.label);
        }

        // Add all transitions (multi-label)
        for (&(from, to), labels) in &self.transitions {
            for label in labels {
                // A blank or whitespace-only label denotes ε on machine
                // families that have ε-transitions; Turing tapes have no
                // such concept, so blank labels stay as typed there and are
                // reported by Machine::validate.
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

        // Set final states
        for &state_id in &self.final_states {
            self.machine.make_final(state_id as u64);
        }

        // Set initial state
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
        for (id, state) in self.machine.states_ref() {
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

        // Add all transitions (multi-label)
        for (from_id, state) in self.machine.states_ref() {
            for (to_id, inputs) in state.iter_by_transition() {
                let key = (*from_id as usize, *to_id as usize);
                let entry = self.transitions.entry(key).or_insert_with(indexmap::IndexSet::new);
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

        // Fresh content starts visible at the top-left of the canvas.
        self.state_machine.set_scroll(iced::Vector::new(0.0, 0.0));
        self.state_machine.request_redraw();
    }

    /* Deterministic layered (Sugiyama-lite) layout for loaded machines:
     * states are stacked in top-down layers following the flow from the
     * initial state. Cycle back-edges are excluded from the layering so
     * star loops cannot fold the graph onto itself, crossing order is
     * improved with barycenter sweeps, and the horizontal spacing adapts
     * so small graphs breathe while wide ones stay navigable. */
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

        // Adjacency over the drawn states, self-loops excluded, iteration
        // order deterministic (sorted ids).
        let neighbors: Vec<Vec<usize>> = {
            let mut adjacency: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); node_count];
            for (&(from, to), _) in &active_tab.transitions {
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

        // Depth-first search from the initial state with gray/black
        // coloring: an edge into a gray node closes a cycle (back edge)
        // and is dropped from the layering graph.
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

        // Longest-path layering: Kahn topological order over the kept
        // edges, relaxing layer[v] to layer[u] + 1.
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

        // Barycenter sweeps: repeatedly reorder each layer by the mean
        // position of its (undirected) neighbors in the neighboring
        // layers; stable sort keeps ties deterministic.
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
                // Precompute barycenter keys up front so the position
                // table is not borrowed while the layer is reordered.
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

        // Coordinates: horizontal spacing adapts to the widest layer so
        // the drawing comfortably uses the canvas width.
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

        // States not reachable from the initial state go to their own
        // wrapped rows below the layered region.
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

        // No-op: no from_point/to_point to update
    }
}
