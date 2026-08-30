use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

use super::Message;
use crate::gui::theme;

/* Wraps dialog content in the full-window modal layer. Two mouse_area
 * levels make clicks never reach the canvas below: the outer one captures
 * backdrop clicks and dismisses the dialog, the inner one swallows clicks
 * on the dialog box's empty areas so they don't dismiss it either. */
fn modal_layer<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    iced::widget::mouse_area(
        container(
            iced::widget::mouse_area(content).on_press(Message::Noop)
        )
        .center(Length::Fill)
        .style(|_theme: &iced::Theme| theme::scrim())
    )
    .on_press(Message::DismissModal)
    .into()
}

/* Modal layer without a dismissal path: the backdrop swallows clicks.
 * Used for the startup picker, where a choice is mandatory. */
fn blocking_layer<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    iced::widget::mouse_area(
        container(
            iced::widget::mouse_area(content).on_press(Message::Noop)
        )
        .center(Length::Fill)
        .style(|_theme: &iced::Theme| theme::scrim())
    )
    .on_press(Message::Noop)
    .into()
}

fn dialog_button<'a>(label: &'a str, message: Message, primary: bool) -> Element<'a, Message> {
    button(
        text(label)
            .size(14)
            .color(if primary { theme::BG } else { theme::CREAM })
    )
    .on_press(message)
    .style(move |_theme: &iced::Theme, status| {
        if primary {
            theme::primary_button(status)
        } else {
            theme::secondary_button(status)
        }
    })
    .padding([5, 12])
    .into()
}

impl super::app::App {
    /* Load `.ce` dialog: a path field that always works plus a native
     * Browse button for systems with a working file chooser. */
    pub(crate) fn create_load_dialog(&self) -> Element<'_, Message> {
        let dialog = container(
            column![
                text("Load computational entities")
                    .size(17)
                    .color(theme::CREAM),
                text("Open a .ce file; each entity in it becomes a tab.")
                    .size(13)
                    .color(theme::TEXT_FAINT),
                text_input("/path/to/file.ce", &self.load_path_text)
                    .on_input(Message::LoadPathChanged)
                    .on_submit(Message::LoadPathSubmitted)
                    .width(380)
                    .style(|_theme: &iced::Theme, status| theme::input(status)),
                if let Some(error) = &self.load_dialog_error {
                    text(error.clone())
                        .size(13)
                        .color(theme::REJECT)
                } else {
                    text(" ")
                        .size(13)
                        .color(theme::TEXT_FAINT)
                },
                row![
                    button(
                        text("Browse…").size(14).color(theme::CREAM)
                    )
                    .on_press(Message::LoadBrowseClicked)
                    .style(|_theme: &iced::Theme, status| theme::secondary_button(status))
                    .padding([5, 12]),
                    dialog_button("Load", Message::LoadPathSubmitted, true),
                    dialog_button("Cancel", Message::CancelLoadDialog, false),
                ]
                .spacing(8)
            ]
            .spacing(10)
            .padding(16)
            .width(440)
        )
        .style(|_theme: &iced::Theme| theme::dialog_box());

