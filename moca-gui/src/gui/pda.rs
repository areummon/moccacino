/* Pushdown automaton run panel: stepping over configurations with a stack
 * lane instead of a tape strip. Mirrors the Turing panel's structure;
 * nondeterministic machines explore the frontier level by level. */
use std::collections::HashSet;

use iced::Task;
use iced::widget::{column, row};
use iced::{Alignment, Element, Length};

use moca_data::pushdown_automata::PdaConfiguration;

use super::message::Message;
use super::widgets::text;
use super::tab::{PdaNdFrontier, PdaRun, TabMachine};
use super::widgets::{self, CellSize, Outcome};
use crate::gui::theme::{self, Family};

const STACK_VISIBLE_ENTRIES: usize = 14;
/* Lane window rendered per branch in the nondeterministic view. */
const ND_LANES_VISIBLE: usize = 12;

impl super::app::App {
    pub(crate) fn active_tab_is_pda(&self) -> bool {
        self.get_active_tab().machine.machine_kind()
            == Some(moca_data::state_machine::MachineKind::Pushdown)
    }

    pub(crate) fn pda_input_changed(&mut self, text: String) -> Task<Message> {
        self.get_active_tab_mut().pda_input_text = text;
        Task::none()
    }

    pub(crate) fn pda_load_input(&mut self) -> Task<Message> {
        if !self.active_tab_is_pda() {
            return Task::none();
        }
        self.get_active_tab_mut().sync_gui_to_machine();
        // Malformed labels or a missing initial state abort the run early
        // with a precise message instead of halting silently mid-run.
        if let Err(problem) = self.get_active_tab().machine.validate() {
            self.error_message = Some(problem);
            return Task::none();
        }

        let input = self.get_active_tab().pda_input_text.clone();
        let machine = match &self.get_active_tab().machine {
            TabMachine::Pushdown(pda) => pda,
            _ => return Task::none(),
        };
        match machine.initial_configuration(&input) {
            Some(config) => {
                let tab = self.get_active_tab_mut();
                tab.pda_playing = false;
                if tab.machine.is_deterministic() {
                    tab.pda_frontier = None;
                    tab.pda_run = Some(PdaRun {
                        config,
                        input,
                        steps: 0,
                        finished: None,
                    });
                } else {
                    tab.pda_run = None;
                    tab.pda_frontier = Some(PdaNdFrontier {
                        level: 0,
                        alive: vec![config.clone()],
                        visited: HashSet::from([config]),
                        finished: None,
                    });
                }
            },
            None => {
                self.error_message = Some("The pushdown automaton has no initial state.".to_string());
            },
        }
        Task::none()
    }

    pub(crate) fn pda_step(&mut self) -> Task<Message> {
        if self.active_tab_is_pda() && !self.get_active_tab().machine.is_deterministic() {
            self.advance_pda_frontier_by_one_level();
        } else {
            self.advance_pda_by_one_step();
        }
        Task::none()
    }

    pub(crate) fn pda_toggle_play(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        tab.pda_playing = !tab.pda_playing;
        Task::none()
    }

    pub(crate) fn pda_reset(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        tab.pda_run = None;
        tab.pda_frontier = None;
        tab.pda_playing = false;
        Task::none()
    }

    /* One level of the nondeterministic frontier: every live branch takes a
     * parallel step, successors dedup against the visited set, and the
     * verdict mirrors the PDA traversal exactly (arrival acceptance before
     * expanding, empty next frontier = rejected). */
    pub(crate) fn advance_pda_frontier_by_one_level(&mut self) {
        if !self.active_tab_is_pda() {
            return;
        }
        let tab = self.get_active_tab_mut();
        let (machine, frontier) = (&tab.machine, &mut tab.pda_frontier);
        let pda = match machine {
            TabMachine::Pushdown(pda) => pda,
            _ => return,
        };
        let frontier = match frontier {
            Some(frontier) => frontier,
            None => return,
        };
        if frontier.finished.is_some() {
            return;
        }
        // Arrival semantics, mirroring `traverse`: an accepting configuration
        // accepts before any transition is applied.
        if frontier.alive.iter().any(|config| pda.is_accepting(config)) {
            frontier.finished = Some(true);
            return;
        }
        let mut next_alive: Vec<PdaConfiguration> = Vec::new();
        for config in frontier.alive.drain(..) {
            for successor in pda.step_all(&config) {
                if frontier.visited.insert(successor.clone()) {
                    next_alive.push(successor);
                }
            }
        }
        if next_alive.is_empty() {
            frontier.finished = Some(false);
        } else {
            frontier.alive = next_alive;
            frontier.level += 1;
        }
    }

