use std::collections::HashSet;
use std::hash::Hash;

use iced::Task;
use iced::widget::{column, row};
use iced::{Alignment, Element, Length};

use moca_data::finite_automata::{FiniteAutomata, FiniteConfiguration};
use moca_data::pushdown_automata::{PdaConfiguration, PushdownAutomata};
use moca_data::state::StateID;
use moca_data::state_machine::StateMachine;
use moca_data::turing_machine::{Configuration, TuringMachine};

use super::message::Message;
use super::tab::TabMachine;
use super::widgets::{self, text, CellSize, Outcome};
use crate::gui::theme::{self, Family};

const ND_LANES_VISIBLE: usize = 12;
const STACK_VISIBLE_ENTRIES: usize = 14;
const TAPE_CONTEXT: i64 = 12;
const AMBIGUOUS_STEP: &str = "Ambiguous step: several transitions apply at once. The deterministic flag missed an ambiguity, so step-by-step running stops here.";

pub(crate) trait RunConfig: Clone + Eq + Hash {
    fn state(&self) -> StateID;
}

impl RunConfig for FiniteConfiguration {
    fn state(&self) -> StateID {
        self.state_id()
    }
}

impl RunConfig for PdaConfiguration {
    fn state(&self) -> StateID {
        self.state_id()
    }
}

impl RunConfig for Configuration {
    fn state(&self) -> StateID {
        self.state_id()
    }
}

trait Runnable {
    type Config: RunConfig;
    fn start(&self, input: &str) -> Option<Self::Config>;
    fn accepts_at(&self, config: &Self::Config) -> bool;
    fn successors(&self, config: &Self::Config) -> Vec<Self::Config>;
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
}

