use iced::widget::{button, column, container, horizontal_space, row, text};
use iced::{Alignment, Element, Length};

use super::Message;

impl super::app::App {
    pub(crate) fn create_menu_bar(&self) -> Element<'_, Message> {
        let abstract_machine_button = button(text("Abstract Machine"))
            .style(|_theme: &iced::Theme, _status| {
                button::Style {
                    background: Some(iced::Color::from_rgba(0.176, 0.172, 0.176, 1.0).into()),
                    text_color: iced::Color::from_rgba(0.6, 0.6, 0.6, 1.0),
                    border: iced::Border::default(),
                    ..Default::default()
                }
            })
            .padding([4, 12]);

        let operations_button = button(text("Operation"))
            .on_press(Message::ToggleOperationsMenu)
            .style(|_theme: &iced::Theme, status| {
                let background_color = iced::Color::from_rgba(0.176, 0.172, 0.176, 1.0);
                let hover_color = iced::Color::from_rgba(0.25, 0.24, 0.25, 1.0);
                let text_color = iced::Color::WHITE;

                match status {
                    button::Status::Hovered => button::Style {
                        background: Some(hover_color.into()),
                        text_color,
                        border: iced::Border::default(),
                        ..Default::default()
                    },
                    _ => button::Style {
                        background: Some(background_color.into()),
                        text_color,
                        border: iced::Border::default(),
                        ..Default::default()
                    }
                }
            })
            .padding([4, 12]);

        let latex_button = button(text("LaTeX"))
            .on_press(Message::OpenLatexExport)
            .style(|_theme: &iced::Theme, status| {
                let background_color = iced::Color::from_rgba(0.176, 0.172, 0.176, 1.0);
                let hover_color = iced::Color::from_rgba(0.25, 0.24, 0.25, 1.0);
                let text_color = iced::Color::WHITE;
                match status {
                    button::Status::Hovered => button::Style {
                        background: Some(hover_color.into()),
                        text_color,
                        border: iced::Border::default(),
                        ..Default::default()
                    },
                    _ => button::Style {
                        background: Some(background_color.into()),
                        text_color,
                        border: iced::Border::default(),
                        ..Default::default()
                    }
                }
            })
            .padding([4, 12]);

