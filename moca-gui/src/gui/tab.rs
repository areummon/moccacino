use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};

use iced::widget::text_editor;
use indexmap::IndexSet;

use crate::gui::layout;
use crate::gui::automaton::Automaton;
use crate::gui::history::History;
use crate::gui::run::RunSession;
use crate::family::Family;
use crate::state_machine::{self, Drawing, EditorTool, StateNode, Transitions};

use moca_data::entity_file::{write_grammar_entity, Entity};
use moca_data::finite_automata::FiniteAutomata;
use moca_data::grammar::{parse_grammar, Grammar};
use moca_data::pushdown_automata::{PushdownAutomata, DEFAULT_STACK};
use moca_data::state::EPSILON;
use moca_data::turing_machine::{TuringMachine, DEFAULT_BLANK};

const DEFAULT_GRAMMAR: &str = "S -> a S b | ε\nS -> ε";

#[derive(Default)]
pub(crate) struct GrammarEditor {
    pub(crate) content: text_editor::Content,
    pub(crate) text: String,
    pub(crate) word: String,
    pub(crate) output: Option<String>,
}

impl GrammarEditor {
    fn new(text: &str, output: Option<String>) -> Self {
        GrammarEditor { content: text_editor::Content::with_text(text), text: text.to_string(), output, ..Self::default() }
    }

    pub(crate) fn parse(&self) -> Result<Grammar, String> {
        if self.text.trim().is_empty() {
            return Err("The grammar is empty.".to_string());
        }
        parse_grammar(self.text.trim()).map_err(|error| format!("Invalid grammar: {error}"))
    }
}

pub(crate) enum TabKind {
    Automaton(Automaton),
    Grammar(GrammarEditor),
}

