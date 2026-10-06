use iced::widget::{button, column, container, horizontal_space, row, scrollable, text_input, Space};
use iced::{Alignment, Element, Length};

use super::icons::{icon, Icon, Ink};
use super::message::Message;
use super::widgets::text;
use super::app::CloseRequest;
use super::tab::TabMachine;
use super::widgets::{self, dialog_card, modal_layer, DIALOG_LG, DIALOG_MD, DIALOG_SM};
use crate::gui::theme::{self, Family};

pub(crate) const EDIT_TEXT_INPUT: &str = "edit-text";
pub(crate) const EDIT_LABEL_INPUT: &str = "edit-label";
pub(crate) const CHECK_INPUT: &str = "check-input";
pub(crate) const REGEX_INPUT: &str = "regex-input";
pub(crate) const LOAD_INPUT: &str = "load-path";
pub(crate) const SAVE_INPUT: &str = "save-path";

fn field<'a>(
    id: impl Into<String>,
    placeholder: &str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
    on_submit: Message,
) -> Element<'a, Message> {
    text_input(placeholder, value)
        .id(id.into())
        .on_input(on_input)
        .on_submit(on_submit)
        .padding([9, 12])
        .size(15)
        .style(theme::input)
        .into()
}

fn error_line<'a>(error: Option<&String>) -> Element<'a, Message> {
    match error {
        Some(error) => row![icon(Icon::Warn, 14.0, Ink::Danger), text(error.clone()).size(13).style(theme::text_danger)]
            .spacing(6)
            .align_y(Alignment::Center)
            .into(),
        None => Space::with_height(18).into(),
    }
}

fn label_hint(machine: &TabMachine) -> &'static str {
    match machine {
        TabMachine::Finite(_) => "A symbol such as a — leave blank for ε",
        TabMachine::Pushdown(_) => "input;pop/push — e.g. a;Z/AZ  (ε = no-op)",
        TabMachine::Turing(_) => "read;write/dir — e.g. 0;1/R  (dir: L, R or S)",
        TabMachine::Grammar(_) => "",
    }
}

impl super::app::App {
    pub(crate) fn create_load_dialog(&self) -> Element<'_, Message> {
        let body = column![
            row![
                field(LOAD_INPUT, "/path/to/file.ce", &self.load_path_text, Message::LoadPathChanged, Message::LoadPathSubmitted),
                button(widgets::labeled(Some((Icon::Folder, Ink::Text)), "Browse…"))
                    .on_press(Message::LoadBrowseClicked)
                    .padding([8, 12])
                    .style(theme::button_secondary),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            error_line(self.load_dialog_error.as_ref()),
        ]
        .spacing(8);
        modal_layer(dialog_card(
            "Open computational entities",
            Some("Every entity in the .ce file opens as its own tab.".to_string()),
            body.into(),
            vec![
                widgets::ghost("Cancel", Some(Message::CancelLoadDialog)).into(),
                widgets::primary("Open", Some(Message::LoadPathSubmitted)).into(),
            ],
            DIALOG_MD,
        ))
    }

    pub(crate) fn create_save_dialog(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let body = column![
            row![
                field(SAVE_INPUT, "/path/to/file.ce", &self.save_path_text, Message::SavePathChanged, Message::SavePathSubmitted),
                button(widgets::labeled(Some((Icon::Folder, Ink::Text)), "Browse…"))
                    .on_press(Message::SaveBrowseClicked)
                    .padding([8, 12])
                    .style(theme::button_secondary),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            error_line(self.save_dialog_error.as_ref()),
        ]
        .spacing(8);
        modal_layer(dialog_card(
            "Save computational entity",
            Some(format!("Write “{}” to a .ce file.", tab.name)),
            body.into(),
            vec![
                widgets::ghost("Cancel", Some(Message::CancelSaveDialog)).into(),
                widgets::primary("Save", Some(Message::SavePathSubmitted)).into(),
            ],
            DIALOG_MD,
        ))
    }

    pub(crate) fn create_startup_picker(&self) -> Element<'_, Message> {
        let p = self.palette();
        let card = |index: usize, family: Family| -> Element<'_, Message> {
            let selected = index == self.startup_selected;
            button(
                column![
                    container(icon(Icon::Family(family), 26.0, Ink::Text))
                        .padding(10)
                        .style(theme::family_badge(family)),
                    text(family.title()).size(16).font(theme::BOLD),
                    text(family.blurb()).size(13).style(theme::text_dim),
                ]
                .spacing(8),
            )
            .on_press(Message::ChooseStartupModule(index))
            .width(Length::Fixed(250.0))
            .padding(18)
            .style(theme::option_card(selected, family.color(p)))
            .into()
        };
        let families = Family::ALL;
        let grid = column![
            row![card(0, families[0]), card(1, families[1])].spacing(14),
            row![card(2, families[2]), card(3, families[3])].spacing(14),
        ]
        .spacing(14);

