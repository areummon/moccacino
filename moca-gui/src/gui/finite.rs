/* Finite automaton run panel: single-branch stepping for deterministic
 * machines, level-by-level branches for nondeterministic ones. Mirrors the
 * Turing panel's structure. */
use std::collections::HashSet;

use iced::Task;
use iced::widget::row;
use iced::{Alignment, Element};

use moca_data::finite_automata::FiniteConfiguration;

use super::message::Message;
use super::tab::{FiniteNdFrontier, FiniteRun, TabMachine};
use super::widgets::{self, CellSize, Outcome};
use crate::gui::theme::Family;

/* Lane window rendered per branch in the nondeterministic view. */
const ND_LANES_VISIBLE: usize = 12;

impl super::app::App {
    pub(crate) fn active_tab_is_finite(&self) -> bool {
        self.get_active_tab().machine.machine_kind()
            == Some(moca_data::state_machine::MachineKind::Finite)
    }

    pub(crate) fn finite_input_changed(&mut self, text: String) -> Task<Message> {
        self.get_active_tab_mut().finite_input_text = text;
        Task::none()
    }

    pub(crate) fn finite_load_input(&mut self) -> Task<Message> {
        if !self.active_tab_is_finite() {
            return Task::none();
        }
        self.get_active_tab_mut().sync_gui_to_machine();
        // Malformed labels or a missing initial state abort the run early
        // with a precise message instead of halting silently mid-run.
        if let Err(problem) = self.get_active_tab().machine.validate() {
            self.error_message = Some(problem);
            return Task::none();
        }

        let input = self.get_active_tab().finite_input_text.clone();
        match &self.get_active_tab().machine {
            TabMachine::Finite(finite) => {
                match finite.initial_configuration(&input) {
                    Some(config) => {
                        let tab = self.get_active_tab_mut();
                        if tab.machine.is_deterministic() {
                            tab.finite_frontier = None;
                            tab.finite_run = Some(FiniteRun {
                                visited: HashSet::from([config.clone()]),
                                config,
                                input: input.clone(),
                                steps: 0,
                                finished: None,
                            });
                        } else {
                            tab.finite_run = None;
                            tab.finite_frontier = Some(FiniteNdFrontier {
                                level: 0,
                                alive: vec![config.clone()],
                                visited: HashSet::from([config]),
                                finished: None,
                            });
                        }
                    },
                    None => {
                        self.error_message = Some("The automaton has no initial state.".to_string());
                    },
                }
            },
            _ => (),
        }
        Task::none()
    }

    pub(crate) fn finite_step(&mut self) -> Task<Message> {
        if self.active_tab_is_finite() && !self.get_active_tab().machine.is_deterministic() {
            self.advance_finite_frontier_by_one_level();
        } else {
            self.advance_finite_by_one_step();
        }
        Task::none()
    }

    pub(crate) fn finite_toggle_play(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        tab.finite_playing = !tab.finite_playing;
        Task::none()
    }

    pub(crate) fn finite_reset(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        tab.finite_run = None;
        tab.finite_frontier = None;
        tab.finite_playing = false;
        Task::none()
    }

