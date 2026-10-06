use std::collections::HashSet;

use iced::Task;
use iced::widget::{column, row};
use iced::{Alignment, Element};

use moca_data::state_machine::StateMachine;
use moca_data::turing_machine::{Configuration, RunOutcome};

use super::message::Message;
use super::tab::{NdFrontier, TabMachine, TmRun};
use super::widgets::{self, CellSize, Outcome};
use crate::gui::theme::Family;

const TAPE_CONTEXT: i64 = 12;
const ND_LANES_VISIBLE: usize = 12;

impl super::app::App {
    pub(crate) fn active_tab_is_turing(&self) -> bool {
        self.get_active_tab().machine.machine_kind()
            == Some(moca_data::state_machine::MachineKind::Turing)
    }

    pub(crate) fn tm_input_changed(&mut self, text: String) -> Task<Message> {
        self.get_active_tab_mut().tm_input_text = text;
        Task::none()
    }

    pub(crate) fn tm_load_input(&mut self) -> Task<Message> {
        if !self.active_tab_is_turing() {
            return Task::none();
        }
        self.get_active_tab_mut().sync_gui_to_machine();
        if let Err(problem) = self.get_active_tab().machine.validate() {
            self.error_message = Some(problem);
            return Task::none();
        }

        let input = self.get_active_tab().tm_input_text.clone();
        match &self.get_active_tab().machine {
            TabMachine::Turing(turing) => {
                match turing.initial_configuration(&input) {
                    Some(config) => {
                        let tab = self.get_active_tab_mut();
                        tab.tm_playing = false;
                        if tab.machine.is_deterministic() {
                            tab.tm_frontier = None;
                            tab.tm_run = Some(TmRun {
                                config,
                                steps: 0,
                                finished: None,
                            });
                        } else {
                            tab.tm_run = None;
                            tab.tm_frontier = Some(NdFrontier {
                                level: 0,
                                alive: vec![config.clone()],
                                visited: HashSet::from([config]),
                                finished: None,
                            });
                        }
                    },
                    None => {
                        self.error_message = Some("The machine has no initial state.".to_string());
                    },
                }
            },
            _ => (),
        }
        Task::none()
    }

    pub(crate) fn tm_step(&mut self) -> Task<Message> {
        if self.active_tab_is_turing() && !self.get_active_tab().machine.is_deterministic() {
            self.advance_tm_frontier_by_one_level();
        } else {
            self.advance_tm_by_one_step();
        }
        Task::none()
    }

    pub(crate) fn tm_toggle_play(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        tab.tm_playing = !tab.tm_playing;
        Task::none()
    }

    pub(crate) fn run_tick(&mut self) -> Task<Message> {
        if self.run_tick_interval().is_some() {
            if self.active_tab_is_turing() {
                if self.get_active_tab().machine.is_deterministic() {
                    self.advance_tm_by_one_step();
                } else {
                    self.advance_tm_frontier_by_one_level();
                }
            } else if self.active_tab_is_pda() {
                if self.get_active_tab().machine.is_deterministic() {
                    self.advance_pda_by_one_step();
                } else {
                    self.advance_pda_frontier_by_one_level();
                }
            } else if self.active_tab_is_finite() {
                if self.get_active_tab().machine.is_deterministic() {
                    self.advance_finite_by_one_step();
                } else {
                    self.advance_finite_frontier_by_one_level();
                }
            }
        }
        Task::none()
    }

    pub(crate) fn tm_reset(&mut self) -> Task<Message> {
        let tab = self.get_active_tab_mut();
        tab.tm_run = None;
        tab.tm_frontier = None;
        tab.tm_playing = false;
        Task::none()
    }

    fn advance_tm_frontier_by_one_level(&mut self) {
        if !self.active_tab_is_turing() {
            return;
        }
        let tab = self.get_active_tab_mut();
        let (machine, frontier) = (&tab.machine, &mut tab.tm_frontier);
        let turing = match machine {
            TabMachine::Turing(turing) => turing,
            _ => return,
        };
        let frontier = match frontier {
            Some(frontier) => frontier,
            None => return,
        };
        if frontier.finished.is_some() {
            return;
        }
        let finals = turing.get_final_states();
        if frontier.alive.iter().any(|config| finals.contains(&config.state_id())) {
            frontier.finished = Some(RunOutcome::Accepted);
            return;
        }
        let mut next_alive: Vec<Configuration> = Vec::new();
        for config in frontier.alive.drain(..) {
            for successor in turing.step_all(&config) {
                if frontier.visited.insert(successor.clone()) {
                    next_alive.push(successor);
                }
            }
        }
        if next_alive.is_empty() {
            frontier.finished = Some(RunOutcome::Rejected);
        } else {
            frontier.alive = next_alive;
            frontier.level += 1;
        }
    }