impl Runnable for TuringMachine {
    type Config = Configuration;
    fn start(&self, input: &str) -> Option<Self::Config> {
        self.initial_configuration(input)
    }
    fn accepts_at(&self, config: &Self::Config) -> bool {
        self.get_final_states().contains(&config.state_id())
    }
    fn successors(&self, config: &Self::Config) -> Vec<Self::Config> {
        self.step_all(config)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Trace<C> {
    input: String,
    branching: bool,
    alive: Vec<C>,
    visited: HashSet<C>,
    count: u64,
    finished: Option<bool>,
}

impl<C: RunConfig> Trace<C> {
    fn start<M: Runnable<Config = C>>(machine: &M, input: String, branching: bool) -> Option<Self> {
        let config = machine.start(&input)?;
        Some(Trace {
            input,
            branching,
            alive: vec![config.clone()],
            visited: HashSet::from([config]),
            count: 0,
            finished: None,
        })
    }

    fn advance<M: Runnable<Config = C>>(&mut self, machine: &M) -> Result<(), &'static str> {
        if self.finished.is_some() {
            return Ok(());
        }
        if self.alive.iter().any(|config| machine.accepts_at(config)) {
            self.finished = Some(true);
            return Ok(());
        }
        let mut next = Vec::new();
        for config in &self.alive {
            for successor in machine.successors(config) {
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
        self.alive.iter().map(|config| config.state() as usize).collect()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Run {
    Finite(Trace<FiniteConfiguration>),
    Pushdown(Trace<PdaConfiguration>),
    Turing(Trace<Configuration>),
}

impl Run {
    fn start(machine: &TabMachine, input: String) -> Option<Self> {
        let branching = !machine.is_deterministic();
        match machine {
            TabMachine::Finite(finite) => Trace::start(finite, input, branching).map(Run::Finite),
            TabMachine::Pushdown(pda) => Trace::start(pda, input, branching).map(Run::Pushdown),
            TabMachine::Turing(turing) => Trace::start(turing, input, branching).map(Run::Turing),
            TabMachine::Grammar(_) => None,
        }
    }

    fn advance(&mut self, machine: &TabMachine) -> Result<(), &'static str> {
        match (self, machine) {
            (Run::Finite(trace), TabMachine::Finite(finite)) => trace.advance(finite),
            (Run::Pushdown(trace), TabMachine::Pushdown(pda)) => trace.advance(pda),
            (Run::Turing(trace), TabMachine::Turing(turing)) => trace.advance(turing),
            _ => Ok(()),
        }
    }

    pub(crate) fn finished(&self) -> Option<bool> {
        match self {
            Run::Finite(trace) => trace.finished,
            Run::Pushdown(trace) => trace.finished,
            Run::Turing(trace) => trace.finished,
        }
    }

    pub(crate) fn active_states(&self) -> HashSet<usize> {
        match self {
            Run::Finite(trace) => trace.active_states(),
            Run::Pushdown(trace) => trace.active_states(),
            Run::Turing(trace) => trace.active_states(),
        }
    }
}

impl super::app::App {
    pub(crate) fn run_input_changed(&mut self, text: String) -> Task<Message> {
        self.get_active_tab_mut().run_input = text;
        Task::none()
    }

    pub(crate) fn run_load(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        if tab.machine.is_grammar() {
            return Task::none();
        }
        tab.sync_gui_to_machine();
        let started = tab
            .machine
            .validate()
            .and_then(|()| {
                Run::start(&tab.machine, tab.run_input.clone())
                    .ok_or_else(|| "The machine has no initial state.".to_string())
            });
        match started {
            Ok(run) => {
                tab.run = Some(run);
                tab.playing = false;
            }
            Err(problem) => self.error_message = Some(problem),
        }
        Task::none()
    }

    pub(crate) fn run_step(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        if let Some(Err(problem)) = tab.run.as_mut().map(|run| run.advance(&tab.machine)) {
            self.error_message = Some(problem.to_string());
        }
        Task::none()
    }

    pub(crate) fn run_toggle_play(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        tab.playing = !tab.playing;
        Task::none()
    }

    pub(crate) fn run_reset(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        tab.run = None;
        tab.playing = false;
        Task::none()
    }

    pub(crate) fn run_tick(&mut self) -> Task<Message> {
        if self.run_tick_interval().is_some() {
            return self.run_step();
        }
        Task::none()
    }

    pub(crate) fn create_run_panel(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let family = tab.machine.family();
        let (loaded, finished, playing) = tab.run_state();
        let controls = row![
            widgets::run_input(&tab.run_input, Message::RunInputChanged, Message::RunLoad, loaded),
            widgets::run_controls(
                playing,
                loaded,
                finished,
                widgets::RunMessages {
                    play: Message::RunTogglePlay,
                    step: Message::RunStep,
                    reset: Message::RunReset,
                },
            ),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let (status, body) = match &tab.run {
            None => (
                widgets::outcome_chip(Outcome::Idle),
                widgets::hint(idle_hint(family, tab.insight.deterministic)),
            ),
            Some(Run::Finite(trace)) => (self.run_status(trace), self.finite_body(trace)),
            Some(Run::Pushdown(trace)) => (self.run_status(trace), self.pda_body(trace)),
            Some(Run::Turing(trace)) => (self.run_status(trace), self.tm_body(trace)),
        };

        widgets::run_dock(family, family.title(), tab.dock_collapsed, self.dock_is_compact(), controls.into(), status, body)
    }

    fn run_status<C: RunConfig>(&self, trace: &Trace<C>) -> Element<'_, Message> {
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
                widgets::stat("state", self.state_label(trace.alive[0].state())),
                widgets::stat("steps", trace.count.to_string()),
            ]
        };
        stats.spacing(12).align_y(Alignment::Center).into()
    }

    fn lanes<'a, C: RunConfig>(
        &self,
        trace: &'a Trace<C>,
        lane_body: impl Fn(&'a C) -> Element<'a, Message>,
    ) -> Element<'a, Message> {
        let rows = trace
            .alive
            .iter()
            .take(ND_LANES_VISIBLE)
            .enumerate()
            .map(|(index, config)| widgets::lane(index, self.state_label(config.state()), lane_body(config)))
            .collect();
        widgets::lanes(rows, trace.alive.len().saturating_sub(ND_LANES_VISIBLE))
    }

    fn finite_body<'a>(&self, trace: &'a Trace<FiniteConfiguration>) -> Element<'a, Message> {
        if trace.branching {
            return self.lanes(trace, |config| widgets::ribbon(config.remaining_input(), 0, CellSize::Compact));
        }
        widgets::strip(widgets::ribbon(&trace.input, consumed(trace, trace.alive[0].remaining_input()), CellSize::Regular))
    }

    fn pda_body<'a>(&self, trace: &'a Trace<PdaConfiguration>) -> Element<'a, Message> {
        if trace.branching {
            return self.lanes(trace, |config| {
                let stack_text = if config.stack().is_empty() {
                    "[ ]".to_string()
                } else {
                    format!("[{}]", config.stack().join(" "))
                };
                row![
                    text(stack_text).font(theme::MONO).size(12).style(theme::text_dim),
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

    fn tm_body<'a>(&self, trace: &'a Trace<Configuration>) -> Element<'a, Message> {
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

    pub(crate) fn state_label(&self, state_id: u64) -> String {
        self.get_active_tab()
            .states
            .iter()
            .find(|node| node.id == state_id as usize)
            .map(|node| node.label.to_string())
            .unwrap_or_else(|| format!("q{state_id}"))
    }
}

fn consumed<C>(trace: &Trace<C>, remaining: &str) -> usize {
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
