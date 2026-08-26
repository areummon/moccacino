use iced::keyboard;
use iced::widget::{button, container, horizontal_space, hover, row, column, stack};
use iced::{Element, Alignment, Event, Subscription, Task, Length};

use super::message::Message;
use super::tab::Tab;

#[derive(Default)]
pub struct App {
    pub(crate) tabs: Vec<Box<Tab>>,
    pub(crate) active_tab: usize,
    pub(crate) error_message: Option<String>,
    pub(crate) latex_export_dialog_open: bool,
    pub(crate) latex_export_code: Option<String>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let mut app = Self::default();
        app.tabs.push(Box::new(Tab::new()));
        (app, Task::none())
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::event::listen_with(|event, _status, _| match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                Some(Message::KeyPressed(key))
            }
            Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => {
                Some(Message::KeyReleased(key))
            }
            _ => None,
        })
    }

    pub(crate) fn get_active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    pub(crate) fn get_active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Canvas(canvas_message) => self.handle_canvas_message(canvas_message),
            Message::KeyPressed(key) => self.handle_key_pressed(key),
            Message::KeyReleased(key) => self.handle_key_released(key),
            Message::Clear => self.clear_active_tab(),
            Message::EditTextChanged(text) => self.edit_text_changed(text),
            Message::FinishEditing => self.finish_editing(),
            Message::CancelEditing => self.cancel_editing(),
            Message::ToggleOperationsMenu => self.toggle_operations_menu(),
            Message::CheckInput => self.open_check_input(),
            Message::DfaToNfa => self.dfa_to_nfa(),
            Message::Minimize => self.minimize(),
            Message::CheckInputTextChanged(text) => self.check_input_text_changed(text),
            Message::SubmitCheckInput => self.submit_check_input(),
            Message::CancelCheckInput => self.cancel_check_input(),
            Message::CloseCheckResultPopup => self.close_check_result_popup(),
            Message::AddTab => self.add_tab(),
            Message::RemoveTab(index) => self.remove_tab(index),
            Message::SwitchTab(index) => self.switch_tab(index),
            Message::CloseError => self.close_error(),
            Message::OpenLatexExport => self.open_latex_export(),
            Message::CloseLatexExport => self.close_latex_export(),
            Message::CopyLatexExport => self.copy_latex_export(),
            Message::OpenEditTransition(pair) => self.open_edit_transition(pair),
            Message::EditTransitionLabelChanged(idx, text) => self.edit_transition_label_changed(idx, text),
            Message::SaveEditTransitionLabels => self.save_edit_transition_labels(),
            Message::DeleteEditTransitionLabel(idx) => self.delete_edit_transition_label(idx),
            Message::AddEditTransitionLabel => self.add_edit_transition_label(),
            Message::CancelEditTransitionLabels => self.cancel_edit_transition_labels(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let menu_bar = self.create_menu_bar();
        let tab_bar = self.create_tab_bar();

        let main_content = container(hover(
            self.get_active_tab().state_machine.view(
                &self.get_active_tab().states,
                &self.get_active_tab().transitions,
                self.get_active_tab().initial_state,
                &self.get_active_tab().final_states
            ).map(Message::Canvas),
            if self.get_active_tab().states.is_empty() && self.get_active_tab().transitions.is_empty() {
                container(horizontal_space())
            } else {
                container(row![
                    horizontal_space(),
                    button("Clear")
                        .style(button::danger)
                        .on_press(Message::Clear)
                ]
                .align_y(Alignment::Center)
                )
                .padding(10)
            },
        ))
        .style(|_theme: &iced::Theme| {
            container::Style {
                background: None,
                border: iced::Border::default(),
                ..Default::default()
            }
        })
        .padding(0);

        let content_with_menu = column![
            menu_bar,
            tab_bar,
            main_content
        ]
        .spacing(0);

        let mut final_content: Element<Message> = if self.get_active_tab().operations_menu_open {
            let operations_menu = container(
                self.create_operations_menu()
            );

            stack![
                content_with_menu,
                container(
                    container(operations_menu)
                        .style(|_theme: &iced::Theme| {
                            container::Style {
                                background: Some(iced::Color::TRANSPARENT.into()),
                                ..Default::default()
                            }
                        })
                )
                .style(|_theme: &iced::Theme| {
                    container::Style {
                        background: Some(iced::Color::TRANSPARENT.into()),
                        ..Default::default()
                    }
                })
                .padding(iced::Padding {
                    top: 40.0,
                    left: 141.0,
                    right: 0.0,
                    bottom: 0.0,
                })
            ].into()
        } else {
            content_with_menu.into()
        };

        if self.get_active_tab().editing_state.is_some() || self.get_active_tab().editing_transition.is_some() {
            let edit_dialog = self.create_edit_dialog();
            final_content = iced::widget::stack![final_content, edit_dialog].into();
        }

        if self.get_active_tab().check_input_dialog_open {
            let check_input_dialog = self.create_check_input_dialog();
            final_content = iced::widget::stack![final_content, check_input_dialog].into();
        }

        if self.get_active_tab().check_result_popup_open {
            let check_result_popup = self.create_check_result_popup();
            final_content = iced::widget::stack![final_content, check_result_popup].into();
        }

        if self.error_message.is_some() {
            let error_popup = self.create_error_popup();
            final_content = iced::widget::stack![final_content, error_popup].into();
        }

        if self.latex_export_dialog_open {
            let latex_dialog = self.create_latex_export_dialog();
            final_content = iced::widget::stack![final_content, latex_dialog].into();
        }

        // Always show the pending transition dialog on top if open
        if self.get_active_tab().pending_transition_dialog_open {
            let pending_dialog = self.create_edit_dialog();
            final_content = iced::widget::stack![final_content, pending_dialog].into();
        }

        // Always show the edit transition labels dialog on top if open
        if self.get_active_tab().editing_transition_dialog_open {
            let edit_labels_dialog = self.create_edit_dialog();
            final_content = iced::widget::stack![final_content, edit_labels_dialog].into();
        }

        container(final_content)
            .style(|_theme: &iced::Theme| {
                container::Style {
                    background: Some(iced::Color::from_rgb(0.1, 0.1, 0.1).into()),
                    border: iced::Border::default(),
                    ..Default::default()
                }
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(0)
            .into()
    }
}
