use iced::keyboard;
use iced::widget::{column, container, horizontal_rule, stack};
use iced::time::Instant;
use iced::{Element, Event, Length, Size, Subscription, Task};

use std::time::Duration;

use super::entity_loader::{PathDialog, SaveDialog};
use super::message::{Menu, Message};
use super::metrics::{CANVAS_CHROME_RESERVE, DEFAULT_WINDOW};
use super::settings::Settings;
use super::run::RunSession;
use super::tab::Tab;
use super::workspace::CloseRequest;
use super::toast::{Toast, TOAST_EXIT_TICK, TOAST_TICK};
use crate::theme::{self, Palette, ThemeMode};
use crate::family::Family;

const RUN_TICK_MILLIS: u64 = 250;
const DEFAULT_VIEWPORT: Size = Size::new(1100.0, 560.0);

#[derive(Default)]
pub struct App {
    pub(crate) tabs: Vec<Tab>,
    pub(crate) active_tab: usize,
    pub(crate) theme_mode: ThemeMode,
    pub(crate) startup_picker_open: bool,
    pub(crate) startup_selected: usize,
    pub(crate) open_menu: Option<Menu>,
    pub(crate) shortcuts_open: bool,
    pub(crate) load_dialog: Option<PathDialog>,
    pub(crate) save_dialog: Option<SaveDialog>,
    pub(crate) error_message: Option<String>,
    pub(crate) latex_export: Option<String>,
    pub(crate) regex_export: Option<String>,
    pub(crate) toasts: Vec<Toast>,
    pub(crate) next_toast_id: u64,
    pub(crate) canvas_viewport: Option<Size>,
    pub(crate) renaming_tab: Option<usize>,
    pub(crate) tab_rename_text: String,
    pub(crate) last_tab_click: Option<(usize, Instant)>,
    pub(crate) window_size: Option<Size>,
    pub(crate) pending_close: Option<CloseRequest>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let mut tab = Tab::new_of(Family::Finite);
        tab.refresh_insight();
        let app = Self {
            theme_mode: Settings::load().theme,
            tabs: vec![tab],
            startup_picker_open: true,
            ..Self::default()
        };
        let size = iced::window::get_oldest()
            .and_then(iced::window::get_size)
            .map(Message::WindowResized);
        (app, size)
    }

    pub fn theme(&self) -> iced::Theme {
        self.theme_mode.iced_theme()
    }

    pub(crate) fn palette(&self) -> &'static Palette {
        self.theme_mode.palette()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard = iced::event::listen_with(|event, status, _| match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => Some(Message::KeyPressed {
                key,
                modifiers,
                captured: status == iced::event::Status::Captured,
            }),
            Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => Some(Message::KeyReleased(key)),
            _ => None,
        });

        let resizes = iced::window::resize_events().map(|(_, size)| Message::WindowResized(size));
        let close_requests = iced::window::close_requests().map(|_| Message::WindowCloseRequested);
        let mut subscriptions = vec![keyboard, resizes, close_requests];
        if self.run_playing() {
            subscriptions.push(
                iced::time::every(Duration::from_millis(RUN_TICK_MILLIS)).map(|_| Message::RunTick),
            );
        }
        if !self.toasts.is_empty() {
            let tick = if self.toasts_leaving() { TOAST_EXIT_TICK } else { TOAST_TICK };
            subscriptions.push(iced::time::every(tick).map(|_| Message::ToastTick));
        }
        Subscription::batch(subscriptions)
    }

    pub(crate) fn run_playing(&self) -> bool {
        self.get_active_tab().run.as_ref().is_some_and(RunSession::playing)
    }

    pub(crate) fn get_active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    pub(crate) fn get_active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    fn viewport(&self) -> Size {
        self.canvas_viewport
            .or_else(|| self.window_size.map(|size| Size::new(size.width, (size.height - CANVAS_CHROME_RESERVE).max(200.0))))
            .unwrap_or(DEFAULT_VIEWPORT)
    }

    fn reveal_active_tab(&self) -> Task<Message> {
        let x = if self.tabs.len() > 1 { self.active_tab as f32 / (self.tabs.len() - 1) as f32 } else { 0.0 };
        iced::widget::scrollable::snap_to(
            iced::widget::scrollable::Id::new(super::workspace::TAB_STRIP),
            iced::widget::scrollable::RelativeOffset { x, y: 0.0 },
        )
    }

    pub(crate) fn layout_width(&self) -> f32 {
        self.window_size.map(|size| size.width).unwrap_or(DEFAULT_WINDOW.width)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let may_change_content = message.may_change_content();
        let tabs_before = (self.active_tab, self.tabs.len());
        let mut task = self.dispatch(message);
        if self.tabs.is_empty() {
            return task;
        }
        if (self.active_tab, self.tabs.len()) != tabs_before {
            task = Task::batch([task, self.reveal_active_tab()]);
        }
        let tab = self.get_active_tab_mut();
        if may_change_content {
            tab.refresh_insight();
        }
        let highlight = tab.active_run_states();
        if highlight != tab.run_highlight {
            tab.run_highlight = highlight;
            tab.state_machine.request_full_redraw();
        }
        task
    }

    fn dispatch(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Canvas(canvas_message) => self.handle_canvas_message(canvas_message),
            Message::SelectTool(tool) => {
                self.get_active_tab_mut().set_active_tool(tool);
                Task::none()
            }
            Message::KeyPressed { key, modifiers, captured } => self.handle_key_pressed(key, modifiers, captured),
            Message::KeyReleased(key) => self.handle_key_released(key),
            Message::Clear => {
                self.get_active_tab_mut().clear();
                Task::none()
            }
            Message::EditTextChanged(text) => {
                self.get_active_tab_mut().edit_text = text;
                Task::none()
            }
            Message::FinishEditing => self.finish_editing(),
            Message::ToggleMenu(menu) => {
                self.open_menu = if self.open_menu == Some(menu) { None } else { Some(menu) };
                Task::none()
            }
            Message::CloseMenus => {
                self.open_menu = None;
                Task::none()
            }
            Message::DismissModal => self.dismiss_modal(),
            Message::CloseModal(modal) => {
                self.close_modal(modal);
                Task::none()
            }
            Message::Noop => Task::none(),
            Message::ChooseStartupModule(index) => self.choose_startup_module(index),
            Message::StartupHover(index) => {
                self.startup_selected = index;
                Task::none()
            }
            Message::StartupOpenFile => {
                let chosen = self.choose_startup_module(0);
                Task::batch([chosen, self.open_load_dialog()])
            }
            Message::ExitApp | Message::WindowCloseRequested => self.request_app_close(),
            Message::ConfirmCloseDiscard => self.confirm_close_discard(),
            Message::ConfirmCloseSave => self.confirm_close_save(),
            Message::ToggleTheme => self.toggle_theme(),
            Message::WindowResized(size) => {
                self.window_size = Some(size);
                Task::none()
            }
            Message::ToggleDock => {
                let tab = self.get_active_tab_mut();
                tab.dock_collapsed = !tab.dock_collapsed;
                Task::none()
            }
            Message::ToggleShortcuts => self.toggle_shortcuts(),
            Message::ToastTick => self.toast_tick(),
            Message::DismissToast(id) => self.dismiss_toast(id),
            Message::ZoomIn => self.zoom_by(1.2),
            Message::ZoomOut => self.zoom_by(1.0 / 1.2),
            Message::ZoomReset => self.zoom_reset(),
            Message::FitView => self.fit_view(),
            Message::OpenLoadDialog => {
                self.open_menu = None;
                self.open_load_dialog()
            }
            Message::LoadPathChanged(text) => {
                if let Some(dialog) = self.load_dialog.as_mut() {
                    dialog.path = text;
                }
                Task::none()
            }
            Message::LoadBrowseClicked => self.load_browse_clicked(),
            Message::LoadBrowseResult { picked } => self.load_browse_result(picked),
            Message::LoadPathSubmitted => self.load_path_submitted(),
            Message::OpenSaveDialog => self.open_save_dialog(),
            Message::SavePathChanged(text) => {
                if let Some(dialog) = self.save_dialog.as_mut() {
                    dialog.file.path = text;
                }
                Task::none()
            }
            Message::SaveBrowseClicked => self.save_browse_clicked(),
            Message::SaveBrowseResult { result } => self.save_browse_result(result),
            Message::SavePathSubmitted => self.save_path_submitted(),
            Message::CopyLlmPrompt => self.copy_llm_prompt(),
            Message::CheckInput => self.open_check_input(),
            Message::NfaToDfa => self.nfa_to_dfa(),
            Message::Minimize => self.minimize(),
            Message::CheckInputTextChanged(text) => {
                self.get_active_tab_mut().check_input = Some(text);
                Task::none()
            }
            Message::SubmitCheckInput => self.submit_check_input(),
            Message::OpenRegexDialog => self.open_regex_dialog(),
            Message::RegexTextChanged(text) => {
                self.get_active_tab_mut().regex_input = Some(text);
                Task::none()
            }
            Message::SubmitRegex => self.submit_regex(),
            Message::NewTab(family) => {
                self.push_tab(Tab::new_of(family));
                Task::none()
            }
            Message::RemoveTab(index) => self.request_tab_close(index),
            Message::SwitchTab(index) => self.switch_tab(index),
            Message::TabRenameChanged(text) => {
                self.tab_rename_text = text;
                Task::none()
            }
            Message::TabRenameSubmit => {
                self.commit_pending_rename();
                Task::none()
            }
            Message::RunInputChanged(text) => {
                self.get_active_tab_mut().run_input = text;
                Task::none()
            }
            Message::RunLoad => self.run_load(),
            Message::RunStep => self.run_step(),
            Message::RunReset => {
                self.get_active_tab_mut().run = None;
                Task::none()
            }
            Message::RunTogglePlay => self.run_toggle_play(),
            Message::RunTick => self.run_tick(),
            Message::GrammarEditorAction(action) => self.grammar_editor_action(action),
            Message::GrammarWordChanged(text) => {
                if let Some(editor) = self.get_active_tab_mut().grammar_mut() {
                    editor.word = text;
                }
                Task::none()
            }
            Message::GrammarParse => self.grammar_parse(),
            Message::GrammarCheckWord => self.grammar_check_word(),
            Message::GrammarDerive => self.grammar_derive(),
            Message::GrammarToCnf => self.grammar_to_cnf(),
            Message::OpenLatexExport => self.open_latex_export(),
            Message::CopyLatexExport => self.copy_export(self.latex_export.clone(), "LaTeX copied to the clipboard"),
            Message::OpenRegexExport => self.open_regex_export(),
            Message::CopyRegexExport => self.copy_export(self.regex_export.clone(), "Regular expression copied"),
            Message::EditTransitionLabelChanged(index, text) => {
                if let Some(input) = self.label_inputs_mut().and_then(|inputs| inputs.get_mut(index)) {
                    *input = text;
                }
                Task::none()
            }
            Message::SaveEditTransitionLabels => self.save_transition_labels(),
            Message::DeleteEditTransitionLabel(index) => {
                if let Some(inputs) = self.label_inputs_mut().filter(|inputs| index < inputs.len()) {
                    inputs.remove(index);
                }
                Task::none()
            }
            Message::AddEditTransitionLabel => {
                if let Some(inputs) = self.label_inputs_mut() {
                    inputs.push(String::new());
                }
                Task::none()
            }
        }
    }

    pub(crate) fn toggle_theme(&mut self) -> Task<Message> {
        self.theme_mode = self.theme_mode.toggled();
        Settings { theme: self.theme_mode }.save();
        for tab in &mut self.tabs {
            tab.state_machine.request_full_redraw();
        }
        Task::none()
    }

    pub(crate) fn toggle_shortcuts(&mut self) -> Task<Message> {
        self.open_menu = None;
        self.shortcuts_open = !self.shortcuts_open;
        Task::none()
    }

    pub(crate) fn zoom_by(&mut self, factor: f32) -> Task<Message> {
        let viewport = self.viewport();
        let center = iced::Point::new(viewport.width / 2.0, viewport.height / 2.0);
        let tab = self.get_active_tab_mut();
        tab.state_machine.zoom_around(factor, center, &tab.states, viewport);
        Task::none()
    }

    pub(crate) fn zoom_reset(&mut self) -> Task<Message> {
        let zoom = self.get_active_tab().state_machine.zoom();
        self.zoom_by(1.0 / zoom)
    }

    pub(crate) fn fit_view(&mut self) -> Task<Message> {
        let viewport = self.viewport();
        let tab = self.get_active_tab_mut();
        tab.state_machine.fit_to(&tab.states, viewport);
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let mut content = column![
            self.create_header(),
            horizontal_rule(1).style(theme::divider),
        ];

        if let Some(editor) = tab.grammar() {
            content = content.push(self.create_grammar_panel(editor));
        } else {
            content = content.push(self.create_workspace());
            content = content.push(self.create_run_dock());
        }
        content = content.push(self.create_status_bar());

        let mut layers: Vec<Element<'_, Message>> = vec![content.into()];

        if let Some(menu) = self.open_menu {
            layers.push(self.create_menu_layer(menu));
        }
        layers.extend(self.open_modals().map(|modal| self.modal_view(modal)));
        if !self.toasts.is_empty() {
            layers.push(self.create_toast_layer());
        }
        if self.startup_picker_open {
            layers.push(self.create_startup_picker());
        }

        container(stack(layers))
            .style(theme::app_background)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