    /* One level of the nondeterministic frontier: every live branch takes
     * one transition, successors dedup against the visited set, and the
     * verdict mirrors `check_input` exactly (a final state with the input
     * fully consumed accepts before expanding; an empty frontier rejects). */
    pub(crate) fn advance_finite_frontier_by_one_level(&mut self) {
        if !self.active_tab_is_finite() {
            return;
        }
        let tab = self.get_active_tab_mut();
        let (machine, frontier) = (&tab.machine, &mut tab.finite_frontier);
        let finite = match machine {
            TabMachine::Finite(finite) => finite,
            _ => return,
        };
        let frontier = match frontier {
            Some(frontier) => frontier,
            None => return,
        };
        if frontier.finished.is_some() {
            return;
        }
        if frontier.alive.iter().any(|config| finite.is_accepting(config)) {
            frontier.finished = Some(true);
            return;
        }
        let mut next_alive: Vec<FiniteConfiguration> = Vec::new();
        for config in frontier.alive.drain(..) {
            for successor in finite.step_all(&config) {
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

    pub(crate) fn advance_finite_by_one_step(&mut self) {
        if !self.active_tab_is_finite() || !self.get_active_tab().machine.is_deterministic() {
            return;
        }
        // Split borrows so the machine and its run state can be touched at
        // the same time.
        let tab = self.get_active_tab_mut();
        let (machine, finite_run) = (&tab.machine, &mut tab.finite_run);
        let finite = match machine {
            TabMachine::Finite(finite) => finite,
            _ => return,
        };
        let run = match finite_run {
            Some(run) => run,
            None => return,
        };
        if run.finished.is_some() {
            return;
        }
        // Whole-input semantics, mirroring `check_input`: a final state
        // accepts only once the input is fully consumed.
        if finite.is_accepting(&run.config) {
            run.finished = Some(true);
            return;
        }
        let successors = finite.step_all(&run.config);
        match successors.len() {
            0 => run.finished = Some(false),
            1 => {
                let next = successors.into_iter().next().expect("length checked");
                if run.visited.insert(next.clone()) {
                    run.config = next;
                    run.steps += 1;
                } else {
                    // ε-cycle: `check_input`'s memoization abandons the
                    // branch, so the single-branch run rejects here.
                    run.finished = Some(false);
                }
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

    pub(crate) fn create_finite_panel(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let (loaded, finished, playing) = tab.run_state();
        let controls = row![
            widgets::run_input(&tab.finite_input_text, Message::FiniteInputChanged, Message::FiniteLoadInput, loaded),
            widgets::run_controls(
                playing,
                loaded,
                finished,
                widgets::RunMessages {
                    play: Message::FiniteTogglePlay,
                    step: Message::FiniteStep,
                    reset: Message::FiniteReset,
                },
            ),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let (status, body): (Element<'_, Message>, Element<'_, Message>) =
            if let Some(frontier) = &tab.finite_frontier {
                (
                    row![
                        widgets::outcome_chip(Outcome::from_finished(frontier.finished)),
                        widgets::stat("level", frontier.level.to_string()),
                        widgets::stat("branches", frontier.alive.len().to_string()),
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center)
                    .into(),
                    self.create_finite_nd_lane_view(frontier),
                )
            } else if let Some(run) = &tab.finite_run {
                // Input ribbon: the consumed prefix dimmed, the next symbol
                // highlighted.
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
                    widgets::strip(widgets::ribbon(&run.input, consumed, CellSize::Regular)),
                )
            } else {
                (
                    widgets::outcome_chip(Outcome::Idle),
                    widgets::hint(if tab.insight.deterministic {
                        "Type a word and press Load, then step through its consumption symbol by symbol."
                    } else {
                        "Nondeterministic: Load a word to explore every branch, one transition level at a time."
                    }),
                )
            };

        widgets::run_dock(Family::Finite, Family::Finite.title(), tab.dock_collapsed, self.dock_is_compact(), controls.into(), status, body)
    }

    /* Level-by-level lane view for nondeterministic machines: one row per
     * live branch with its state and the input still to consume. */
    fn create_finite_nd_lane_view(&self, frontier: &FiniteNdFrontier) -> Element<'_, Message> {
        let rows = frontier
            .alive
            .iter()
            .take(ND_LANES_VISIBLE)
            .enumerate()
            .map(|(index, config)| {
                widgets::lane(
                    index,
                    self.state_label(config.state_id()),
                    widgets::ribbon(config.remaining_input(), 0, CellSize::Compact),
                )
            })
            .collect();
        widgets::lanes(rows, frontier.alive.len().saturating_sub(ND_LANES_VISIBLE))
    }

    /* Display label of a machine state id: the canvas name when drawn. */
    pub(crate) fn state_label(&self, state_id: u64) -> String {
        self.get_active_tab()
            .states
            .iter()
            .find(|node| node.id == state_id as usize)
            .map(|node| node.label.to_string())
            .unwrap_or_else(|| format!("q{}", state_id))
    }
}
