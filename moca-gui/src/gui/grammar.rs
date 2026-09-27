/* Grammar editing panel: parse, membership check, leftmost derivation and
 * CNF conversion over the grammar held by a tab. */
use iced::Task;
use iced::widget::{column, container, horizontal_space, row, scrollable, text_input, tooltip};
use iced::{Alignment, Element, Length};

use moca_data::grammar::Grammar;

use super::message::Message;
use super::widgets::text;
use super::tab::TabMachine;
use super::widgets;
use crate::gui::theme::{self, Family};

const GRAMMAR_PLACEHOLDER: &str = "S -> a S b | ε";

impl super::app::App {
    /* Multi-line editor actions: perform the edit and mirror the content
     * into the plain string the parser consumes. Ctrl+Enter parses instead
     * of inserting a newline. */
    pub(crate) fn grammar_editor_action(&mut self, action: iced::widget::text_editor::Action) -> Task<Message> {
        if matches!(
            action,
            iced::widget::text_editor::Action::Edit(iced::widget::text_editor::Edit::Enter)
        ) && self.get_active_tab().state_machine.is_ctrl_pressed()
        {
            self.get_active_tab_mut().grammar_text =
                self.get_active_tab().grammar_content.text();
            return self.grammar_parse();
        }
        let tab = self.get_active_tab_mut();
        tab.grammar_content.perform(action);
        tab.grammar_text = tab.grammar_content.text();
        Task::none()
    }

    pub(crate) fn grammar_word_changed(&mut self, text: String) -> Task<Message> {
        self.get_active_tab_mut().grammar_word = text;
        Task::none()
    }

    /* Parses the textarea content into the machine slot of the tab; errors
     * surface through the shared error popup with line information. */
    pub(crate) fn grammar_parse(&mut self) -> Task<Message> {
        let source = self.get_active_tab().grammar_text.trim().to_string();
        if source.is_empty() {
            self.error_message = Some("The grammar is empty.".to_string());
            return Task::none();
        }
        match moca_data::grammar::parse_grammar(&source) {
            Ok(grammar) => {
                let start = grammar.start_symbol().to_string();
                self.get_active_tab_mut().machine = TabMachine::Grammar(grammar);
                self.get_active_tab_mut().grammar_output =
                    Some(format!("Parsed OK. Start symbol: {}. Terminals are derived from the bodies.", start));
            },
            Err(error) => {
                self.error_message = Some(format!("Invalid grammar: {}", error));
            },
        }
        Task::none()
    }

    /* Membership via CYK: the word is segmented into single characters. */
    pub(crate) fn grammar_check_word(&mut self) -> Task<Message> {
        let parsed = self.ensure_parsed_grammar();
        let (grammar, problem) = match parsed {
            Some(pair) => pair,
            None => return Task::none(),
        };
        if let Some(problem) = problem {
            self.error_message = Some(problem);
            return Task::none();
        }
        let input = self.get_active_tab().grammar_word.trim().to_string();
        let answer = grammar.generate(&input);
        self.get_active_tab_mut().grammar_output = Some(format!(
            "{} {:?} {}",
            if answer { "Accepted" } else { "Rejected" },
            input,
            if answer { "\u{2713}" } else { "\u{2717}" }
        ));
        Task::none()
    }

    /* Leftmost derivation display for the typed word. */
    pub(crate) fn grammar_derive(&mut self) -> Task<Message> {
        let parsed = self.ensure_parsed_grammar();
        let (grammar, problem) = match parsed {
            Some(pair) => pair,
            None => return Task::none(),
        };
        if let Some(problem) = problem {
            self.error_message = Some(problem);
            return Task::none();
        }
        let input = self.get_active_tab().grammar_word.trim().to_string();
        let max_derive_steps: usize = 50_000;
        match grammar.derive_leftmost(&input, max_derive_steps) {
            None => {
                self.get_active_tab_mut().grammar_output =
                    Some(format!("No leftmost derivation of {:?} within {} steps.", input, max_derive_steps));
            },
            Some(chain) => {
                let rendered: Vec<String> = chain
                    .iter()
                    .map(|form| form.join(" "))
                    .collect();
                let mut shown = rendered.join("\n=> ");
                if shown.chars().count() > 2000 {
                    shown.truncate(2000);
                    shown.push_str(" ...");
                }
                self.get_active_tab_mut().grammar_output = Some(shown);
            },
        }
        Task::none()
    }

