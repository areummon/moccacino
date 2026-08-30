/* Turing machine run panel: interactive stepping over configurations. */
use std::collections::HashSet;

use iced::Task;
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

use moca_data::state_machine::StateMachine;
use moca_data::turing_machine::{Configuration, RunOutcome};

use super::message::Message;
use super::tab::{NdFrontier, TabMachine, TmRun};
use crate::gui::theme;

const TAPE_CONTEXT: i64 = 12;
/* Lane window rendered per branch in the nondeterministic view. */
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
        // Malformed labels or a missing initial state abort the run early
        // with a precise message instead of halting silently mid-run.
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

    /* One auto-play tick: deterministic runs advance one step,
     * nondeterministic frontiers advance one level. */
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

    /* One level of the nondeterministic frontier: every live branch takes a
     * parallel step, successors dedup against the visited set, and the
     * verdict mirrors `run_nondeterministic` exactly (arrival acceptance
     * before expanding, empty next frontier = rejected). */
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
        // Split borrows so the machine and its run state can be touched at
        // the same time.
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
        // Arrival semantics, mirroring `run`: an accepting state accepts
        // before any transition is applied.
        if turing.get_final_states().contains(&run.config.state_id()) {
            run.finished = Some(RunOutcome::Accepted);
            return;
        }
        match turing.step(&run.config) {
            Some(next_config) => {
                run.config = next_config;
                run.steps += 1;
            },
            // No applicable transition on some tape.
            None => run.finished = Some(RunOutcome::Rejected),
        }
    }

    pub(crate) fn create_tm_panel(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();

        let mut panel = column![
            text("Turing machine")
                .size(14)
                .color(theme::TEXT_DIM),
            row![
                text_input("Input word...", &tab.tm_input_text)
                    .on_input(Message::TmInputChanged)
                    .on_submit(Message::TmLoadInput)
                    .width(220)
                    .style(|_theme: &iced::Theme, status| theme::input(status)),
                button(text("Load").size(14).color(theme::CREAM))
                    .on_press(Message::TmLoadInput)
                    .style(|_theme: &iced::Theme, status| theme::secondary_button(status))
                    .padding([4, 10]),
                button(text(if tab.tm_playing { "Pause" } else { "Play" }).size(14).color(if tab.tm_playing { theme::BG } else { theme::CREAM }))
                    .on_press(Message::TmTogglePlay)
                    .style(move |_theme: &iced::Theme, status| {
                        if tab.tm_playing { theme::primary_button(status) } else { theme::secondary_button(status) }
                    })
                    .padding([4, 10]),
                button(text("Step").size(14).color(theme::CREAM))
                    .on_press(Message::TmStep)
                    .style(|_theme: &iced::Theme, status| theme::secondary_button(status))
                    .padding([4, 10]),
                button(text("Reset").size(14).color(theme::CREAM))
                    .on_press(Message::TmReset)
                    .style(|_theme: &iced::Theme, status| theme::secondary_button(status))
                    .padding([4, 10]),
            ]
            .spacing(6),
        ]
        .spacing(8);

        if !tab.machine.is_deterministic() {
            if let Some(frontier) = &tab.tm_frontier {
                panel = panel.push(self.create_nd_lane_view(frontier));
            } else {
                panel = panel.push(
                    text("Nondeterministic machine: type an input and press Load to explore the branching computation level by level.")
                        .size(13)
                        .color(theme::TEXT_DIM),
                );
            }
        } else if let Some(run) = &tab.tm_run {
            // Tape strip with the head highlighted.
            let (window, head_offset) = run.config.tape().snapshot(TAPE_CONTEXT);
            panel = panel.push(Self::tape_cells(&window, head_offset, 20.0, 16.0));

            // Status line: current state (by label when available), steps and outcome.
            let state_id = run.config.state_id() as usize;
            let state_label = tab.states.iter()
                .find(|node| node.id == state_id)
                .map(|node| node.label.to_string())
                .unwrap_or_else(|| format!("q{}", state_id));
            let status_color = match run.finished {
                Some(RunOutcome::Accepted) => theme::ACCEPT,
                Some(RunOutcome::Rejected) => theme::REJECT,
                _ => theme::TEXT_DIM,
            };
            panel = panel.push(
                row![
                    text(format!("State: {}", state_label)).size(14).color(status_color),
                    text(format!("Steps: {}", run.steps)).size(14).color(theme::TEXT_DIM),
                    text(match run.finished {
                        Some(RunOutcome::Accepted) => "Accepted".to_string(),
                        Some(RunOutcome::Rejected) => "Rejected".to_string(),
                        _ => "Running...".to_string(),
                    }).size(14).color(status_color),
                ]
                .spacing(18)
            );
        } else {
            panel = panel.push(
                text("Type an input and press Load to start stepping through the computation.")
                    .size(13)
                    .color(theme::TEXT_FAINT),
            );
        }

        container(panel)
            .style(|_theme: &iced::Theme| theme::panel_box())
            .padding([8, 12])
            .into()
    }

    /* One tape window as a row of fixed-width monospace cells. Shared by the
     * deterministic tape strip and the nondeterministic lane view. */
    fn tape_cells(window: &str, head_offset: usize, cell_width: f32, font_size: f32) -> iced::widget::Row<'static, Message> {
        let chars: Vec<(usize, char)> = window.chars().enumerate().collect();
        let cells = chars.into_iter().map(|(index, c)| {
            let is_head = index == head_offset;
            container(
                text(c.to_string())
                    .font(iced::Font::MONOSPACE)
                    .size(font_size)
                    .color(if is_head { theme::CREAM } else { theme::TEXT_DIM })
            )
            .width(Length::Fixed(cell_width))
            .center_x(Length::Fixed(cell_width))
            .style(move |_theme: &iced::Theme| {
                container::Style {
                    background: Some(
                        if is_head { theme::TEAL } else { theme::RAISED }.into()
                    ),
                    border: iced::Border {
                        color: theme::BORDER,
                        width: 1.0,
                        radius: 2.0.into(),
                    },
                    ..Default::default()
                }
            })
            .into()
        });
        row(cells).spacing(1)
    }

    /* Level-by-level lane view for nondeterministic machines. */
    fn create_nd_lane_view(&self, frontier: &NdFrontier) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let mut view = column![
            text(format!(
                "Level {} · {} branches alive",
                frontier.level,
                frontier.alive.len()
            ))
            .size(14)
            .color(theme::CREAM),
        ]
        .spacing(6);

        let state_label_of = |state_id: u64| -> String {
            tab.states
                .iter()
                .find(|node| node.id == state_id as usize)
                .map(|node| node.label.to_string())
                .unwrap_or_else(|| format!("q{}", state_id))
        };

        for (index, config) in frontier.alive.iter().take(ND_LANES_VISIBLE).enumerate() {
            let (window, head_offset) = config.tape().snapshot(TAPE_CONTEXT);
            view = view.push(
                row![
                    text(format!("─ {:>2}", index + 1))
                        .font(iced::Font::MONOSPACE)
                        .size(12)
                        .color(theme::TEXT_FAINT),
                    text(state_label_of(config.state_id()))
                        .font(iced::Font::MONOSPACE)
                        .size(12)
                        .color(theme::GREEN),
                    Self::tape_cells(&window, head_offset, 14.0, 12.0),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            );
        }
        if frontier.alive.len() > ND_LANES_VISIBLE {
            view = view.push(
                text(format!("… {} more branches", frontier.alive.len() - ND_LANES_VISIBLE))
                    .size(12)
                    .color(theme::TEXT_FAINT),
            );
        }

        let status_color = match frontier.finished {
            Some(RunOutcome::Accepted) => theme::ACCEPT,
            Some(RunOutcome::Rejected) | Some(RunOutcome::MaxStepsExceeded) => theme::REJECT,
            None => theme::TEXT_DIM,
        };
        view = view.push(
            text(match frontier.finished {
                Some(RunOutcome::Accepted) => "Accepted".to_string(),
                Some(RunOutcome::Rejected) | Some(RunOutcome::MaxStepsExceeded) => "Rejected".to_string(),
                None => "Running...".to_string(),
            })
            .size(14)
            .color(status_color),
        );

        container(view)
            .padding([6, 10])
            .style(|_theme: &iced::Theme| theme::inset_box())
            .into()
    }
}
