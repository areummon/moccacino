use iced::widget::{button, column, container, horizontal_space, row, scrollable, text};
use iced::{Alignment, Element, Length};

use super::Message;
use crate::gui::theme;

impl super::app::App {
    pub(crate) fn create_menu_bar(&self) -> Element<'_, Message> {
        let abstract_machine_button = button(
            row![
                text("Computational Entity").size(14).color(theme::CREAM),
                text("▾").size(10).color(theme::TEXT_DIM),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
        )
        .on_press(Message::ToggleMachineMenu)
        .style(move |_theme: &iced::Theme, status| {
            theme::bar_button(self.machine_menu_open, status)
        })
        .padding([5, 12]);

        let operations_button = button(
            row![
                text("Operation").size(14).color(theme::CREAM),
                text("▾").size(10).color(theme::TEXT_DIM),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
        )
        .on_press(Message::ToggleOperationsMenu)
        .style(move |_theme: &iced::Theme, status| {
            theme::bar_button(self.get_active_tab().operations_menu_open, status)
        })
        .padding([5, 12]);

        let latex_button = with_tooltip(
            button(text("LaTeX").size(14).color(theme::CREAM))
                .on_press(Message::OpenLatexExport)
                .style(|_theme: &iced::Theme, status| theme::bar_button(false, status))
                .padding([5, 12]),
            "Shows this tab as TikZ/LaTeX code — an automaton diagram or the \
             grammar's productions — ready to copy into a LaTeX document",
        );

        let file_button = button(
            row![
                text("File").size(14).color(theme::CREAM),
                text("▾").size(10).color(theme::TEXT_DIM),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
        )
        .on_press(Message::ToggleFileMenu)
        .style(move |_theme: &iced::Theme, status| {
            theme::bar_button(self.file_menu_open, status)
        })
        .padding([5, 12]);

        let prompt_button = with_tooltip(
            button(text("Copy LLM Prompt").size(14).color(theme::CREAM))
                .on_press(Message::CopyLlmPrompt)
                .style(|_theme: &iced::Theme, status| theme::bar_button(false, status))
                .padding([5, 12]),
            "Copies a ready-made prompt that turns a picture of a state diagram \
             into a loadable .ce file — paste it together with the image into any \
             vision-capable LLM",
        );

        container(
            row![
                abstract_machine_button,
                operations_button,
                file_button,
                latex_button,
                prompt_button,
                horizontal_space(),
            ]
            .spacing(4)
            .align_y(Alignment::Center)
        )
        .style(|_theme: &iced::Theme| theme::panel_box())
        .padding([4, 8])
        .width(Length::Fill)
        .into()
    }

    /* The editing-tool bar (JFLAP style): one button per canvas tool, the
     * active one highlighted, each with a tooltip describing its
     * interactions. Number keys switch tools as well. */
    pub(crate) fn create_tool_bar(&self) -> Element<'_, Message> {
        let active = self.get_active_tab().active_tool();

        let tool_button = |tool: crate::state_machine::EditorTool| {
            let is_active = tool == active;
            button(
                text(tool.label())
                    .size(14)
                    .color(if is_active { theme::BG } else { theme::CREAM }),
            )
            .on_press(Message::SelectTool(tool))
            .style(move |_theme: &iced::Theme, status| {
                if is_active {
                    theme::primary_button(status)
                } else {
                    theme::secondary_button(status)
                }
            })
            .padding([4, 10])
        };

        let arrow = crate::state_machine::EditorTool::Arrow;
        let state = crate::state_machine::EditorTool::State;
        let transition = crate::state_machine::EditorTool::Transition;
        let delete = crate::state_machine::EditorTool::Delete;

        container(
            row![
                with_tooltip(tool_button(arrow), arrow.tooltip()),
                with_tooltip(tool_button(state), state.tooltip()),
                with_tooltip(tool_button(transition), transition.tooltip()),
                with_tooltip(tool_button(delete), delete.tooltip()),
                horizontal_space(),
            ]
            .spacing(4)
            .align_y(Alignment::Center)
        )
        .style(|_theme: &iced::Theme| theme::panel_box())
        .padding([4, 8])
        .width(Length::Fill)
        .into()
    }

    /* The "Computational Entity" dropdown: every machine family that can
     * be opened as a new tab, in a scrollable list. */
    pub(crate) fn create_machine_menu(&self) -> Element<'_, Message> {
        let menu_items = column![
            self.machine_menu_item("Finite Automaton", Message::AddTab),
            self.machine_menu_item("Turing Machine", Message::AddTuringTab),
            self.machine_menu_item("Pushdown Automaton", Message::AddPushdownTab),
            self.machine_menu_item("Grammar", Message::AddGrammarTab),
        ]
        .spacing(2)
        .width(Length::Fill);

        container(
            scrollable(menu_items)
                .height(Length::Fixed(150.0))
                .width(Length::Fixed(210.0))
                .style(|_theme: &iced::Theme, _status| theme::menu_scroll())
        )
        .style(|_theme: &iced::Theme| theme::menu_panel())
        .padding(4)
        .into()
    }

