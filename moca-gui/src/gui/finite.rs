/* Finite automaton run panel: single-branch stepping for deterministic
 * machines, level-by-level branches for nondeterministic ones. Mirrors the
 * Turing panel's structure. */
use std::collections::HashSet;

use iced::Task;
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

use moca_data::finite_automata::FiniteConfiguration;

use super::message::Message;
use super::tab::{FiniteNdFrontier, FiniteRun, TabMachine};
use crate::gui::theme;

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

        let mut panel = column![
            text("Finite automaton")
                .size(14)
                .color(theme::TEXT_DIM),
            row![
                text_input("Input word...", &tab.finite_input_text)
                    .on_input(Message::FiniteInputChanged)
                    .on_submit(Message::FiniteLoadInput)
                    .width(220)
                    .style(|_theme: &iced::Theme, status| theme::input(status)),
                button(text("Load").size(14).color(theme::CREAM))
                    .on_press(Message::FiniteLoadInput)
                    .style(|_theme: &iced::Theme, status| theme::secondary_button(status))
                    .padding([4, 10]),
                button(text(if tab.finite_playing { "Pause" } else { "Play" }).size(14).color(if tab.finite_playing { theme::BG } else { theme::CREAM }))
                    .on_press(Message::FiniteTogglePlay)
                    .style(move |_theme: &iced::Theme, status| {
                        if tab.finite_playing { theme::primary_button(status) } else { theme::secondary_button(status) }
                    })
                    .padding([4, 10]),
                button(text("Step").size(14).color(theme::CREAM))
                    .on_press(Message::FiniteStep)
                    .style(|_theme: &iced::Theme, status| theme::secondary_button(status))
                    .padding([4, 10]),
                button(text("Reset").size(14).color(theme::CREAM))
                    .on_press(Message::FiniteReset)
                    .style(|_theme: &iced::Theme, status| theme::secondary_button(status))
                    .padding([4, 10]),
            ]
            .spacing(6),
        ]
        .spacing(8);

        if !tab.machine.is_deterministic() {
            if let Some(frontier) = &tab.finite_frontier {
                panel = panel.push(self.create_finite_nd_lane_view(frontier));
            } else {
                panel = panel.push(
                    text("Nondeterministic machine: type an input and press Load to explore the branches transition by transition.")
                        .size(13)
                        .color(theme::TEXT_DIM),
                );
            }
        } else if let Some(run) = &tab.finite_run {
            // Input ribbon: the consumed prefix dimmed, the rest bright, the
            // next character highlighted.
            let consumed_chars =
                run.input.chars().count() - run.config.remaining_input().chars().count();
            panel = panel.push(Self::input_ribbon(&run.input, consumed_chars, 22.0, 15.0));

            // Status line: current state (by label when available), steps
            // and outcome.
            let state_id = run.config.state_id() as usize;
            let state_label = tab.states.iter()
                .find(|node| node.id == state_id)
                .map(|node| node.label.to_string())
                .unwrap_or_else(|| format!("q{}", state_id));
            let status_color = match run.finished {
                Some(true) => theme::ACCEPT,
                Some(false) => theme::REJECT,
                None => theme::TEXT_DIM,
            };
            panel = panel.push(
                row![
                    text(format!("State: {}", state_label)).size(14).color(status_color),
                    text(format!("Steps: {}", run.steps)).size(14).color(theme::TEXT_DIM),
                    text(match run.finished {
                        Some(true) => "Accepted".to_string(),
                        Some(false) => "Rejected".to_string(),
                        None => "Running...".to_string(),
                    }).size(14).color(status_color),
                ]
                .spacing(18)
            );
        } else {
            panel = panel.push(
                text("Type an input and press Load to step through the consumption of the word.")
                    .size(13)
                    .color(theme::TEXT_FAINT),
            );
        }

        container(panel)
            .style(|_theme: &iced::Theme| theme::panel_box())
            .padding([8, 12])
            .into()
    }

    /* Level-by-level lane view for nondeterministic machines: one row per
     * live branch with its state and the input still to consume. */
    fn create_finite_nd_lane_view(&self, frontier: &FiniteNdFrontier) -> Element<'_, Message> {
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
                    if config.remaining_input().is_empty() {
                        Self::epsilon_lane_label()
                    } else {
                        Self::input_ribbon(config.remaining_input(), 0, 16.0, 12.0).into()
                    },
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
            Some(true) => theme::ACCEPT,
            Some(false) => theme::REJECT,
            None => theme::TEXT_DIM,
        };
        view = view.push(
            text(match frontier.finished {
                Some(true) => "Accepted".to_string(),
                Some(false) => "Rejected".to_string(),
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

    pub(crate) fn epsilon_lane_label() -> iced::Element<'static, Message> {
        text("ε")
            .font(iced::Font::MONOSPACE)
            .size(12)
            .color(theme::TEXT_FAINT)
            .into()
    }

    /* Input ribbon shared by the finite and pushdown run panels: the
     * consumed prefix dimmed, the next character highlighted, the rest
     * bright. */
    pub(crate) fn input_ribbon(
        input: &str,
        consumed_chars: usize,
        cell_width: f32,
        font_size: f32,
    ) -> iced::widget::Row<'static, Message> {
        let mut ribbon = row![].spacing(1);
        for (index, c) in input.chars().enumerate() {
            let consumed = index < consumed_chars;
            let is_next = index == consumed_chars;
            ribbon = ribbon.push(
                container(
                    text(c.to_string())
                        .font(iced::Font::MONOSPACE)
                        .size(font_size)
                        .color(if is_next {
                            theme::CREAM
                        } else if consumed {
                            theme::TEXT_FAINT
                        } else {
                            theme::TEXT_DIM
                        }),
                )
                .width(Length::Fixed(cell_width))
                .center_x(Length::Fixed(cell_width))
                .padding([2, 4])
                .style(move |_theme: &iced::Theme| {
                    container::Style {
                        background: Some(
                            if is_next { theme::TEAL } else { theme::RAISED }.into()
                        ),
                        border: iced::Border {
                            color: theme::BORDER,
                            width: 1.0,
                            radius: 2.0.into(),
                        },
                        ..Default::default()
                    }
                }),
            );
        }
        ribbon
    }
}