        let dialog = container(
            column![
                row![
                    icon(Icon::Logo, 34.0, Ink::Text),
                    column![
                        text("Welcome to Moccacino").size(22).font(theme::BOLD),
                        text("What would you like to explore today?").size(14).style(theme::text_dim),
                    ]
                    .spacing(2),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                grid,
                row![
                    button(widgets::labeled(Some((Icon::Folder, Ink::Text)), "Open a .ce file…"))
                        .on_press(Message::StartupOpenFile)
                        .padding([7, 12])
                        .style(theme::button_ghost),
                    horizontal_space(),
                    widgets::kbd("← → ↑ ↓"),
                    text("choose").size(12).style(theme::text_faint),
                    widgets::kbd("Enter"),
                    text("open").size(12).style(theme::text_faint),
                    widgets::ghost("Quit", Some(Message::ExitApp)),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            ]
            .spacing(22),
        )
        .padding(28)
        .width(Length::Fixed(2.0 * 250.0 + 14.0 + 2.0 * 28.0))
        .style(theme::dialog);

        widgets::blocking_layer(dialog.into())
    }

    pub(crate) fn create_check_input_dialog(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let subtitle = if tab.machine.is_grammar() {
            "Membership is decided with CYK over the parsed grammar."
        } else {
            "Runs the whole machine at once; use the run panel to watch it step by step."
        };
        modal_layer(dialog_card(
            "Check an input",
            Some(subtitle.to_string()),
            field(
                CHECK_INPUT,
                "Input word (blank = ε)",
                &tab.check_input_text,
                Message::CheckInputTextChanged,
                Message::SubmitCheckInput,
            ),
            vec![
                widgets::ghost("Cancel", Some(Message::CancelCheckInput)).into(),
                widgets::primary("Check", Some(Message::SubmitCheckInput)).into(),
            ],
            DIALOG_SM,
        ))
    }

    pub(crate) fn create_error_popup(&self) -> Element<'_, Message> {
        let Some(error_message) = &self.error_message else {
            return Space::new(0, 0).into();
        };
        let body = row![
            container(icon(Icon::Warn, 20.0, Ink::Danger))
                .padding(8)
                .style(theme::chip(theme::Tone::Danger)),
            text(error_message.clone()).size(14),
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        modal_layer(dialog_card(
            "That didn't work",
            None,
            body.into(),
            vec![widgets::primary("OK", Some(Message::CloseError)).into()],
            DIALOG_MD,
        ))
    }

    pub(crate) fn create_edit_dialog(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        if tab.pending_transition_dialog_open {
            return modal_layer(dialog_card(
                "New transition",
                Some(label_hint(&tab.machine).to_string()),
                field(
                    EDIT_TEXT_INPUT,
                    "Label",
                    &tab.pending_transition_label,
                    Message::EditTextChanged,
                    Message::FinishEditing,
                ),
                vec![
                    widgets::ghost("Cancel", Some(Message::CancelEditing)).into(),
                    widgets::primary("Add", Some(Message::FinishEditing)).into(),
                ],
                DIALOG_SM,
            ));
        }
        if tab.editing_transition_dialog_open {
            let mut label_inputs = column![].spacing(8);
            for (i, label) in tab.editing_transition_label_inputs.iter().enumerate() {
                label_inputs = label_inputs.push(
                    row![
                        text_input("Label", label)
                            .id(format!("{}-{}", EDIT_LABEL_INPUT, i))
                            .on_input(move |text| Message::EditTransitionLabelChanged(i, text))
                            .on_submit(Message::SaveEditTransitionLabels)
                            .padding([8, 12])
                            .font(theme::MONO)
                            .style(theme::input),
                        widgets::tip(
                            button(icon(Icon::Delete, 16.0, Ink::Danger))
                                .on_press(Message::DeleteEditTransitionLabel(i))
                                .padding(8)
                                .style(theme::button_danger_ghost),
                            "Remove this label",
                            iced::widget::tooltip::Position::Right,
                        ),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                );
            }
            let body = column![
                scrollable(label_inputs).height(Length::Shrink).style(theme::scroll),
                button(widgets::labeled(Some((Icon::Plus, Ink::Accent)), "Add label"))
                    .on_press(Message::AddEditTransitionLabel)
                    .padding([6, 10])
                    .style(theme::button_ghost),
            ]
            .spacing(8);
            return modal_layer(dialog_card(
                "Transition labels",
                Some(label_hint(&tab.machine).to_string()),
                container(body).max_height(320.0).into(),
                vec![
                    widgets::ghost("Cancel", Some(Message::CancelEditTransitionLabels)).into(),
                    widgets::primary("Save", Some(Message::SaveEditTransitionLabels)).into(),
                ],
                DIALOG_SM,
            ));
        }
        let title = if tab.editing_state.is_some() { "Rename state" } else { "Edit transition" };
        modal_layer(dialog_card(
            title,
            None,
            field(EDIT_TEXT_INPUT, "Name", &tab.edit_text, Message::EditTextChanged, Message::FinishEditing),
            vec![
                widgets::ghost("Cancel", Some(Message::CancelEditing)).into(),
                widgets::primary("Save", Some(Message::FinishEditing)).into(),
            ],
            DIALOG_SM,
        ))
    }

    pub(crate) fn create_latex_export_dialog(&self) -> Element<'_, Message> {
        let code = self.latex_export_code.as_deref().unwrap_or("");
        let is_grammar = self.get_active_tab().machine.is_grammar();
        let subtitle = if is_grammar {
            "The grammar's productions, ready for a LaTeX document."
        } else {
            "A TikZ drawing of this automaton, ready for a LaTeX document."
        };
        let body = container(
            scrollable(container(text(code.to_string()).font(theme::MONO).size(13)).padding(14))
                .height(Length::Fixed(320.0))
                .style(theme::scroll),
        )
        .style(theme::sunken);
        modal_layer(dialog_card(
            "LaTeX export",
            Some(subtitle.to_string()),
            body.into(),
            vec![
                widgets::ghost("Close", Some(Message::CloseLatexExport)).into(),
                button(widgets::labeled(Some((Icon::Check, Ink::OnAccent)), "Copy"))
                    .on_press(Message::CopyLatexExport)
                    .padding([7, 16])
                    .style(theme::button_primary)
                    .into(),
            ],
            DIALOG_LG,
        ))
    }

    pub(crate) fn create_regex_dialog(&self) -> Element<'_, Message> {
        modal_layer(dialog_card(
            "Build from a regular expression",
            Some("Operators: | * + ?, grouping with ( ), \\c escapes, ε for empty.".to_string()),
            field(
                REGEX_INPUT,
                "e.g. (a|b)*abb",
                &self.get_active_tab().regex_text,
                Message::RegexTextChanged,
                Message::SubmitRegex,
            ),
            vec![
                widgets::ghost("Cancel", Some(Message::CancelRegex)).into(),
                widgets::primary("Build", Some(Message::SubmitRegex)).into(),
            ],
            DIALOG_MD,
        ))
    }