        modal_layer(dialog.into())
    }

    /* Save `.ce` dialog: mirrors the load dialog — a path field that
     * always works plus a native Browse button for systems with a working
     * file chooser. The entity text is already serialized into
     * `pending_save` when this opens. */
    pub(crate) fn create_save_dialog(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let dialog = container(
            column![
                text("Save computational entity")
                    .size(17)
                    .color(theme::CREAM),
                text(format!(
                    "Write the active tab ({}) to a .ce file.",
                    tab.name
                ))
                .size(13)
                .color(theme::TEXT_FAINT),
                text_input("/path/to/file.ce", &self.save_path_text)
                    .on_input(Message::SavePathChanged)
                    .on_submit(Message::SavePathSubmitted)
                    .width(380)
                    .style(|_theme: &iced::Theme, status| theme::input(status)),
                if let Some(error) = &self.save_dialog_error {
                    text(error.clone())
                        .size(13)
                        .color(theme::REJECT)
                } else {
                    text(" ")
                        .size(13)
                        .color(theme::TEXT_FAINT)
                },
                row![
                    button(
                        text("Browse…").size(14).color(theme::CREAM)
                    )
                    .on_press(Message::SaveBrowseClicked)
                    .style(|_theme: &iced::Theme, status| theme::secondary_button(status))
                    .padding([5, 12]),
                    dialog_button("Save", Message::SavePathSubmitted, true),
                    dialog_button("Cancel", Message::CancelSaveDialog, false),
                ]
                .spacing(8)
            ]
            .spacing(10)
            .padding(16)
            .width(440)
        )
        .style(|_theme: &iced::Theme| theme::dialog_box());

        modal_layer(dialog.into())
    }

    /* First-screen chooser: pick the module family to open before the main
     * GUI unlocks. Keyboard navigation (↑ ↓ + Enter) is handled in
     * `handle_key_pressed`; the highlight mirrors `startup_selected`. */
    pub(crate) fn create_startup_picker(&self) -> Element<'_, Message> {
        const OPTIONS: [&str; 4] = [
            "Finite Automaton",
            "Turing Machine",
            "Pushdown Automaton",
            "Grammar",
        ];

        let mut options = column![].spacing(2);
        for (index, label) in OPTIONS.iter().enumerate() {
            let selected = index == self.startup_selected;
            options = options.push(
                button(
                    text(label.to_string())
                        .size(14)
                        .color(if selected { theme::BG } else { theme::CREAM })
                )
                .on_press(Message::ChooseStartupModule(index))
                .width(Length::Fill)
                .style(move |_theme: &iced::Theme, status| {
                    if selected {
                        theme::menu_item_selected(status)
                    } else {
                        theme::menu_item(status)
                    }
                })
                .padding([8, 12])
            );
        }

        let dialog = container(
            column![
                text("Choose a computational entity")
                    .size(17)
                    .color(theme::CREAM),
                options,
                text("↑ ↓ navigate · Enter opens · Esc exits")
                    .size(12)
                    .color(theme::TEXT_FAINT),
                row![
                    dialog_button("Exit", Message::ExitApp, false),
                ]
            ]
            .spacing(12)
            .padding(20)
            .width(300)
        )
        .style(|_theme: &iced::Theme| theme::dialog_box());

        blocking_layer(dialog.into())
    }

    pub(crate) fn create_check_input_dialog(&self) -> Element<'_, Message> {
        let dialog = container(
            column![
                text("Input String:")
                    .size(15)
                    .color(theme::CREAM),
                text_input("Enter input string...", &self.get_active_tab().check_input_text)
                    .on_input(Message::CheckInputTextChanged)
                    .on_submit(Message::SubmitCheckInput)
                    .width(220)
                    .style(|_theme: &iced::Theme, status| theme::input(status)),
                row![
                    dialog_button("Accept", Message::SubmitCheckInput, true),
                    dialog_button("Cancel", Message::CancelCheckInput, false),
                ]
                .spacing(8)
            ]
            .spacing(10)
            .padding(16)
            .width(260)
        )
        .style(|_theme: &iced::Theme| theme::dialog_box());

        modal_layer(dialog.into())
    }

    pub(crate) fn create_check_result_popup(&self) -> Element<'_, Message> {
        let result_text = if let Some(result) = self.get_active_tab().check_input_result {
            if result {
                "Input is accepted by the automaton :)"
            } else {
                "Input is rejected by the automaton :("
            }
        } else {
            "No result available"
        };

        let dialog = container(
            column![
                text(result_text)
                    .size(15)
                    .color(theme::CREAM),
                dialog_button("Close", Message::CloseCheckResultPopup, false),
            ]
            .spacing(10)
            .padding(16)
            .width(260)
        )
        .style(|_theme: &iced::Theme| theme::dialog_box());

        modal_layer(dialog.into())
    }

    pub(crate) fn create_error_popup(&self) -> Element<'_, Message> {
        if let Some(error_message) = &self.error_message {
            let dialog = container(
                column![
                    text(error_message.clone())
                        .size(15)
                        .color(theme::REJECT),
                    dialog_button("Close", Message::CloseError, false),
                ]
                .spacing(10)
                .padding(16)
                .width(280)
            )
            .style(|_theme: &iced::Theme| theme::dialog_box());

            modal_layer(dialog.into())
        } else {
            container(iced::widget::horizontal_space()).into()
        }
    }

    pub(crate) fn create_edit_dialog(&self) -> Element<'_, Message> {
        if self.get_active_tab().pending_transition_dialog_open {
            let dialog = container(
                column![
                    text("Enter transition label:")
                        .size(15)
                        .color(theme::CREAM),
                    text_input("Enter label...", &self.get_active_tab().pending_transition_label)
                        .on_input(Message::EditTextChanged)
                        .on_submit(Message::FinishEditing)
                        .width(170)
                        .style(|_theme: &iced::Theme, status| theme::input(status)),
                    row![
                        dialog_button("Save", Message::FinishEditing, true),
                        dialog_button("Cancel", Message::CancelEditing, false),
                    ]
                    .spacing(8)
                ]
                .spacing(10)
                .padding(16)
                .width(230)
            )
            .style(|_theme: &iced::Theme| theme::dialog_box());

            return modal_layer(dialog.into());
        }
        if self.get_active_tab().editing_transition_dialog_open {
            let active_tab = self.get_active_tab();
            let mut label_inputs = column![];
            for (i, label) in active_tab.editing_transition_label_inputs.iter().enumerate() {
                label_inputs = label_inputs.push(
                    row![
                        text_input("Label", label)
                            .on_input(move |text| Message::EditTransitionLabelChanged(i, text))
                            .width(120)
                            .style(|_theme: &iced::Theme, status| theme::input(status)),
                        button(
                            text("Delete").size(13).color(theme::CREAM)
                        )
                        .on_press(Message::DeleteEditTransitionLabel(i))
                        .style(|_theme: &iced::Theme, status| theme::danger_button(status))
                        .padding([3, 8])
                    ].spacing(8)
                );
            }
            let dialog = container(
                column![
                    text("Edit Transition Labels:")
                        .size(15)
                        .color(theme::CREAM),
                    scrollable(label_inputs)
                        .height(Length::Fixed(180.0))
                        .style(|_theme: &iced::Theme, _status| theme::menu_scroll()),
                    dialog_button("Add Label", Message::AddEditTransitionLabel, false),
                    row![
                        dialog_button("Save", Message::SaveEditTransitionLabels, true),
                        dialog_button("Cancel", Message::CancelEditTransitionLabels, false),
                    ]
                    .spacing(8)
                ]
                .spacing(10)
                .padding(16)
                .width(260)
            )
            .style(|_theme: &iced::Theme| theme::dialog_box());

            return modal_layer(dialog.into());
        }
        let dialog = container(
            column![
                text(if self.get_active_tab().editing_state.is_some() { "Edit State:" } else { "Edit Transition:" })
                    .size(15)
                    .color(theme::CREAM),
                text_input("Enter label...", &self.get_active_tab().edit_text)
                    .on_input(Message::EditTextChanged)
                    .on_submit(Message::FinishEditing)
                    .width(170)
                    .style(|_theme: &iced::Theme, status| theme::input(status)),
                row![
                    dialog_button("Save", Message::FinishEditing, true),
                    dialog_button("Cancel", Message::CancelEditing, false),
                ]
                .spacing(8)
            ]
            .spacing(10)
            .padding(16)
            .width(230)
        )
        .style(|_theme: &iced::Theme| theme::dialog_box());

        modal_layer(dialog.into())
    }

    pub(crate) fn create_latex_export_dialog(&self) -> Element<'_, Message> {
        let code = self.latex_export_code.as_deref().unwrap_or("");
        let is_grammar = matches!(
            self.get_active_tab().machine,
            super::tab::TabMachine::Grammar(_)
        );
        let title = if is_grammar {
            "LaTeX code for this grammar:"
        } else {
            "LaTeX (TikZ) code for this automaton:"
        };
        let dialog = container(
            column![
                text(title)
                    .size(15)
                    .color(theme::CREAM),
                scrollable(
                    container(
                        text(code.to_string())
                            .font(iced::Font::MONOSPACE)
                            .size(13)
                            .color(theme::CREAM),
                    )
                    .padding(10)
                    .style(|_theme: &iced::Theme| theme::inset_box())
                )
                .width(Length::Fixed(560.0))
                .height(Length::Fixed(300.0))
                .style(|_theme: &iced::Theme, _status| theme::menu_scroll()),
                row![
                    dialog_button("Copy", Message::CopyLatexExport, true),
                    dialog_button("Close", Message::CloseLatexExport, false),
                ]
                .spacing(8)
            ]
            .spacing(10)
            .padding(16)
            .width(600)
        )
        .style(|_theme: &iced::Theme| theme::dialog_box());

        modal_layer(dialog.into())
    }

    pub(crate) fn create_regex_dialog(&self) -> Element<'_, Message> {
        let dialog = container(
            column![
                text("Regular expression:")
                    .size(15)
                    .color(theme::CREAM),
                text_input("e.g. (a|b)*abb", &self.get_active_tab().regex_text)
                    .on_input(Message::RegexTextChanged)
                    .on_submit(Message::SubmitRegex)
                    .width(220)
                    .style(|_theme: &iced::Theme, status| theme::input(status)),
                row![
                    dialog_button("Accept", Message::SubmitRegex, true),
                    dialog_button("Cancel", Message::CancelRegex, false),
                ]
                .spacing(8)
            ]
            .spacing(10)
            .padding(16)
            .width(280)
        )
        .style(|_theme: &iced::Theme| theme::dialog_box());

        modal_layer(dialog.into())
    }

    pub(crate) fn create_regex_export_dialog(&self) -> Element<'_, Message> {
        let code = self.regex_export_code.as_deref().unwrap_or("");
        let dialog = container(
            column![
                text("Equivalent regular expression:")
                    .size(15)
                    .color(theme::CREAM),
                text_input("", code)
                    .width(440)
                    .on_input(|_| Message::OpenRegexExport)
                    .style(|_theme: &iced::Theme, status| theme::input(status)),
                row![
                    dialog_button("Copy", Message::CopyRegexExport, true),
                    dialog_button("Close", Message::CloseRegexExport, false),
                ]
                .spacing(8)
            ]
            .spacing(10)
            .padding(16)
            .width(480)
        )
        .style(|_theme: &iced::Theme| theme::dialog_box());

        modal_layer(dialog.into())
    }
}