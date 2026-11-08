use std::collections::HashSet;
use std::hash::Hash;

use iced::Task;
use iced::widget::{column, row};
use iced::{Alignment, Element, Length};

use moca_data::finite_automata::{FiniteAutomata, FiniteConfiguration};
use moca_data::pushdown_automata::{PdaConfiguration, PushdownAutomata};
use moca_data::state::StateID;
use moca_data::turing_machine::{Configuration, TuringMachine};

use super::message::Message;
use super::automaton::Automaton;
use super::widgets::{self, text, CellSize, Outcome};
use crate::theme;
use crate::family::Family;

const ND_LANES_VISIBLE: usize = 12;
const STACK_VISIBLE_ENTRIES: usize = 14;
const TAPE_CONTEXT: i64 = 12;
const AMBIGUOUS_STEP: &str = "Ambiguous step: several transitions apply at once. The deterministic flag missed an ambiguity, so step-by-step running stops here.";

trait Runnable {
    type Config: Clone + Eq + Hash;
    fn start(&self, input: &str) -> Option<Self::Config>;
    fn accepts_at(&self, config: &Self::Config) -> bool;
    fn successors(&self, config: &Self::Config) -> Vec<Self::Config>;
    fn state_of(config: &Self::Config) -> StateID;
}

impl Runnable for FiniteAutomata {
    type Config = FiniteConfiguration;
    fn start(&self, input: &str) -> Option<Self::Config> {
        self.initial_configuration(input)
    }
    fn accepts_at(&self, config: &Self::Config) -> bool {
        self.is_accepting(config)
    }
    fn successors(&self, config: &Self::Config) -> Vec<Self::Config> {
        self.step_all(config)
    }
    fn state_of(config: &Self::Config) -> StateID {
        config.state_id()
    }
}

impl Runnable for PushdownAutomata {
    type Config = PdaConfiguration;
    fn start(&self, input: &str) -> Option<Self::Config> {
        self.initial_configuration(input)
    }
    fn accepts_at(&self, config: &Self::Config) -> bool {
        self.is_accepting(config)
    }
    fn successors(&self, config: &Self::Config) -> Vec<Self::Config> {
        self.step_all(config)
    }
    fn state_of(config: &Self::Config) -> StateID {
        config.state_id()
    }
}

impl Runnable for TuringMachine {
    type Config = Configuration;
    fn start(&self, input: &str) -> Option<Self::Config> {
        self.initial_configuration(input)
    }
    fn accepts_at(&self, config: &Self::Config) -> bool {
        self.is_accepting(config)
    }
    fn successors(&self, config: &Self::Config) -> Vec<Self::Config> {
        self.step_all(config)
    }
    fn state_of(config: &Self::Config) -> StateID {
        config.state_id()
    }
}

struct Trace<M: Runnable> {
    machine: M,
    input: String,
    branching: bool,
    alive: Vec<M::Config>,
    visited: HashSet<M::Config>,
    count: u64,
    finished: Option<bool>,
}

impl<M: Runnable> Trace<M> {
    fn start(machine: M, input: String, branching: bool) -> Option<Self> {
        let config = machine.start(&input)?;
        Some(Trace {
            machine,
            input,
            branching,
            alive: vec![config.clone()],
            visited: HashSet::from([config]),
            count: 0,
            finished: None,
        })
    }

    fn advance(&mut self) -> Result<(), &'static str> {
        if self.finished.is_some() {
            return Ok(());
        }
        if self.alive.iter().any(|config| self.machine.accepts_at(config)) {
            self.finished = Some(true);
            return Ok(());
        }
        let mut next = Vec::new();
        for config in &self.alive {
            for successor in self.machine.successors(config) {
                if self.visited.insert(successor.clone()) {
                    next.push(successor);
                }
            }
        }
        if !self.branching && next.len() > 1 {
            for config in &next {
                self.visited.remove(config);
            }
            return Err(AMBIGUOUS_STEP);
        }
        if next.is_empty() {
            self.finished = Some(false);
        } else {
            self.alive = next;
            self.count += 1;
        }
        Ok(())
    }

    fn active_states(&self) -> HashSet<usize> {
        self.alive.iter().map(|config| M::state_of(config) as usize).collect()
    }
}