    pub(crate) fn create_regex_export_dialog(&self) -> Element<'_, Message> {
        let code = self.regex_export_code.as_deref().unwrap_or("");
        let body = text_input("", code)
            .on_input(|_| Message::OpenRegexExport)
            .padding([9, 12])
            .font(theme::MONO)
            .style(theme::input);
        modal_layer(dialog_card(
            "Equivalent regular expression",
            Some("Obtained by state elimination.".to_string()),
            body.into(),
            vec![
                widgets::ghost("Close", Some(Message::CloseRegexExport)).into(),
                button(widgets::labeled(Some((Icon::Check, Ink::OnAccent)), "Copy"))
                    .on_press(Message::CopyRegexExport)
                    .padding([7, 16])
                    .style(theme::button_primary)
                    .into(),
            ],
            DIALOG_MD,
        ))
    }

    pub(crate) fn create_shortcuts_dialog(&self) -> Element<'_, Message> {
        let entry = |keys: &'static [&'static str], description: &'static str| -> Element<'_, Message> {
            let mut caps = row![].spacing(4).align_y(Alignment::Center);
            for key in keys {
                caps = caps.push(widgets::kbd(*key));
            }
            row![
                container(caps).width(Length::Fixed(128.0)),
                text(description).size(13).style(theme::text_dim),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .into()
        };
        let section = |title: &'static str, entries: Vec<Element<'static, Message>>| -> Element<'static, Message> {
            let mut list = column![widgets::caption(title)].spacing(8);
            for entry in entries {
                list = list.push(entry);
            }
            list.width(Length::Fill).into()
        };
        let canvas = section(
            "CANVAS",
            vec![
                entry(&["1"], "Select / move tool"),
                entry(&["2"], "State tool"),
                entry(&["3"], "Transition tool"),
                entry(&["4"], "Delete tool"),
                entry(&["Tab"], "Toggle Select ↔ Delete"),
                entry(&["Del"], "Hold for a quick delete"),
                entry(&["Shift", "click"], "Toggle accepting state"),
                entry(&["Alt", "click"], "Toggle initial state"),
                entry(&["Esc"], "Back to Select"),
            ],
        );
        let view = section(
            "VIEW & RUN",
            vec![
                entry(&["Ctrl", "wheel"], "Zoom around the cursor"),
                entry(&["Ctrl", "+ / −"], "Zoom in / out"),
                entry(&["Ctrl", "0"], "Reset zoom"),
                entry(&["F"], "Fit the machine to the view"),
                entry(&["Space"], "Play / pause the run"),
                entry(&["→"], "Step the run"),
            ],
        );
        let app = section(
            "APP",
            vec![
                entry(&["Ctrl", "T"], "New tab"),
                entry(&["Ctrl", "W"], "Close tab"),
                entry(&["Ctrl", "Tab"], "Next tab"),
                entry(&["Ctrl", "O"], "Open a .ce file"),
                entry(&["Ctrl", "S"], "Save the tab as .ce"),
                entry(&["Ctrl", "Shift", "L"], "Light / dark theme"),
                entry(&["F1"], "This cheat sheet"),
            ],
        );
        let sections: Element<'_, Message> = if self.layout_width() >= 820.0 {
            row![canvas, column![view, app].spacing(18).width(Length::Fill)].spacing(28).into()
        } else {
            column![canvas, view, app].spacing(18).into()
        };
        let max_height = self.window_size.map(|size| size.height - 220.0).unwrap_or(560.0).max(200.0);
        let body = container(scrollable(sections).style(theme::scroll)).max_height(max_height);
        modal_layer(dialog_card(
            "Keyboard shortcuts",
            Some("Double-click a tab to rename it.".to_string()),
            body.into(),
            vec![widgets::primary("Done", Some(Message::ToggleShortcuts)).into()],
            DIALOG_LG + 80.0,
        ))
    }
}