    pub(crate) fn advance_tm_by_one_step(&mut self) {
        if !self.active_tab_is_turing() || !self.get_active_tab().machine.is_deterministic() {
            return;
        }
        let tab = self.get_active_tab_mut();
        let (machine, tm_run) = (&tab.machine, &mut tab.tm_run);
        let turing = match machine {
            TabMachine::Turing(turing) => turing,
            _ => return,
        };
        let run = match tm_run {
            Some(run) => run,
            None => return,
        };
        if run.finished.is_some() {
            return;
        }
        if turing.get_final_states().contains(&run.config.state_id()) {
            run.finished = Some(RunOutcome::Accepted);
            return;
        }
        match turing.step(&run.config) {
            Some(next_config) => {
                run.config = next_config;
                run.steps += 1;
            },
            None => run.finished = Some(RunOutcome::Rejected),
        }
    }

    pub(crate) fn create_tm_panel(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let (loaded, finished, playing) = tab.run_state();
        let controls = row![
            widgets::run_input(&tab.tm_input_text, Message::TmInputChanged, Message::TmLoadInput, loaded),
            widgets::run_controls(
                playing,
                loaded,
                finished,
                widgets::RunMessages {
                    play: Message::TmTogglePlay,
                    step: Message::TmStep,
                    reset: Message::TmReset,
                },
            ),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let (status, body): (Element<'_, Message>, Element<'_, Message>) =
            if let Some(frontier) = &tab.tm_frontier {
                (
                    row![
                        widgets::outcome_chip(outcome(frontier.finished)),
                        widgets::stat("level", frontier.level.to_string()),
                        widgets::stat("branches", frontier.alive.len().to_string()),
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center)
                    .into(),
                    self.create_nd_lane_view(frontier),
                )
            } else if let Some(run) = &tab.tm_run {
                let (window, head_offset) = run.config.tape().snapshot(TAPE_CONTEXT);
                (
                    row![
                        widgets::outcome_chip(outcome(run.finished)),
                        widgets::stat("state", self.state_label(run.config.state_id())),
                        widgets::stat("steps", run.steps.to_string()),
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center)
                    .into(),
                    column![
                        widgets::caption("TAPE"),
                        widgets::strip(widgets::tape(&window, head_offset, CellSize::Regular)),
                    ]
                    .spacing(6)
                    .into(),
                )
            } else {
                (
                    widgets::outcome_chip(Outcome::Idle),
                    widgets::hint(if tab.insight.deterministic {
                        "Type a word and press Load to watch the head move along the tape."
                    } else {
                        "Nondeterministic: Load a word to explore every branch, one level at a time."
                    }),
                )
            };

        widgets::run_dock(Family::Turing, Family::Turing.title(), tab.dock_collapsed, self.dock_is_compact(), controls.into(), status, body)
    }

    fn create_nd_lane_view(&self, frontier: &NdFrontier) -> Element<'_, Message> {
        let rows = frontier
            .alive
            .iter()
            .take(ND_LANES_VISIBLE)
            .enumerate()
            .map(|(index, config)| {
                let (window, head_offset) = config.tape().snapshot(TAPE_CONTEXT);
                widgets::lane(
                    index,
                    self.state_label(config.state_id()),
                    widgets::tape(&window, head_offset, CellSize::Compact),
                )
            })
            .collect();
        widgets::lanes(rows, frontier.alive.len().saturating_sub(ND_LANES_VISIBLE))
    }
}

fn outcome(finished: Option<RunOutcome>) -> Outcome {
    match finished {
        Some(RunOutcome::Accepted) => Outcome::Accepted,
        Some(RunOutcome::Rejected) | Some(RunOutcome::MaxStepsExceeded) => Outcome::Rejected,
        None => Outcome::Running,
    }
}
