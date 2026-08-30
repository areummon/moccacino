/* Pushdown automaton run panel: stepping over configurations with a stack
 * lane instead of a tape strip. Mirrors the Turing panel's structure;
 * nondeterministic machines explore the frontier level by level. */
use std::collections::HashSet;

use iced::Task;
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

use moca_data::pushdown_automata::PdaConfiguration;

use super::message::Message;
use super::tab::{PdaNdFrontier, PdaRun, TabMachine};
use crate::gui::theme;

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

        let mut panel = column![
            text("Pushdown automaton")
                .size(14)
                .color(theme::TEXT_DIM),
            row![
                text_input("Input word...", &tab.pda_input_text)
                    .on_input(Message::PdaInputChanged)
                    .on_submit(Message::PdaLoadInput)
                    .width(220)
                    .style(|_theme: &iced::Theme, status| theme::input(status)),
                button(text("Load").size(14).color(theme::CREAM))
                    .on_press(Message::PdaLoadInput)
                    .style(|_theme: &iced::Theme, status| theme::secondary_button(status))
                    .padding([4, 10]),
                button(text(if tab.pda_playing { "Pause" } else { "Play" }).size(14).color(if tab.pda_playing { theme::BG } else { theme::CREAM }))
                    .on_press(Message::PdaTogglePlay)
                    .style(move |_theme: &iced::Theme, status| {
                        if tab.pda_playing { theme::primary_button(status) } else { theme::secondary_button(status) }
                    })
                    .padding([4, 10]),
                button(text("Step").size(14).color(theme::CREAM))
                    .on_press(Message::PdaStep)
                    .style(|_theme: &iced::Theme, status| theme::secondary_button(status))
                    .padding([4, 10]),
                button(text("Reset").size(14).color(theme::CREAM))
                    .on_press(Message::PdaReset)
                    .style(|_theme: &iced::Theme, status| theme::secondary_button(status))
                    .padding([4, 10]),
            ]
            .spacing(6),
        ]
        .spacing(8);

        if !tab.machine.is_deterministic() {
            if let Some(frontier) = &tab.pda_frontier {
                panel = panel.push(self.create_pda_nd_lane_view(frontier));
            } else {
                panel = panel.push(
                    text("Nondeterministic machine: type an input and press Load to explore the branching computation level by level.")
                        .size(13)
                        .color(theme::TEXT_DIM),
                );
            }
        } else if let Some(run) = &tab.pda_run {
            // Input ribbon: the consumed prefix dimmed, the rest bright, the
            // next character highlighted.
            let consumed_chars = run.input.chars().count() - run.config.remaining_input().chars().count();
            panel = panel.push(Self::input_ribbon(&run.input, consumed_chars, 22.0, 15.0));

            // Content row: the stack lane on the left, status on the right.
            let stack_entries = run.config.stack();
            let visible = stack_entries.len().min(STACK_VISIBLE_ENTRIES);
            let hidden = stack_entries.len() - visible;
            let mut stack_lane = column![
                text("stack")
                    .size(12)
                    .color(theme::TEXT_FAINT),
            ]
            .spacing(2);
            // Render top first: reverse over the visible window.
            for (offset, entry) in stack_entries.iter().rev().take(visible).enumerate() {
                let is_top = offset == 0;
                stack_lane = stack_lane.push(
                    container(
                        text(entry.as_str())
                            .font(iced::Font::MONOSPACE)
                            .size(14)
                            .color(if is_top { theme::CREAM } else { theme::TEXT_DIM }),
                    )
                    .width(Length::Fixed(72.0))
                    .center_x(Length::Fixed(72.0))
                    .padding([2, 4])
                    .style(move |_theme: &iced::Theme| {
                        container::Style {
                            background: Some(
                                if is_top { theme::TEAL } else { theme::RAISED }.into()
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
            if hidden > 0 {
                stack_lane = stack_lane.push(
                    text(format!("… {} more below", hidden))
                        .size(11)
                        .color(theme::TEXT_FAINT),
                );
            }

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
            let status = row![
                text(format!("State: {}", state_label)).size(14).color(status_color),
                text(format!("Steps: {}", run.steps)).size(14).color(theme::TEXT_DIM),
                text(match run.finished {
                    Some(true) => "Accepted".to_string(),
                    Some(false) => "Rejected".to_string(),
                    None => "Running...".to_string(),
                }).size(14).color(status_color),
            ]
            .spacing(18);

            panel = panel.push(
                row![
                    stack_lane,
                    column![
                        status,
                        text(format!("Input left: {}", if run.config.remaining_input().is_empty() { "ε".to_string() } else { run.config.remaining_input().clone() }))
                            .size(13)
                            .color(theme::TEXT_DIM),
                    ]
                    .spacing(8),
                ]
                .spacing(24),
            );
        } else {
            panel = panel.push(
                text("Type an input and press Load to step through the computation; the stack grows upward.")
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
     * live branch with its state, a compact stack snapshot (bottom first,
     * top last) and the input still to consume. */
    fn create_pda_nd_lane_view(&self, frontier: &PdaNdFrontier) -> Element<'_, Message> {
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
            let stack_text = if config.stack().is_empty() {
                "[]".to_string()
            } else {
                format!("[{}]", config.stack().join(" "))
            };
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
                    text(stack_text)
                        .font(iced::Font::MONOSPACE)
                        .size(12)
                        .color(theme::CREAM),
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

}