    /* Converts to Chomsky normal form and opens it in a fresh grammar tab,
     * mirroring how DFA->NFA/Minimize produce their own tabs. */
    pub(crate) fn grammar_to_cnf(&mut self) -> Task<Message> {
        let parsed = self.ensure_parsed_grammar();
        let (grammar, problem) = match parsed {
            Some(pair) => pair,
            None => return Task::none(),
        };
        if let Some(problem) = problem {
            self.error_message = Some(problem);
            return Task::none();
        }
        let cnf = grammar.to_chomsky_normal_form();
        let mut new_tab = super::tab::Tab::new_with_name("CNF".to_string());
        new_tab.machine = TabMachine::Grammar(cnf.clone());
        new_tab.grammar_text = format!("{}", cnf);
        new_tab.grammar_content = iced::widget::text_editor::Content::with_text(&new_tab.grammar_text);
        new_tab.grammar_output = Some(format!(
            "Chomsky normal form ready. Start symbol: {}.",
            cnf.start_symbol()
        ));
        self.tabs.push(Box::new(new_tab));
        self.active_tab = self.tabs.len() - 1;
        Task::none()
    }

    /* Runs an FA-only transformation report used by operations.rs guards on
     * grammar tabs. */
    pub(crate) fn reject_operation_for_grammar(&mut self, operation: &str) -> Task<Message> {
        self.error_message = Some(format!(
            "Cannot {}: the tab holds a grammar, not a state machine.",
            operation
        ));
        Task::none()
    }

    /* Parses the current textarea content without touching error state when
     * it succeeds; returns (grammar, parse-problem-if-any). */
    pub(crate) fn ensure_parsed_grammar(&mut self) -> Option<(Grammar, Option<String>)> {
        let source = self.get_active_tab().grammar_text.trim().to_string();
        match moca_data::grammar::parse_grammar(&source) {
            Ok(grammar) => {
                // Refresh the stored machine so canvas-free tabs stay in sync.
                self.get_active_tab_mut().machine = TabMachine::Grammar(grammar.clone());
                Some((grammar, None))
            },
            Err(error) => Some((
                Grammar::default(),
                Some(format!("Invalid grammar: {}", error)),
            )),
        }
    }
}

/* -------- panel view builder -------- */

impl super::app::App {
    /* Grammar tabs have no canvas: the workspace becomes two cards, the
     * productions editor on the left and word testing on the right. */
    pub(crate) fn create_grammar_panel(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();

        let editor_card = container(
            column![
                row![
                    widgets::family_dot(Family::Grammar, 9.0),
                    text("Productions").size(15).font(theme::BOLD),
                    horizontal_space(),
                    widgets::caption("one rule per line · | between alternatives · ε for empty"),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                iced::widget::text_editor(&tab.grammar_content)
                    .height(Length::Fill)
                    .padding(12)
                    .font(theme::MONO)
                    .size(14)
                    .placeholder(GRAMMAR_PLACEHOLDER)
                    .on_action(Message::GrammarEditorAction)
                    .style(theme::editor),
                row![
                    widgets::tip(
                        widgets::primary("Parse", Some(Message::GrammarParse)),
                        "Check the rules  (Ctrl+Enter)",
                        tooltip::Position::Top,
                    ),
                    widgets::tip(
                        widgets::secondary("To CNF", Some(Message::GrammarToCnf)),
                        "Open the Chomsky normal form in a new tab",
                        tooltip::Position::Top,
                    ),
                ]
                .spacing(8),
            ]
            .spacing(12),
        )
        .padding(18)
        .width(Length::FillPortion(3))
        .height(Length::Fill)
        .style(theme::card);

        let output: Element<'_, Message> = match &tab.grammar_output {
            Some(output) => scrollable(
                container(text(output.as_str()).font(theme::MONO).size(13))
                    .padding([10, 12])
                    .width(Length::Fill),
            )
            .height(Length::Fill)
            .style(theme::scroll)
            .into(),
            None => container(widgets::hint(
                "Parse the grammar, then check words (CYK) or show their leftmost derivations.",
            ))
            .padding([10, 12])
            .into(),
        };

        let test_card = container(
            column![
                text("Test a word").size(15).font(theme::BOLD),
                text_input("e.g. aabb", &tab.grammar_word)
                    .on_input(Message::GrammarWordChanged)
                    .on_submit(Message::GrammarCheckWord)
                    .padding([8, 12])
                    .font(theme::MONO)
                    .style(theme::input),
                row![
                    widgets::primary("Check", Some(Message::GrammarCheckWord)),
                    widgets::secondary("Derive", Some(Message::GrammarDerive)),
                ]
                .spacing(8),
                widgets::caption("Result"),
                container(output)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(theme::sunken),
            ]
            .spacing(12),
        )
        .padding(18)
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .style(theme::card);

        container(container(row![editor_card, test_card].spacing(16)).max_width(1280))
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .into()
    }
}
