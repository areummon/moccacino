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
                    Some(format!("Parsed OK. Start symbol: {start}. Terminals are derived from the bodies."));
            },
            Err(error) => {
                self.error_message = Some(format!("Invalid grammar: {error}"));
            },
        }
        Task::none()
    }

    pub(crate) fn grammar_check_word(&mut self) -> Task<Message> {
        let grammar = match self.parsed_grammar() {
            Ok(grammar) => grammar,
            Err(problem) => {
                self.error_message = Some(problem);
                return Task::none();
            }
        };
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

    pub(crate) fn grammar_derive(&mut self) -> Task<Message> {
        let grammar = match self.parsed_grammar() {
            Ok(grammar) => grammar,
            Err(problem) => {
                self.error_message = Some(problem);
                return Task::none();
            }
        };
        let input = self.get_active_tab().grammar_word.trim().to_string();
        let max_derive_steps: usize = 50_000;
        match grammar.derive_leftmost(&input, max_derive_steps) {
            None => {
                self.get_active_tab_mut().grammar_output =
                    Some(format!("No leftmost derivation of {input:?} within {max_derive_steps} steps."));
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

    pub(crate) fn grammar_to_cnf(&mut self) -> Task<Message> {
        let grammar = match self.parsed_grammar() {
            Ok(grammar) => grammar,
            Err(problem) => {
                self.error_message = Some(problem);
                return Task::none();
            }
        };
        let cnf = grammar.to_chomsky_normal_form();
        let output = format!("Chomsky normal form ready. Start symbol: {}.", cnf.start_symbol());
        self.open_grammar_in_new_tab("CNF".to_string(), cnf, output);
        Task::none()
    }

    pub(crate) fn reject_operation_for_grammar(&mut self, operation: &str) -> Task<Message> {
        self.error_message = Some(format!(
            "Cannot {operation}: the tab holds a grammar, not a state machine."
        ));
        Task::none()
    }

    pub(crate) fn parsed_grammar(&mut self) -> Result<Grammar, String> {
        let source = self.get_active_tab().grammar_text.trim().to_string();
        let grammar = moca_data::grammar::parse_grammar(&source)
            .map_err(|error| format!("Invalid grammar: {error}"))?;
        self.get_active_tab_mut().machine = TabMachine::Grammar(grammar.clone());
        Ok(grammar)
    }

    pub(crate) fn open_grammar_in_new_tab(&mut self, name: String, grammar: Grammar, output: String) {
        let mut tab = super::tab::Tab::new_grammar();
        tab.name = name;
        tab.grammar_text = grammar.to_string();
        tab.grammar_content = iced::widget::text_editor::Content::with_text(&tab.grammar_text);
        tab.grammar_output = Some(output);
        tab.machine = TabMachine::Grammar(grammar);
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
    }
}

impl super::app::App {
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
