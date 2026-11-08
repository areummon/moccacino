use iced::Task;
use iced::widget::text_editor::{Action, Binding, KeyPress};
use iced::widget::{column, container, horizontal_space, row, scrollable, tooltip};
use iced::{Alignment, Element, Length};

use moca_data::grammar::Grammar;

use super::message::Message;
use super::shortcuts;
use super::tab::{GrammarEditor, Tab};
use super::widgets::{self, text};
use crate::theme;
use crate::family::Family;

const GRAMMAR_PLACEHOLDER: &str = "S -> a S b | ε";
const MAX_DERIVE_STEPS: usize = 50_000;
const MAX_DERIVATION_CHARS: usize = 2000;

impl super::app::App {
    pub(crate) fn grammar_editor_action(&mut self, action: Action) -> Task<Message> {
        if let Some(editor) = self.get_active_tab_mut().grammar_mut() {
            let edits = action.is_edit();
            editor.content.perform(action);
            if edits {
                editor.text = editor.content.text();
            }
        }
        Task::none()
    }

    fn parsed_grammar(&mut self) -> Option<Grammar> {
        match self.get_active_tab().grammar()?.parse() {
            Ok(grammar) => Some(grammar),
            Err(problem) => {
                self.error_message = Some(problem);
                None
            }
        }
    }

    fn set_grammar_output(&mut self, output: String) {
        if let Some(editor) = self.get_active_tab_mut().grammar_mut() {
            editor.output = Some(output);
        }
    }

    fn grammar_word(&self) -> String {
        self.get_active_tab().grammar().map(|editor| editor.word.trim().to_string()).unwrap_or_default()
    }

    pub(crate) fn grammar_parse(&mut self) -> Task<Message> {
        if let Some(grammar) = self.parsed_grammar() {
            let start = grammar.start_symbol();
            self.set_grammar_output(format!("Parsed OK. Start symbol: {start}. Terminals are derived from the bodies."));
        }
        Task::none()
    }

    pub(crate) fn grammar_check_word(&mut self) -> Task<Message> {
        let Some(grammar) = self.parsed_grammar() else {
            return Task::none();
        };
        let input = self.grammar_word();
        let answer = grammar.generate(&input);
        self.set_grammar_output(format!(
            "{} {:?} {}",
            if answer { "Accepted" } else { "Rejected" },
            input,
            if answer { "\u{2713}" } else { "\u{2717}" }
        ));
        Task::none()
    }

    pub(crate) fn grammar_derive(&mut self) -> Task<Message> {
        let Some(grammar) = self.parsed_grammar() else {
            return Task::none();
        };
        let input = self.grammar_word();
        let output = match grammar.derive_leftmost(&input, MAX_DERIVE_STEPS) {
            None => format!("No leftmost derivation of {input:?} within {MAX_DERIVE_STEPS} steps."),
            Some(chain) => {
                let rendered: Vec<String> = chain.iter().map(|form| form.join(" ")).collect();
                let mut shown = rendered.join("\n=> ");
                if let Some((cut, _)) = shown.char_indices().nth(MAX_DERIVATION_CHARS) {
                    shown.truncate(cut);
                    shown.push_str(" ...");
                }
                shown
            }
        };
        self.set_grammar_output(output);
        Task::none()
    }

    pub(crate) fn grammar_to_cnf(&mut self) -> Task<Message> {
        let Some(grammar) = self.parsed_grammar() else {
            return Task::none();
        };
        let cnf = grammar.to_chomsky_normal_form();
        let output = format!("Chomsky normal form ready. Start symbol: {}.", cnf.start_symbol());
        self.open_in_new_tab(Tab::from_grammar("CNF".to_string(), &cnf, output))
    }
}

impl super::app::App {
    pub(crate) fn create_grammar_panel<'a>(&self, editor: &'a GrammarEditor) -> Element<'a, Message> {

        let editor_card = container(
            column![
                row![
                    widgets::family_dot(Family::Grammar, 9.0),
                    text("Productions").size(theme::TEXT_BASE).font(theme::BOLD),
                    horizontal_space(),
                    widgets::caption("one rule per line · | between alternatives · ε for empty"),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                iced::widget::text_editor(&editor.content)
                    .height(Length::Fill)
                    .padding(12)
                    .font(theme::MONO)
                    .size(theme::TEXT_BASE)
                    .placeholder(GRAMMAR_PLACEHOLDER)
                    .on_action(Message::GrammarEditorAction)
                    .key_binding(grammar_key_binding)
                    .style(theme::editor),
                row![
                    widgets::tip(
                        widgets::primary(None, "Parse", Some(Message::GrammarParse)),
                        format!("Check the rules  ({})", shortcuts::hint(shortcuts::Action::ParseGrammar)),
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

        let output: Element<'_, Message> = match &editor.output {
            Some(output) => scrollable(
                container(text(output.as_str()).font(theme::MONO).size(theme::TEXT_MD))
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
                text("Test a word").size(theme::TEXT_BASE).font(theme::BOLD),
                widgets::input("e.g. aabb", &editor.word)
                    .on_input(Message::GrammarWordChanged)
                    .on_submit(Message::GrammarCheckWord)
                    .font(theme::MONO),
                row![
                    widgets::primary(None, "Check", Some(Message::GrammarCheckWord)),
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

fn grammar_key_binding(key_press: KeyPress) -> Option<Binding<Message>> {
    if shortcuts::matches(shortcuts::Action::ParseGrammar, &key_press.key, key_press.modifiers) {
        return Some(Binding::Custom(Message::GrammarParse));
    }
    Binding::from_key_press(key_press)
}
