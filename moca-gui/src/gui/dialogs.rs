use iced::widget::{button, column, container, horizontal_space, row};
use iced::{Element, Length};

use super::Message;

impl super::app::App {
    pub(crate) fn create_check_input_dialog(&self) -> Element<'_, Message> {
        let menu_background_color = iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0);
        let text_color = iced::Color::WHITE;
        let border_color = iced::Color::from_rgba(0.4, 0.4, 0.4, 1.0);

        let dialog = container(
            container(
                iced::widget::column![
                    iced::widget::text("Input String:")
                        .size(17)
                        .color(text_color),
                    iced::widget::text_input("Enter input string...", &self.get_active_tab().check_input_text)
                        .on_input(Message::CheckInputTextChanged)
                        .on_submit(Message::SubmitCheckInput)
                        .width(200)
                        .style(|_theme: &iced::Theme, _status| {
                            iced::widget::text_input::Style {
                                background: iced::Background::Color(iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0)),
                                border: iced::Border {
                                    color: iced::Color::from_rgba(0.0, 0.5, 1.0, 1.0),
                                    width: 2.0,
                                    radius: 4.0.into(),
                                },
                                icon: iced::Color::WHITE,
                                placeholder: iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0),
                                value: iced::Color::WHITE,
                                selection: iced::Color::from_rgba(0.0, 0.5, 1.0, 0.3),
                            }
                        }),
                    row![
                        button("Accept")
                            .on_press(Message::SubmitCheckInput)
                            .padding([4, 8]),
                        button("Cancel")
                            .on_press(Message::CancelCheckInput)
                            .padding([4, 8])
                    ]
                    .spacing(8)
                ]
                .spacing(8)
                .padding(12)
                .width(250)
            )
            .style(move |_theme: &iced::Theme| {
                container::Style {
                    background: Some(menu_background_color.into()),
                    border: iced::Border {
                        color: border_color,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
        )
        .center(iced::Length::Fill)
        .style(|_theme: &iced::Theme| {
            container::Style {
                background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3).into()),
                ..Default::default()
            }
        });

        dialog.into()
    }

    pub(crate) fn create_check_result_popup(&self) -> Element<'_, Message> {
        let menu_background_color = iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0);
        let text_color = iced::Color::WHITE;
        let border_color = iced::Color::from_rgba(0.4, 0.4, 0.4, 1.0);

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
            container(
                iced::widget::column![
                    iced::widget::text(result_text)
                        .size(17)
                        .color(text_color),
                    button("Close")
                        .on_press(Message::CloseCheckResultPopup)
                        .padding([4, 8])
                ]
                .spacing(8)
                .padding(12)
                .width(250)
            )
            .style(move |_theme: &iced::Theme| {
                container::Style {
                    background: Some(menu_background_color.into()),
                    border: iced::Border {
                        color: border_color,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
        )
        .center(iced::Length::Fill)
        .style(|_theme: &iced::Theme| {
            container::Style {
                background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3).into()),
                ..Default::default()
            }
        });

        dialog.into()
    }

    pub(crate) fn create_error_popup(&self) -> Element<'_, Message> {
        if let Some(error_message) = &self.error_message {
            let menu_background_color = iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0);
            let text_color = iced::Color::WHITE;
            let border_color = iced::Color::from_rgba(0.4, 0.4, 0.4, 1.0);

            let dialog = container(
                container(
                    iced::widget::column![
                        iced::widget::text(error_message)
                            .size(17)
                            .color(text_color),
                        button("Close")
                            .on_press(Message::CloseError)
                            .padding([4, 8])
                    ]
                    .spacing(8)
                    .padding(12)
                    .width(250)
                )
                .style(move |_theme: &iced::Theme| {
                    container::Style {
                        background: Some(menu_background_color.into()),
                        border: iced::Border {
                            color: border_color,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                })
            )
            .center(iced::Length::Fill)
            .style(|_theme: &iced::Theme| {
                container::Style {
                    background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3).into()),
                    ..Default::default()
                }
            });

            dialog.into()
        } else {
            container(horizontal_space()).into()
        }
    }

    pub(crate) fn create_edit_dialog(&self) -> Element<'_, Message> {
        let menu_background_color = iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0);
        let text_color = iced::Color::WHITE;
        let border_color = iced::Color::from_rgba(0.4, 0.4, 0.4, 1.0);

        if self.get_active_tab().pending_transition_dialog_open {
            let edit_dialog = container(
                container(
                    iced::widget::column![
                        iced::widget::text("Enter transition label:")
                            .size(17)
                            .color(text_color),
                        iced::widget::text_input("Enter label...", &self.get_active_tab().pending_transition_label)
                            .on_input(Message::EditTextChanged)
                            .on_submit(Message::FinishEditing)
                            .width(150)
                            .style(|_theme: &iced::Theme, _status| {
                                iced::widget::text_input::Style {
                                    background: iced::Background::Color(iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0)),
                                    border: iced::Border {
                                        color: iced::Color::from_rgba(0.0, 0.5, 1.0, 1.0),
                                        width: 2.0,
                                        radius: 4.0.into(),
                                    },
                                    icon: iced::Color::WHITE,
                                    placeholder: iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0),
                                    value: iced::Color::WHITE,
                                    selection: iced::Color::from_rgba(0.0, 0.5, 1.0, 0.3),
                                }
                            }),
                        row![
                            button("Save")
                                .on_press(Message::FinishEditing)
                                .padding([4, 8]),
                            button("Cancel")
                                .on_press(Message::CancelEditing)
                                .padding([4, 8])
                        ]
                        .spacing(8)
                    ]
                    .spacing(8)
                    .padding(12)
                    .width(200)
                )
                .style(move |_theme: &iced::Theme| {
                    container::Style {
                        background: Some(menu_background_color.into()),
                        border: iced::Border {
                            color: border_color,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                })
            )
            .center(iced::Length::Fill)
            .style(|_theme: &iced::Theme| {
                container::Style {
                    background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3).into()),
                    ..Default::default()
                }
            });
            return edit_dialog.into();
        }
        if self.get_active_tab().editing_transition_dialog_open {
            let active_tab = self.get_active_tab();
            let mut label_inputs = column![];
            for (i, label) in active_tab.editing_transition_label_inputs.iter().enumerate() {
                label_inputs = label_inputs.push(
                    row![
                        iced::widget::text_input("Label", label)
                            .on_input(move |text| Message::EditTransitionLabelChanged(i, text))
                            .width(120),
                        button("Delete")
                            .on_press(Message::DeleteEditTransitionLabel(i))
                            .padding([2, 6])
                    ].spacing(8)
                );
            }
            let dialog = container(
                container(
                    column![
                        iced::widget::text("Edit Transition Labels:")
                            .size(17)
                            .style(|_| iced::widget::text::Style { color: Some(iced::Color::WHITE), ..Default::default() }),
                        iced::widget::scrollable(label_inputs.padding([0, 16])).height(Length::Fixed(180.0)),
                        button("Add Label").on_press(Message::AddEditTransitionLabel).padding([4, 8]),
                        row![
                            button("Save").on_press(Message::SaveEditTransitionLabels).padding([4, 8]),
                            button("Cancel").on_press(Message::CancelEditTransitionLabels).padding([4, 8])
                        ].spacing(8)
                    ].spacing(8).padding(12).width(250)
                ).style(|_theme: &iced::Theme| container::Style {
                    background: Some(iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0).into()),
                    border: iced::Border {
                        color: iced::Color::from_rgba(0.4, 0.4, 0.4, 1.0),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
            ).center(iced::Length::Fill)
            .style(|_theme: &iced::Theme| container::Style {
                background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3).into()),
                ..Default::default()
            });
            return dialog.into();
        }
        let edit_dialog = container(
            container(
                iced::widget::column![
                    iced::widget::text(if self.get_active_tab().editing_state.is_some() { "Edit State:" } else { "Edit Transition:" })
                        .size(17)
                        .color(text_color),
                    iced::widget::text_input("Enter label...", &self.get_active_tab().edit_text)
                        .on_input(Message::EditTextChanged)
                        .on_submit(Message::FinishEditing)
                        .width(150)
                        .style(|_theme: &iced::Theme, _status| {
                            iced::widget::text_input::Style {
                                background: iced::Background::Color(iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0)),
                                border: iced::Border {
                                    color: iced::Color::from_rgba(0.0, 0.5, 1.0, 1.0),
                                    width: 2.0,
                                    radius: 4.0.into(),
                                },
                                icon: iced::Color::WHITE,
                                placeholder: iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0),
                                value: iced::Color::WHITE,
                                selection: iced::Color::from_rgba(0.0, 0.5, 1.0, 0.3),
                            }
                        }),
                    row![
                        button("Save")
                            .on_press(Message::FinishEditing)
                            .padding([4, 8]),
                        button("Cancel")
                            .on_press(Message::CancelEditing)
                            .padding([4, 8])
                    ]
                    .spacing(8)
                ]
                .spacing(8)
                .padding(12)
                .width(200)
            )
            .style(move |_theme: &iced::Theme| {
                container::Style {
                    background: Some(menu_background_color.into()),
                    border: iced::Border {
                        color: border_color,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
        )
        .center(iced::Length::Fill)
        .style(|_theme: &iced::Theme| {
            container::Style {
                background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3).into()),
                ..Default::default()
            }
        });

        edit_dialog.into()
    }

    pub(crate) fn create_latex_export_dialog(&self) -> Element<'_, Message> {
        let menu_background_color = iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0);
        let text_color = iced::Color::WHITE;
        let border_color = iced::Color::from_rgba(0.4, 0.4, 0.4, 1.0);
        let code = self.latex_export_code.as_deref().unwrap_or("");
        let dialog = container(
            container(
                iced::widget::column![
                    iced::widget::text("LaTeX (TikZ) code for this automaton:")
                        .size(17)
                        .color(text_color),
                    iced::widget::text_input("", code)
                        .width(400)
                        .on_input(|_| Message::OpenLatexExport)
                        .style(move |_theme: &iced::Theme, _status| {
                            iced::widget::text_input::Style {
                                background: iced::Background::Color(menu_background_color),
                                border: iced::Border {
                                    color: border_color,
                                    width: 1.0,
                                    radius: 4.0.into(),
                                },
                                icon: iced::Color::WHITE,
                                placeholder: iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0),
                                value: iced::Color::WHITE,
                                selection: iced::Color::from_rgba(0.0, 0.5, 1.0, 0.3),
                            }
                        }),
                    row![
                        button("Copy")
                            .on_press(Message::CopyLatexExport)
                            .padding([4, 8]),
                        button("Close")
                            .on_press(Message::CloseLatexExport)
                            .padding([4, 8])
                    ]
                    .spacing(8)
                ]
                .spacing(8)
                .padding(12)
                .width(500)
            )
            .style(move |_theme: &iced::Theme| {
                container::Style {
                    background: Some(menu_background_color.into()),
                    border: iced::Border {
                        color: border_color,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
        )
        .center(iced::Length::Fill)
        .style(|_theme: &iced::Theme| {
            container::Style {
                background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3).into()),
                ..Default::default()
            }
        });
        dialog.into()
    }
}