    fn machine_menu_item<'a>(&self, label: &'a str, message: Message) -> Element<'a, Message> {
        button(
            text(label)
                .size(14)
                .color(theme::CREAM)
        )
        .on_press(message)
        .width(Length::Fill)
        .style(|_theme: &iced::Theme, status| theme::menu_item(status))
        .padding([6, 10])
        .into()
    }

    /* The "File" dropdown: loading and saving .ce files. */
    pub(crate) fn create_file_menu(&self) -> Element<'_, Message> {
        let menu_items = column![
            self.machine_menu_item("Load .ce…", Message::OpenLoadDialog),
            self.machine_menu_item("Save .ce…", Message::OpenSaveDialog),
        ]
        .spacing(2)
        .width(Length::Fixed(160.0));

        container(menu_items)
            .style(|_theme: &iced::Theme| theme::menu_panel())
            .padding(4)
            .into()
    }

    pub(crate) fn create_operations_menu(&self) -> Element<'_, Message> {        let menu_items = column![
            self.operations_menu_item("Check Input", Message::CheckInput),
            self.operations_menu_item("DFA to NFA", Message::DfaToNfa),
            self.operations_menu_item("Minimize", Message::Minimize),
            self.operations_menu_item("Build from Regex", Message::OpenRegexDialog),
            self.operations_menu_item("To Regex", Message::OpenRegexExport),
        ]
        .spacing(2)
        .width(Length::Fixed(160.0));

        container(menu_items)
            .style(|_theme: &iced::Theme| theme::menu_panel())
            .padding(4)
            .into()
    }

    fn operations_menu_item<'a>(&self, label: &'a str, message: Message) -> Element<'a, Message> {
        button(
            text(label)
                .size(14)
                .color(theme::CREAM)
        )
        .on_press(message)
        .width(Length::Fill)
        .style(|_theme: &iced::Theme, status| theme::menu_item(status))
        .padding([6, 10])
        .into()
    }

    pub(crate) fn create_tab_bar(&self) -> Element<'_, Message> {
        let mut tab_buttons = row![].spacing(4);

        for (index, tab) in self.tabs.iter().enumerate() {
            let is_active = index == self.active_tab;
            let tab_button = button(
                row![
                    text(&tab.name)
                        .size(14)
                        .color(if is_active { theme::BG } else { theme::CREAM }),
                    if self.tabs.len() > 1 {
                        button("×")
                            .on_press(Message::RemoveTab(index))
                            .style(|_theme: &iced::Theme, status| {
                                let (background, text_color) = match status {
                                    button::Status::Hovered => (theme::DANGER, theme::CREAM),
                                    _ => (iced::Color::TRANSPARENT, theme::CREAM),
                                };
                                button::Style {
                                    background: Some(background.into()),
                                    text_color,
                                    border: iced::Border {
                                        radius: 4.0.into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
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
                theme::tab_button(is_active, status)
            })
            .padding([5, 12]);

            tab_buttons = tab_buttons.push(tab_button);
        }

        let new_tab_button = button(
            text("+")
                .size(16)
                .color(theme::CREAM)
        )
        .on_press(Message::AddTab)
        .style(|_theme: &iced::Theme, status| theme::secondary_button(status))
        .padding([3, 10]);

        tab_buttons = tab_buttons.push(new_tab_button);

        container(tab_buttons)
            .style(|_theme: &iced::Theme| theme::panel_box())
            .padding([4, 8])
            .width(Length::Fill)
            .into()
    }
}
/* Wraps a toolbar button in a bottom tooltip with a short description of
 * what it does. */
fn with_tooltip<'a, E>(
    button: E,
    description: &'static str,
) -> Element<'a, Message>
where
    E: Into<iced::Element<'a, Message>>,
{
    iced::widget::tooltip::Tooltip::new(
        button.into(),
        container(text(description).size(12).color(theme::CREAM)).max_width(340),
        iced::widget::tooltip::Position::Bottom,
    )
    .style(|_theme: &iced::Theme| theme::menu_panel())
    .into()
}
