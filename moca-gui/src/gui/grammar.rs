/* Grammar editing panel: parse, membership check, leftmost derivation and
 * CNF conversion over the grammar held by a tab. */
use iced::Task;
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

use moca_data::grammar::Grammar;

use super::message::Message;
use super::tab::TabMachine;
use crate::gui::theme;

const GRAMMAR_PLACEHOLDER: &str = "S -> a S b | ε";

impl super::app::App {
    pub(crate) fn active_tab_is_grammar(&self) -> bool {
        self.get_active_tab().machine.is_grammar()
    }

    pub(crate) fn add_grammar_tab(&mut self) -> Task<Message> {
        self.tabs.push(Box::new(super::tab::Tab::new_grammar()));
        self.active_tab = self.tabs.len() - 1;
        Task::none()
    }

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
    pub(crate) fn create_grammar_panel(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();

        let mut column_root = column![
            text("Context-free grammar")
                .size(14)
                .color(theme::TEXT_DIM),
            row![
                button(text("Parse").size(14).color(theme::CREAM))
                    .on_press(Message::GrammarParse)
                    .style(|_theme: &iced::Theme, status| theme::secondary_button(status))
                    .padding([4, 10]),
                button(text("To CNF").size(14).color(theme::CREAM))
                    .on_press(Message::GrammarToCnf)
                    .style(|_theme: &iced::Theme, status| theme::secondary_button(status))
                    .padding([4, 10]),
                text("(one rule per line, alternatives after '|', \u{3b5} = empty body, Ctrl+Enter parses)")
                    .size(12)
                    .color(theme::TEXT_FAINT),
            ]
            .spacing(6),
        ]
        .spacing(8);

        column_root = column_root.push(
            iced::widget::text_editor(&tab.grammar_content)
                .height(Length::Fixed(110.0))
                .placeholder(GRAMMAR_PLACEHOLDER)
                .on_action(Message::GrammarEditorAction)
                .style(|_theme: &iced::Theme, status| theme::editor(status)),
        );

        column_root = column_root.push(
            row![
                text("Word:").size(14).color(theme::TEXT_DIM),
                text_input("e.g. abab", &tab.grammar_word)
                    .on_input(Message::GrammarWordChanged)
                    .on_submit(Message::GrammarCheckWord)
                    .width(160)
                    .style(|_theme: &iced::Theme, status| theme::input(status)),
                button(text("Check").size(14).color(theme::CREAM))
                    .on_press(Message::GrammarCheckWord)
                    .style(|_theme: &iced::Theme, status| theme::secondary_button(status))
                    .padding([4, 10]),
                button(text("Derive").size(14).color(theme::CREAM))
                    .on_press(Message::GrammarDerive)
                    .style(|_theme: &iced::Theme, status| theme::secondary_button(status))
                    .padding([4, 10]),
            ]
            .spacing(6),
        );

        if let Some(output) = &tab.grammar_output {
            column_root = column_root.push(
                container(
                    text(output.as_str())
                        .font(iced::Font::MONOSPACE)
                        .size(13)
                        .color(theme::CREAM),
                )
                .max_width(1100)
                .padding([6, 10])
                .style(|_theme: &iced::Theme| theme::inset_box()),
            );
        } else {
            column_root = column_root.push(
                text("Parse the grammar, then check words or show their leftmost derivations.")
                    .size(13)
                    .color(theme::TEXT_FAINT),
            );
        }

        container(column_root)
            .style(|_theme: &iced::Theme| theme::panel_box())
            .padding([8, 12])
            .into()
    }
}