enum Run {
    Finite(Trace<FiniteAutomata>),
    Pushdown(Trace<PushdownAutomata>),
    Turing(Trace<TuringMachine>),
}

impl Run {
    fn start(automaton: &Automaton, input: String) -> Option<Self> {
        let branching = !automaton.is_deterministic();
        match automaton {
            Automaton::Finite(finite) => Trace::start(finite.clone(), input, branching).map(Run::Finite),
            Automaton::Pushdown(pda) => Trace::start(pda.clone(), input, branching).map(Run::Pushdown),
            Automaton::Turing(turing) => Trace::start(turing.clone(), input, branching).map(Run::Turing),
        }
    }

    fn advance(&mut self) -> Result<(), &'static str> {
        match self {
            Run::Finite(trace) => trace.advance(),
            Run::Pushdown(trace) => trace.advance(),
            Run::Turing(trace) => trace.advance(),
        }
    }

    fn finished(&self) -> Option<bool> {
        match self {
            Run::Finite(trace) => trace.finished,
            Run::Pushdown(trace) => trace.finished,
            Run::Turing(trace) => trace.finished,
        }
    }

    fn active_states(&self) -> HashSet<usize> {
        match self {
            Run::Finite(trace) => trace.active_states(),
            Run::Pushdown(trace) => trace.active_states(),
            Run::Turing(trace) => trace.active_states(),
        }
    }
}

pub(crate) struct RunSession {
    run: Run,
    playing: bool,
    fingerprint: u64,
}

impl RunSession {
    pub(crate) fn active_states(&self) -> HashSet<usize> {
        self.run.active_states()
    }

    pub(crate) fn finished(&self) -> bool {
        self.run.finished().is_some()
    }

    pub(crate) fn playing(&self) -> bool {
        self.playing && !self.finished()
    }

    pub(crate) fn is_stale(&self, fingerprint: u64) -> bool {
        self.fingerprint != fingerprint
    }
}

impl super::app::App {
    pub(crate) fn run_load(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        let input = tab.run_input.clone();
        let automaton = match tab.automaton() {
            Ok(automaton) => automaton,
            Err(problem) => {
                self.error_message = Some(problem);
                return Task::none();
            }
        };
        let started = automaton.validate().and_then(|()| {
            Run::start(automaton, input).ok_or_else(|| "The machine has no initial state.".to_string())
        });
        match started {
            Ok(run) => {
                let fingerprint = tab.content_fingerprint();
                tab.run = Some(RunSession { run, playing: false, fingerprint });
            }
            Err(problem) => self.error_message = Some(problem),
        }
        Task::none()
    }

    pub(crate) fn run_step(&mut self) -> Task<Message> {
        if let Some(session) = self.get_active_tab_mut().run.as_mut() {
            if let Err(problem) = session.run.advance() {
                session.playing = false;
                self.error_message = Some(problem.to_string());
            }
        }
        Task::none()
    }

    pub(crate) fn run_toggle_play(&mut self) -> Task<Message> {
        if let Some(session) = self.get_active_tab_mut().run.as_mut() {
            session.playing = !session.playing;
        }
        Task::none()
    }

    pub(crate) fn run_tick(&mut self) -> Task<Message> {
        if self.run_playing() {
            return self.run_step();
        }
        Task::none()
    }

    pub(crate) fn create_run_panel(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let family = tab.family();
        let loaded = tab.run.is_some();
        let finished = tab.run.as_ref().is_some_and(RunSession::finished);
        let controls = row![
            widgets::run_input(&tab.run_input, loaded),
            widgets::run_controls(self.run_playing(), loaded, finished),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let (status, body) = match tab.run.as_ref().map(|session| &session.run) {
            None => (
                widgets::outcome_chip(Outcome::Idle),
                widgets::hint(idle_hint(family, tab.insight.deterministic)),
            ),
            Some(Run::Finite(trace)) => (self.run_status(trace), self.finite_body(trace)),
            Some(Run::Pushdown(trace)) => (self.run_status(trace), self.pda_body(trace)),
            Some(Run::Turing(trace)) => (self.run_status(trace), self.tm_body(trace)),
        };

        widgets::run_dock(family, tab.dock_collapsed, self.dock_is_compact(), controls.into(), status, body)
    }

    fn run_status<M: Runnable>(&self, trace: &Trace<M>) -> Element<'_, Message> {
        let chip = widgets::outcome_chip(Outcome::from_finished(trace.finished));
        let stats = if trace.branching {
            row![
                chip,
                widgets::stat("level", trace.count.to_string()),
                widgets::stat("branches", trace.alive.len().to_string()),
            ]
        } else {
            row![
                chip,
                widgets::stat("state", self.state_label(M::state_of(&trace.alive[0]))),
                widgets::stat("steps", trace.count.to_string()),
            ]
        };
        stats.spacing(12).align_y(Alignment::Center).into()
    }

