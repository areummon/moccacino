use iced::keyboard;
use iced::widget::{button, container, horizontal_space, hover, mouse_area, row, column, stack};
use iced::{Element, Alignment, Event, Subscription, Task, Length};

use std::time::Duration;

use super::message::Message;
use super::tab::{Tab, TabMachine};

/* Interval between auto-play ticks of the machine run panels. */
const RUN_TICK_MILLIS: u64 = 250;

/* Dropdown anchoring: the menu bar is a fixed-height strip, so the
 * floating menus hang just below it, aligned to their trigger button. */
const MENU_DROPDOWN_TOP: f32 = 38.0;
const MACHINE_MENU_LEFT: f32 = 8.0;
const OPERATIONS_MENU_LEFT: f32 = 190.0;
const FILE_MENU_LEFT: f32 = 358.0;

#[derive(Default)]
pub struct App {
    pub(crate) tabs: Vec<Box<Tab>>,
    pub(crate) active_tab: usize,
    pub(crate) startup_picker_open: bool,
    pub(crate) startup_selected: usize,
    pub(crate) machine_menu_open: bool,
    pub(crate) file_menu_open: bool,
    pub(crate) load_dialog_open: bool,
    pub(crate) load_path_text: String,
    pub(crate) load_dialog_error: Option<String>,
    pub(crate) save_dialog_open: bool,
    pub(crate) save_path_text: String,
    pub(crate) save_dialog_error: Option<String>,
    // Serialized entity of the active tab, stashed while the save dialog
    // is up so both the Browse and typed-path flows write the same bytes.
    pub(crate) pending_save: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) latex_export_dialog_open: bool,
    pub(crate) latex_export_code: Option<String>,
    pub(crate) regex_export_dialog_open: bool,
    pub(crate) regex_export_code: Option<String>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let mut app = Self::default();
        app.tabs.push(Box::new(Tab::new()));
        // The startup picker blocks the GUI until a module is chosen.
        app.startup_picker_open = true;
        (app, Task::none())
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard = iced::event::listen_with(|event, _status, _| match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                Some(Message::KeyPressed(key))
            }
            Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => {
                Some(Message::KeyReleased(key))
            }
            _ => None,
        });

        // Auto-play ticks for the machine run panels; the subscription only
        // exists while a panel is actively playing, so pauses, halts and tab
        // switches stop the clock by themselves.
        let mut subscriptions = vec![keyboard];
        if self.run_tick_interval().is_some() {
            subscriptions.push(
                iced::time::every(Duration::from_millis(RUN_TICK_MILLIS))
                    .map(|_| Message::RunTick),
            );
        }
        Subscription::batch(subscriptions)
    }

    /* Milliseconds between auto-play ticks, when the active tab has a
     * running panel that should keep stepping; None otherwise. */
    pub(crate) fn run_tick_interval(&self) -> Option<Duration> {
        let tab = self.get_active_tab();
        let (playing, run_loaded, finished) = match &tab.machine {
            TabMachine::Turing(_) => (
                tab.tm_playing,
                tab.tm_run.is_some() || tab.tm_frontier.is_some(),
                tab.tm_run.as_ref().is_some_and(|run| run.finished.is_some())
                    || tab.tm_frontier.as_ref().is_some_and(|frontier| frontier.finished.is_some()),
            ),
            TabMachine::Pushdown(_) => (
                tab.pda_playing,
                tab.pda_run.is_some() || tab.pda_frontier.is_some(),
                tab.pda_run.as_ref().is_some_and(|run| run.finished.is_some())
                    || tab.pda_frontier.as_ref().is_some_and(|frontier| frontier.finished.is_some()),
            ),
            TabMachine::Finite(_) => (
                tab.finite_playing,
                tab.finite_run.is_some() || tab.finite_frontier.is_some(),
                tab.finite_run.as_ref().is_some_and(|run| run.finished.is_some())
                    || tab.finite_frontier.as_ref().is_some_and(|frontier| frontier.finished.is_some()),
            ),
            TabMachine::Grammar(_) => (false, false, false),
        };
        if playing && run_loaded && !finished {
            Some(Duration::from_millis(RUN_TICK_MILLIS))
        } else {
            None
        }
    }

    pub(crate) fn get_active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    pub(crate) fn get_active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    /* True while any modal dialog is up on the active tab or globally, or
     * while the startup picker forces a choice. Used to block canvas input
     * and canvas shortcuts behind popups. */
    pub(crate) fn any_modal_open(&self) -> bool {
        self.startup_picker_open
            || self.load_dialog_open
            || self.save_dialog_open
            || self.error_message.is_some()
            || self.latex_export_dialog_open
            || self.regex_export_dialog_open
            || {
                let tab = self.get_active_tab();
                tab.editing_state.is_some()
                    || tab.editing_transition.is_some()
                    || tab.pending_transition_dialog_open
                    || tab.editing_transition_dialog_open
                    || tab.check_input_dialog_open
                    || tab.check_result_popup_open
                    || tab.regex_dialog_open
            }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Canvas(canvas_message) => self.handle_canvas_message(canvas_message),
            Message::SelectTool(tool) => self.select_tool(tool),
            Message::KeyPressed(key) => self.handle_key_pressed(key),
            Message::KeyReleased(key) => self.handle_key_released(key),
            Message::Clear => self.clear_active_tab(),
            Message::EditTextChanged(text) => self.edit_text_changed(text),
            Message::FinishEditing => self.finish_editing(),
            Message::CancelEditing => self.cancel_editing(),
            Message::ToggleOperationsMenu => self.toggle_operations_menu(),
            Message::ToggleMachineMenu => self.toggle_machine_menu(),
            Message::CloseMenus => self.close_menus(),
            Message::DismissModal => self.dismiss_modal(),
            Message::Noop => Task::none(),
            Message::ChooseStartupModule(index) => self.choose_startup_module(index),
            Message::ExitApp => iced::exit(),
            Message::OpenLoadDialog => {
                self.file_menu_open = false;
                self.open_load_dialog()
            }
            Message::LoadPathChanged(text) => self.load_path_changed(text),
            Message::LoadBrowseClicked => self.load_browse_clicked(),
            Message::LoadBrowseResult { result } => self.load_browse_result(result),
            Message::LoadPathSubmitted => self.load_path_submitted(),
            Message::CancelLoadDialog => self.cancel_load_dialog(),
            Message::ToggleFileMenu => self.toggle_file_menu(),
            Message::OpenSaveDialog => {
                self.file_menu_open = false;
                self.open_save_dialog()
            }
            Message::SavePathChanged(text) => self.save_path_changed(text),
            Message::SaveBrowseClicked => self.save_browse_clicked(),
            Message::SaveBrowseResult { result } => self.save_browse_result(result),
            Message::SavePathSubmitted => self.save_path_submitted(),
            Message::CancelSaveDialog => self.cancel_save_dialog(),
            Message::CopyLlmPrompt => self.copy_llm_prompt(),
            Message::CheckInput => self.open_check_input(),
            Message::DfaToNfa => self.dfa_to_nfa(),
            Message::Minimize => self.minimize(),
            Message::CheckInputTextChanged(text) => self.check_input_text_changed(text),
            Message::SubmitCheckInput => self.submit_check_input(),
            Message::CancelCheckInput => self.cancel_check_input(),
            Message::CloseCheckResultPopup => self.close_check_result_popup(),
            Message::OpenRegexDialog => self.open_regex_dialog(),
            Message::RegexTextChanged(text) => self.regex_text_changed(text),
            Message::SubmitRegex => self.submit_regex(),
            Message::CancelRegex => self.cancel_regex(),
            Message::AddTab => {
                self.machine_menu_open = false;
                self.add_tab()
            }
            Message::RemoveTab(index) => self.remove_tab(index),
            Message::SwitchTab(index) => self.switch_tab(index),
            Message::AddTuringTab => {
                self.machine_menu_open = false;
                self.add_turing_tab()
            }
            Message::TmInputChanged(text) => self.tm_input_changed(text),
            Message::TmLoadInput => self.tm_load_input(),
            Message::TmStep => self.tm_step(),
            Message::TmReset => self.tm_reset(),
            Message::TmTogglePlay => self.tm_toggle_play(),
            Message::AddPushdownTab => {
                self.machine_menu_open = false;
                self.add_pda_tab()
            }
            Message::PdaInputChanged(text) => self.pda_input_changed(text),
            Message::PdaLoadInput => self.pda_load_input(),
            Message::PdaStep => self.pda_step(),
            Message::PdaReset => self.pda_reset(),
            Message::PdaTogglePlay => self.pda_toggle_play(),
            Message::FiniteInputChanged(text) => self.finite_input_changed(text),
            Message::FiniteLoadInput => self.finite_load_input(),
            Message::FiniteStep => self.finite_step(),
            Message::FiniteReset => self.finite_reset(),
            Message::FiniteTogglePlay => self.finite_toggle_play(),
            Message::RunTick => self.run_tick(),
            Message::AddGrammarTab => {
                self.machine_menu_open = false;
                self.add_grammar_tab()
            }
            Message::GrammarEditorAction(action) => self.grammar_editor_action(action),
            Message::GrammarWordChanged(text) => self.grammar_word_changed(text),
            Message::GrammarParse => self.grammar_parse(),
            Message::GrammarCheckWord => self.grammar_check_word(),
            Message::GrammarDerive => self.grammar_derive(),
            Message::GrammarToCnf => self.grammar_to_cnf(),
            Message::CloseError => self.close_error(),
            Message::OpenLatexExport => self.open_latex_export(),
            Message::CloseLatexExport => self.close_latex_export(),
            Message::CopyLatexExport => self.copy_latex_export(),
            Message::OpenRegexExport => self.open_regex_export(),
            Message::CloseRegexExport => self.close_regex_export(),
            Message::CopyRegexExport => self.copy_regex_export(),
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

        // The tool bar sits between the menu bar and the tab bar, JFLAP
        // style; grammar tabs have no canvas so they skip it.
        let mut content_column = column![menu_bar];
        if !self.active_tab_is_grammar() {
            content_column = content_column.push(self.create_tool_bar());
        }
        content_column = content_column
            .push(tab_bar)
            .push(self.create_main_content())
            .spacing(0);

        // The run panel sits below the canvas on Turing tabs.
        if self.active_tab_is_turing() {
            content_column = content_column.push(self.create_tm_panel());
        }

        // Pushdown tabs get the stack-driven variant of the run panel.
        if self.active_tab_is_pda() {
            content_column = content_column.push(self.create_pda_panel());
        }

        // Finite tabs get the consumption-stepping run panel.
        if self.active_tab_is_finite() {
            content_column = content_column.push(self.create_finite_panel());
        }

        // Grammar tabs swap the canvas workflow for the editing panel.
        if self.active_tab_is_grammar() {
            content_column = content_column.push(self.create_grammar_panel());
        }

        let mut final_content: Element<Message> = content_column.into();

        // Dropdown menus float above everything in a full-window layer
        // that captures click-away presses so they close instead of
        // falling through to the canvas below.
        if self.get_active_tab().operations_menu_open {
            let operations_layer = mouse_area(
                container(self.create_operations_menu())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(iced::Padding {
                        top: MENU_DROPDOWN_TOP,
                        left: OPERATIONS_MENU_LEFT,
                        right: 0.0,
                        bottom: 0.0,
                    })
            )
            .on_press(Message::CloseMenus);
            final_content = stack![final_content, operations_layer].into();
        }

        if self.machine_menu_open {
            let machine_layer = mouse_area(
                container(self.create_machine_menu())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(iced::Padding {
                        top: MENU_DROPDOWN_TOP,
                        left: MACHINE_MENU_LEFT,
                        right: 0.0,
                        bottom: 0.0,
                    })
            )
            .on_press(Message::CloseMenus);
            final_content = stack![final_content, machine_layer].into();
        }

        if self.file_menu_open {
            let file_layer = mouse_area(
                container(self.create_file_menu())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(iced::Padding {
                        top: MENU_DROPDOWN_TOP,
                        left: FILE_MENU_LEFT,
                        right: 0.0,
                        bottom: 0.0,
                    })
            )
            .on_press(Message::CloseMenus);
            final_content = stack![final_content, file_layer].into();
        }

        if self.get_active_tab().editing_state.is_some() || self.get_active_tab().editing_transition.is_some() {
            let edit_dialog = self.create_edit_dialog();
            final_content = iced::widget::stack![final_content, edit_dialog].into();
        }

        if self.get_active_tab().check_input_dialog_open {
            let check_input_dialog = self.create_check_input_dialog();
            final_content = iced::widget::stack![final_content, check_input_dialog].into();
        }

        if self.get_active_tab().regex_dialog_open {
            let regex_dialog = self.create_regex_dialog();
            final_content = iced::widget::stack![final_content, regex_dialog].into();
        }

        if self.get_active_tab().check_result_popup_open {
            let check_result_popup = self.create_check_result_popup();
            final_content = iced::widget::stack![final_content, check_result_popup].into();
        }

        if self.error_message.is_some() {
            let error_popup = self.create_error_popup();
            final_content = iced::widget::stack![final_content, error_popup].into();
        }

        // The .ce loader dialog sits above the app but below the error
        // popup so load failures remain visible.
        if self.load_dialog_open {
            let load_dialog = self.create_load_dialog();
            final_content = iced::widget::stack![final_content, load_dialog].into();
        }

        // The .ce save dialog layers next to the loader dialog.
        if self.save_dialog_open {
            let save_dialog = self.create_save_dialog();
            final_content = iced::widget::stack![final_content, save_dialog].into();
        }

        if self.latex_export_dialog_open {
            let latex_dialog = self.create_latex_export_dialog();
            final_content = iced::widget::stack![final_content, latex_dialog].into();
        }

        if self.regex_export_dialog_open {
            let regex_dialog = self.create_regex_export_dialog();
            final_content = iced::widget::stack![final_content, regex_dialog].into();
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

        // The startup picker sits above everything and cannot be dismissed.
        if self.startup_picker_open {
            let picker = self.create_startup_picker();
            final_content = iced::widget::stack![final_content, picker].into();
        }

        container(final_content)
            .style(|_theme: &iced::Theme| {
                container::Style {
                    background: Some(crate::gui::theme::BG.into()),
                    border: iced::Border::default(),
                    ..Default::default()
                }
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(0)
            .into()
    }

    /* The canvas fills the whole viewport; the wheel pans its scroll
     * offset so drawings outside the visible area stay reachable. */
    fn create_main_content(&self) -> Element<'_, Message> {
        container(hover(
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
                        .style(|_theme: &iced::Theme, status| {
                            crate::gui::theme::danger_button(status)
                        })
                        .on_press(Message::Clear)
                ]
                .align_y(Alignment::Center)
                )
                .padding(10)
            },
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme: &iced::Theme| {
            container::Style {
                background: None,
                border: iced::Border::default(),
                ..Default::default()
            }
        })
        .padding(0)
        .into()
    }
}