impl super::app::App {
    pub(crate) fn create_close_dialog(&self) -> Element<'_, Message> {
        let Some(request) = self.pending_close else {
            return Space::new(0, 0).into();
        };
        let (title, subtitle, affected, discard_label): (&str, String, Vec<usize>, &str) = match request {
            CloseRequest::App => (
                "Close without saving?",
                "These tabs have changes that are not saved to a .ce file. Closing Moccacino now discards them.".to_string(),
                (0..self.tabs.len()).filter(|&index| self.tabs[index].insight.unsaved).collect(),
                "Close without saving",
            ),
            CloseRequest::Tab(index) => (
                "Close this tab without saving?",
                "This tab has changes that are not saved to a .ce file. Closing it now discards them.".to_string(),
                vec![index],
                "Close tab",
            ),
        };
        let mut list = column![].spacing(6);
        for index in affected {
            let Some(tab) = self.tabs.get(index) else { continue };
            let note = if tab.saved_fingerprint.is_none() { "never saved" } else { "modified" };
            list = list.push(
                row![
                    widgets::family_dot(tab.machine.family(), 8.0),
                    text(tab.name.clone()).size(14).font(theme::SEMIBOLD),
                    text(note).size(12).style(theme::text_faint),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            );
        }
        let body = row![
            container(icon(Icon::Warn, 20.0, Ink::Danger))
                .padding(8)
                .style(theme::chip(theme::Tone::Danger)),
            container(list).padding([6, 12]).width(Length::Fill).style(theme::sunken),
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        modal_layer(dialog_card(
            title,
            Some(subtitle),
            body.into(),
            vec![
                widgets::ghost("Cancel", Some(Message::CancelClose)).into(),
                widgets::secondary("Save…", Some(Message::ConfirmCloseSave)).into(),
                button(widgets::labeled(None, discard_label))
                    .on_press(Message::ConfirmCloseDiscard)
                    .padding([7, 16])
                    .style(theme::button_danger)
                    .into(),
            ],
            DIALOG_MD,
        ))
    }
}