    fn lanes<'a, M: Runnable>(
        &self,
        trace: &'a Trace<M>,
        lane_body: impl Fn(&'a M::Config) -> Element<'a, Message>,
    ) -> Element<'a, Message> {
        let rows = trace
            .alive
            .iter()
            .take(ND_LANES_VISIBLE)
            .enumerate()
            .map(|(index, config)| widgets::lane(index, self.state_label(M::state_of(config)), lane_body(config)))
            .collect();
        widgets::lanes(rows, trace.alive.len().saturating_sub(ND_LANES_VISIBLE))
    }

    fn finite_body<'a>(&self, trace: &'a Trace<FiniteAutomata>) -> Element<'a, Message> {
        if trace.branching {
            return self.lanes(trace, |config| widgets::ribbon(config.remaining_input(), 0, CellSize::Compact));
        }
        widgets::strip(widgets::ribbon(&trace.input, consumed(trace, trace.alive[0].remaining_input()), CellSize::Regular))
    }

    fn pda_body<'a>(&self, trace: &'a Trace<PushdownAutomata>) -> Element<'a, Message> {
        if trace.branching {
            return self.lanes(trace, |config| {
                let stack_text = if config.stack().is_empty() {
                    "[ ]".to_string()
                } else {
                    format!("[{}]", config.stack().join(" "))
                };
                row![
                    text(stack_text).font(theme::MONO).size(theme::TEXT_SM).style(theme::text_dim),
                    widgets::ribbon(config.remaining_input(), 0, CellSize::Compact),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .into()
            });
        }
        let config = &trace.alive[0];
        row![
            column![
                widgets::caption("INPUT"),
                widgets::strip(widgets::ribbon(&trace.input, consumed(trace, config.remaining_input()), CellSize::Regular)),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            column![
                widgets::caption("STACK  (top →)"),
                widgets::strip(widgets::stack_strip(config.stack(), STACK_VISIBLE_ENTRIES)),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
        ]
        .spacing(24)
        .into()
    }

    fn tm_body<'a>(&self, trace: &'a Trace<TuringMachine>) -> Element<'a, Message> {
        if trace.branching {
            return self.lanes(trace, |config| {
                let (window, head_offset) = config.tape().snapshot(TAPE_CONTEXT);
                widgets::tape(&window, head_offset, CellSize::Compact)
            });
        }
        let (window, head_offset) = trace.alive[0].tape().snapshot(TAPE_CONTEXT);
        column![
            widgets::caption("TAPE"),
            widgets::strip(widgets::tape(&window, head_offset, CellSize::Regular)),
        ]
        .spacing(6)
        .into()
    }

    fn state_label(&self, state_id: StateID) -> String {
        self.get_active_tab()
            .node(state_id as usize)
            .map_or_else(|| format!("q{state_id}"), |node| node.label.clone())
    }
}

fn consumed<M: Runnable>(trace: &Trace<M>, remaining: &str) -> usize {
    trace.input.chars().count() - remaining.chars().count()
}

fn idle_hint(family: Family, deterministic: bool) -> &'static str {
    match (family, deterministic) {
        (Family::Finite, true) => "Type a word and press Load, then step through its consumption symbol by symbol.",
        (Family::Finite, false) => "Nondeterministic: Load a word to explore every branch, one transition level at a time.",
        (Family::Pushdown, true) => "Type a word and press Load to watch the input and the stack step by step.",
        (Family::Turing, true) => "Type a word and press Load to watch the head move along the tape.",
        _ => "Nondeterministic: Load a word to explore every branch, one level at a time.",
    }
}