        let menu_bar = container(
            row![
                abstract_machine_button,
                operations_button,
                latex_button,
                horizontal_space(),
            ]
            .spacing(4)
            .align_y(Alignment::Center)
        )
        .style(|_theme: &iced::Theme| {
            container::Style {
                background: Some(iced::Color::from_rgba(0.176, 0.172, 0.176, 1.0).into()),
                border: iced::Border {
                    color: iced::Color::from_rgba(0.3, 0.3, 0.3, 1.0),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .padding([4, 8])
        .width(Length::Fill);

        menu_bar.into()
    }

    pub(crate) fn create_operations_menu(&self) -> Element<'_, Message> {
        let menu_background_color = iced::Color::from_rgba(0.15, 0.14, 0.15, 1.0);
        let menu_button_hover_color = iced::Color::from_rgba(0.0, 0.5, 1.0, 1.0);
        let text_color = iced::Color::WHITE;

        let menu_items = column![
            button(text("Check Input"))
                .on_press(Message::CheckInput)
                .width(Length::Fill)
                .style(move |_theme: &iced::Theme, status| {
                    match status {
                        button::Status::Hovered => button::Style {
                            background: Some(menu_button_hover_color.into()),
                            text_color,
                            border: iced::Border::default(),
                            ..Default::default()
                        },
                        _ => button::Style {
                            background: Some(menu_background_color.into()),
                            text_color,
                            border: iced::Border::default(),
                            ..Default::default()
                        }
                    }
                })
                .padding([4, 8]),
            button(text("DFA to NFA"))
                .on_press(Message::DfaToNfa)
                .width(Length::Fill)
                .style(move |_theme: &iced::Theme, status| {
                    match status {
                        button::Status::Hovered => button::Style {
                            background: Some(menu_button_hover_color.into()),
                            text_color,
                            border: iced::Border::default(),
                            ..Default::default()
                        },
                        _ => button::Style {
                            background: Some(menu_background_color.into()),
                            text_color,
                            border: iced::Border::default(),
                            ..Default::default()
                        }
                    }
                })
                .padding([4, 8]),
            button(text("Minimize"))
                .on_press(Message::Minimize)
                .width(Length::Fill)
                .style(move |_theme: &iced::Theme, status| {
                    match status {
                        button::Status::Hovered => button::Style {
                            background: Some(menu_button_hover_color.into()),
                            text_color,
                            border: iced::Border::default(),
                            ..Default::default()
                        },
                        _ => button::Style {
                            background: Some(menu_background_color.into()),
                            text_color,
                            border: iced::Border::default(),
                            ..Default::default()
                        }
                    }
                })
                .padding([4, 8]),
        ]
        .spacing(2)
        .width(120);

        container(menu_items)
            .style(move |_theme: &iced::Theme| {
                container::Style {
                    background: Some(menu_background_color.into()),
                    border: iced::Border {
                        color: iced::Color::from_rgba(0.4, 0.4, 0.4, 1.0),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
            .padding(4)
            .into()
    }

    pub(crate) fn create_tab_bar(&self) -> Element<'_, Message> {
        let tab_bar_background = iced::Color::from_rgba(0.176, 0.172, 0.176, 1.0);
        let active_tab_background = iced::Color::from_rgba(0.25, 0.24, 0.25, 1.0);
        let text_color = iced::Color::WHITE;
        let hover_color = iced::Color::from_rgba(0.3, 0.29, 0.3, 1.0);

        let mut tab_buttons = row![].spacing(2);

        for (index, tab) in self.tabs.iter().enumerate() {
            let is_active = index == self.active_tab;
            let tab_button = button(
                row![
                    text(&tab.name)
                        .size(14)
                        .color(text_color),
                    if self.tabs.len() > 1 {
                        button("×")
                            .on_press(Message::RemoveTab(index))
                            .style(move |_theme: &iced::Theme, status| {
                                match status {
                                    button::Status::Hovered => button::Style {
                                        background: Some(iced::Color::from_rgba(0.8, 0.2, 0.2, 1.0).into()),
                                        text_color: iced::Color::WHITE,
                                        border: iced::Border::default(),
                                        ..Default::default()
                                    },
                                    _ => button::Style {
                                        background: Some(iced::Color::TRANSPARENT.into()),
                                        text_color: iced::Color::WHITE,
                                        border: iced::Border::default(),
                                        ..Default::default()
                                    }
                                }
                            })
                            .padding([2, 6])
                    } else {
                        button("")
                            .style(|_theme: &iced::Theme, _status| {
                                button::Style {
                                    background: Some(iced::Color::TRANSPARENT.into()),
                                    text_color: iced::Color::TRANSPARENT,
                                    border: iced::Border::default(),
                                    ..Default::default()
                                }
                            })
                            .padding([2, 6])
                    }
                ]
                .spacing(8)
                .align_y(Alignment::Center)
            )
            .on_press(Message::SwitchTab(index))
            .style(move |_theme: &iced::Theme, status| {
                let background = match (is_active, status) {
                    (true, _) => active_tab_background,
                    (false, button::Status::Hovered) => hover_color,
                    _ => tab_bar_background,
                };
                button::Style {
                    background: Some(background.into()),
                    text_color,
                    border: iced::Border {
                        color: iced::Color::from_rgba(0.3, 0.3, 0.3, 1.0),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            })
            .padding([4, 12]);

            tab_buttons = tab_buttons.push(tab_button);
        }

        let new_tab_button = button(
            text("+")
                .size(16)
                .color(text_color)
        )
        .on_press(Message::AddTab)
        .style(move |_theme: &iced::Theme, status| {
            let background = match status {
                button::Status::Hovered => hover_color,
                _ => tab_bar_background,
            };
            button::Style {
                background: Some(background.into()),
                text_color,
                border: iced::Border {
                    color: iced::Color::from_rgba(0.3, 0.3, 0.3, 1.0),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .padding([4, 12]);

        tab_buttons = tab_buttons.push(new_tab_button);

        container(tab_buttons)
            .style(move |_theme: &iced::Theme| {
                container::Style {
                    background: Some(tab_bar_background.into()),
                    border: iced::Border {
                        color: iced::Color::from_rgba(0.3, 0.3, 0.3, 1.0),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            })
            .padding([4, 8])
            .width(Length::Fill)
            .into()
    }
}