    pub(crate) fn advance_pda_by_one_step(&mut self) {
        if !self.active_tab_is_pda() || !self.get_active_tab().machine.is_deterministic() {
            return;
        }
        let tab = self.get_active_tab_mut();
        let (machine, pda_run) = (&tab.machine, &mut tab.pda_run);
        let pda = match machine {
            TabMachine::Pushdown(pda) => pda,
            _ => return,
        };
        let run = match pda_run {
            Some(run) => run,
            None => return,
        };
        if run.finished.is_some() {
            return;
        }
        // Arrival semantics, mirroring `traverse`: an accepting state with
        // the input fully consumed accepts before any transition is applied.
        if pda.is_accepting(&run.config) {
            run.finished = Some(true);
            return;
        }
        let successors = pda.step_all(&run.config);
        match successors.len() {
            0 => run.finished = Some(false),
            1 => {
                run.config = successors.into_iter().next().expect("length checked");
                run.steps += 1;
            },
            // Only reachable on machines whose determinism flag missed an
            // ambiguity; refuse to pick a branch rather than lie.
            _ => {
                self.error_message = Some(
                    "Ambiguous step: several transitions apply at once. The deterministic flag missed an ambiguity, so step-by-step running stops here.".to_string(),
                );
            },
        }
    }

    pub(crate) fn create_pda_panel(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let (loaded, finished, playing) = tab.run_state();
        let controls = row![
            widgets::run_input(&tab.pda_input_text, Message::PdaInputChanged, Message::PdaLoadInput, loaded),
            widgets::run_controls(
                playing,
                loaded,
                finished,
                widgets::RunMessages {
                    play: Message::PdaTogglePlay,
                    step: Message::PdaStep,
                    reset: Message::PdaReset,
                },
            ),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let (status, body): (Element<'_, Message>, Element<'_, Message>) =
            if let Some(frontier) = &tab.pda_frontier {
                (
                    row![
                        widgets::outcome_chip(Outcome::from_finished(frontier.finished)),
                        widgets::stat("level", frontier.level.to_string()),
                        widgets::stat("branches", frontier.alive.len().to_string()),
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center)
                    .into(),
                    self.create_pda_nd_lane_view(frontier),
                )
            } else if let Some(run) = &tab.pda_run {
                let consumed = run.input.chars().count() - run.config.remaining_input().chars().count();
                (
                    row![
                        widgets::outcome_chip(Outcome::from_finished(run.finished)),
                        widgets::stat("state", self.state_label(run.config.state_id())),
                        widgets::stat("steps", run.steps.to_string()),
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center)
                    .into(),
                    // Input on the left, the stack (growing to the right,
                    // top highlighted) on the right.
                    row![
                        column![
                            widgets::caption("INPUT"),
                            widgets::strip(widgets::ribbon(&run.input, consumed, CellSize::Regular)),
                        ]
                        .spacing(6)
                        .width(Length::FillPortion(1)),
                        column![
                            widgets::caption("STACK  (top →)"),
                            widgets::strip(widgets::stack_strip(run.config.stack(), STACK_VISIBLE_ENTRIES)),
                        ]
                        .spacing(6)
                        .width(Length::FillPortion(1)),
                    ]
                    .spacing(24)
                    .into(),
                )
            } else {
                (
                    widgets::outcome_chip(Outcome::Idle),
                    widgets::hint(if tab.insight.deterministic {
                        "Type a word and press Load to watch the input and the stack step by step."
                    } else {
                        "Nondeterministic: Load a word to explore every branch, one level at a time."
                    }),
                )
            };

        widgets::run_dock(Family::Pushdown, Family::Pushdown.title(), tab.dock_collapsed, self.dock_is_compact(), controls.into(), status, body)
    }

    /* Level-by-level lane view for nondeterministic machines: one row per
     * live branch with its state, its stack (top on the right) and the
     * input still to consume. */
    fn create_pda_nd_lane_view(&self, frontier: &PdaNdFrontier) -> Element<'_, Message> {
        let rows = frontier
            .alive
            .iter()
            .take(ND_LANES_VISIBLE)
            .enumerate()
            .map(|(index, config)| {
                let stack_text = if config.stack().is_empty() {
                    "[ ]".to_string()
                } else {
                    format!("[{}]", config.stack().join(" "))
                };
                widgets::lane(
                    index,
                    self.state_label(config.state_id()),
                    row![
                        text(stack_text).font(theme::MONO).size(12).style(theme::text_dim),
                        widgets::ribbon(config.remaining_input(), 0, CellSize::Compact),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .into(),
                )
            })
            .collect();
        widgets::lanes(rows, frontier.alive.len().saturating_sub(ND_LANES_VISIBLE))
    }
}