impl Default for TabKind {
    fn default() -> Self {
        TabKind::Automaton(Automaton::Finite(FiniteAutomata::default()))
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TabInsight {
    pub(crate) deterministic: bool,
    pub(crate) problem: Option<String>,
    pub(crate) production_count: usize,
    pub(crate) unsaved: bool,
}

pub(crate) struct LabelEdit {
    pub(crate) pair: (usize, usize),
    pub(crate) inputs: Vec<String>,
}

#[derive(Default)]
pub(crate) struct Tab {
    pub(crate) name: String,
    pub(crate) kind: TabKind,
    pub(crate) state_machine: state_machine::State,
    pub(crate) drawing: Drawing,
    history: History,
    pub(crate) editing_state: Option<usize>,
    pub(crate) pending_transition: Option<(usize, usize)>,
    pub(crate) edit_text: String,
    pub(crate) label_edit: Option<LabelEdit>,
    pub(crate) check_input: Option<String>,
    pub(crate) regex_input: Option<String>,
    pub(crate) run_input: String,
    pub(crate) run: Option<RunSession>,
    pub(crate) run_highlight: HashSet<usize>,
    pub(crate) dock_collapsed: bool,
    pub(crate) insight: TabInsight,
    pub(crate) saved_fingerprint: Option<u64>,
    synced_fingerprint: Option<u64>,
    insight_fingerprint: Option<u64>,
}

fn normalize_label(family: Family, label: &str) -> String {
    let label = label.trim();
    if family != Family::Turing && label.is_empty() {
        EPSILON.to_string()
    } else {
        label.to_string()
    }
}

impl Tab {
    pub(crate) fn new_of(family: Family) -> Self {
        let (name, kind) = match family {
            Family::Finite => ("Automaton", TabKind::Automaton(Automaton::Finite(FiniteAutomata::default()))),
            Family::Pushdown => (
                "Pushdown",
                TabKind::Automaton(Automaton::Pushdown(PushdownAutomata::new(DEFAULT_STACK.to_string()))),
            ),
            Family::Turing => ("Turing", TabKind::Automaton(Automaton::Turing(TuringMachine::new(DEFAULT_BLANK)))),
            Family::Grammar => ("Grammar", TabKind::Grammar(GrammarEditor::new(DEFAULT_GRAMMAR, None))),
        };
        let mut tab = Self { name: name.to_string(), kind, ..Self::default() };
        tab.mark_saved();
        tab
    }

    pub(crate) fn from_automaton(name: String, automaton: Automaton) -> Self {
        let mut tab = Self { name, kind: TabKind::Automaton(automaton), ..Self::default() };
        tab.load_machine_to_gui();
        tab.history = History::starting_at(&tab.drawing);
        tab
    }

    pub(crate) fn from_grammar(name: String, grammar: &Grammar, output: String) -> Self {
        let editor = GrammarEditor::new(&grammar.to_string(), Some(output));
        Self { name, kind: TabKind::Grammar(editor), ..Self::default() }
    }

    pub(crate) fn from_entity(name: String, entity: Entity) -> Self {
        match entity {
            Entity::Grammar(grammar) => {
                let output = format!("Loaded from .ce file. Start symbol: {}.", grammar.start_symbol());
                Self::from_grammar(name, &grammar, output)
            }
            Entity::Finite(finite) => Self::from_automaton(name, Automaton::Finite(finite)),
            Entity::Pushdown(pda) => Self::from_automaton(name, Automaton::Pushdown(pda)),
            Entity::Turing(turing) => Self::from_automaton(name, Automaton::Turing(turing)),
        }
    }

    pub(crate) fn clear(&mut self) {
        let kind = match std::mem::take(&mut self.kind) {
            TabKind::Automaton(mut automaton) => {
                automaton.clear();
                TabKind::Automaton(automaton)
            }
            grammar => grammar,
        };
        *self = Self {
            name: std::mem::take(&mut self.name),
            saved_fingerprint: self.saved_fingerprint,
            run_input: std::mem::take(&mut self.run_input),
            dock_collapsed: self.dock_collapsed,
            history: std::mem::take(&mut self.history),
            kind,
            ..Self::default()
        };
    }

    pub(crate) fn family(&self) -> Family {
        match &self.kind {
            TabKind::Automaton(automaton) => automaton.family(),
            TabKind::Grammar(_) => Family::Grammar,
        }
    }

    pub(crate) fn is_grammar(&self) -> bool {
        matches!(self.kind, TabKind::Grammar(_))
    }

    pub(crate) fn grammar(&self) -> Option<&GrammarEditor> {
        match &self.kind {
            TabKind::Grammar(editor) => Some(editor),
            TabKind::Automaton(_) => None,
        }
    }

    pub(crate) fn grammar_mut(&mut self) -> Option<&mut GrammarEditor> {
        match &mut self.kind {
            TabKind::Grammar(editor) => Some(editor),
            TabKind::Automaton(_) => None,
        }
    }

    pub(crate) fn automaton(&mut self) -> Result<&Automaton, String> {
        self.sync_gui_to_machine();
        match &self.kind {
            TabKind::Automaton(automaton) => Ok(automaton),
            TabKind::Grammar(_) => Err("the tab holds a grammar, not a state machine.".to_string()),
        }
    }

    pub(crate) fn finite(&mut self, verb: &str) -> Result<&FiniteAutomata, String> {
        self.automaton()
            .map_err(|reason| format!("Cannot {verb}: {reason}"))
            .and_then(|automaton| automaton.finite(verb))
    }

    pub(crate) fn accepts(&mut self, input: &str) -> Result<bool, String> {
        if let Some(editor) = self.grammar() {
            return Ok(editor.parse()?.generate(input));
        }
        let automaton = self.automaton()?;
        automaton.validate()?;
        Ok(automaton.behavior().accepts(input))
    }

    pub(crate) fn entity_text(&mut self) -> Result<String, String> {
        let name = self.name.clone();
        if let Some(editor) = self.grammar() {
            return write_grammar_entity(&name, &editor.parse()?).map_err(|reason| format!("Cannot save: {reason}."));
        }
        let automaton = self.automaton()?;
        let text = automaton.entity_text(&name).map_err(|reason| format!("Cannot save: {reason}."))?;
        automaton.validate()?;
        Ok(text)
    }

    /// Moves `state_id` to `position`, carrying the rest of the selection along when it is part of it.
    pub(crate) fn move_state(&mut self, state_id: usize, position: iced::Point) {
        let Some(node) = self.drawing.node(state_id) else {
            return;
        };
        let delta = position - node.position;
        let selection = self.state_machine.selection();
        let moving = if selection.contains(&state_id) { selection.clone() } else { HashSet::from([state_id]) };
        self.drawing.translate(&moving, delta);
        self.state_machine.redraw_moved(&moving);
    }

    pub(crate) fn select_all(&mut self) {
        self.state_machine.set_selection(self.drawing.states.iter().map(|node| node.id).collect());
    }

    pub(crate) fn delete_selection(&mut self) {
        for state_id in self.state_machine.selection().clone() {
            self.drawing.delete_state(state_id);
        }
        self.state_machine.set_selection(HashSet::new());
    }

    pub(crate) fn add_transition(&mut self, pair: (usize, usize), label: &str) {
        let label = normalize_label(self.family(), label);
        self.drawing.add_transition(pair, label);
    }

    /// Blank inputs are dropped, so clearing every field removes the transition.
    pub(crate) fn set_transition_labels(&mut self, pair: (usize, usize), inputs: &[String]) {
        let family = self.family();
        let labels = if inputs.iter().all(|input| input.trim().is_empty()) {
            IndexSet::new()
        } else {
            inputs.iter().map(|input| normalize_label(family, input)).collect()
        };
        self.drawing.set_transition_labels(pair, labels);
    }

    pub(crate) fn record_history(&mut self) {
        self.history.record(&self.drawing);
    }

    pub(crate) fn can_undo(&self) -> bool {
        self.history.can_undo(&self.drawing)
    }

    pub(crate) fn can_redo(&self) -> bool {
        self.history.can_redo(&self.drawing)
    }

    pub(crate) fn undo(&mut self) -> bool {
        let changed = self.history.undo(&mut self.drawing);
        self.after_history_step(changed)
    }

    pub(crate) fn redo(&mut self) -> bool {
        let changed = self.history.redo(&mut self.drawing);
        self.after_history_step(changed)
    }

    // Positions are not fingerprinted, so a step that only moves states needs its own redraw.
    fn after_history_step(&mut self, changed: bool) -> bool {
        if changed {
            self.state_machine.request_redraw();
        }
        changed
    }

    pub(crate) fn active_tool(&self) -> EditorTool {
        self.state_machine.active_tool()
    }

    pub(crate) fn set_active_tool(&mut self, tool: EditorTool) {
        self.state_machine.set_tool(tool);
    }

    pub(crate) fn active_run_states(&self) -> HashSet<usize> {
        self.run.as_ref().map(RunSession::active_states).unwrap_or_default()
    }

    pub(crate) fn refresh_insight(&mut self) {
        self.record_history();
        let drawing = &self.drawing;
        self.state_machine.retain_selection(|id| drawing.node(id).is_some());
        let fingerprint = self.content_fingerprint();
        if self.run.as_ref().is_some_and(|session| session.is_stale(fingerprint)) {
            self.run = None;
        }
        if self.insight_fingerprint != Some(fingerprint) {
            self.insight_fingerprint = Some(fingerprint);
            self.state_machine.request_redraw();
            self.insight = self.compute_insight();
        }
        self.insight.unsaved = self.unsaved_given(fingerprint);
    }

    fn compute_insight(&mut self) -> TabInsight {
        if let Some(editor) = self.grammar() {
            return match editor.parse() {
                Ok(grammar) => TabInsight { production_count: grammar.productions().len(), ..TabInsight::default() },
                Err(problem) => TabInsight { problem: Some(problem), ..TabInsight::default() },
            };
        }
        let has_states = !self.drawing.states.is_empty();
        let Ok(automaton) = self.automaton() else {
            return TabInsight::default();
        };
        TabInsight {
            deterministic: automaton.is_deterministic(),
            problem: if has_states { automaton.validate().err() } else { None },
            ..TabInsight::default()
        }
    }

    pub(crate) fn content_fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.family().title().hash(&mut hasher);
        if let Some(editor) = self.grammar() {
            editor.text.trim().hash(&mut hasher);
            return hasher.finish();
        }
        self.drawing.hash_content(&mut hasher);
        hasher.finish()
    }

    fn is_empty_content(&self) -> bool {
        match self.grammar() {
            Some(editor) => editor.text.trim().is_empty(),
            None => self.drawing.is_empty(),
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

    fn sync_gui_to_machine(&mut self) {
        let fingerprint = self.content_fingerprint();
        if self.synced_fingerprint == Some(fingerprint) {
            return;
        }
        let TabKind::Automaton(automaton) = &mut self.kind else {
            return;
        };
        self.synced_fingerprint = Some(fingerprint);
        automaton.clear();
        let machine = automaton.structure_mut();
        for node in &self.drawing.states {
            machine.add_state_with_id_label(node.id as u64, &node.label);
        }
        for (&(from, to), labels) in &self.drawing.transitions {
            for label in labels {
                machine.add_transition(from as u64, to as u64, label.clone());
            }
        }
        for &state_id in &self.drawing.final_states {
            machine.make_final(state_id as u64);
        }
        if let Some(initial_id) = self.drawing.initial_state {
            machine.make_initial(initial_id as u64);
        }
    }

    fn load_machine_to_gui(&mut self) {
        let TabKind::Automaton(automaton) = &self.kind else {
            return;
        };
        let family = automaton.family();
        let machine = automaton.structure();
        let mut loaded: Vec<_> = machine.get_states_by_id_ref().iter().collect();
        loaded.sort_unstable_by_key(|(id, _)| **id);
        let mut states: Vec<StateNode> = loaded
            .into_iter()
            .map(|(id, state)| StateNode::new(*id as usize, iced::Point::new(100.0, 100.0), state.name.clone()))
            .collect();
        let mut transitions = Transitions::new();
        for (from_id, state) in machine.get_states_by_id_ref() {
            for (to_id, inputs) in state.iter_by_transition() {
                let entry = transitions.entry((*from_id as usize, *to_id as usize)).or_default();
                for label in inputs {
                    entry.insert(normalize_label(family, label));
                }
            }
        }
        let initial_state = machine.get_initial_state_id().map(|id| id as usize);
        let final_states = machine.get_final_states().iter().map(|&id| id as usize).collect();
        layout::arrange(&mut states, &transitions, initial_state);
        self.drawing = Drawing::new(states, transitions, initial_state, final_states);
        self.state_machine.set_scroll(iced::Vector::new(0.0, 0.0));
    }
}
