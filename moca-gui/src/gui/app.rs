use iced::keyboard;
use iced::widget::{column, container, horizontal_rule, stack};
use iced::{Element, Event, Length, Size, Subscription, Task};

use std::time::{Duration, Instant};

use super::message::{Menu, Message};
use super::settings::Settings;
use super::tab::Tab;
use super::toast::{Toast, TOAST_TICK};
use crate::gui::theme::{self, Palette, ThemeMode};

/* Interval between auto-play ticks of the machine run panels. */
const RUN_TICK_MILLIS: u64 = 250;
/* Fallback viewport until the canvas reports its real size. */
const DEFAULT_VIEWPORT: Size = Size::new(1100.0, 560.0);

#[derive(Default)]
pub struct App {
    pub(crate) tabs: Vec<Box<Tab>>,
    pub(crate) active_tab: usize,
    pub(crate) theme_mode: ThemeMode,
    pub(crate) startup_picker_open: bool,
    pub(crate) startup_selected: usize,
    pub(crate) open_menu: Option<Menu>,
    pub(crate) shortcuts_open: bool,
    pub(crate) load_dialog_open: bool,
    pub(crate) load_path_text: String,
    pub(crate) load_dialog_error: Option<String>,
    pub(crate) save_dialog_open: bool,
    pub(crate) save_path_text: String,
    pub(crate) save_dialog_error: Option<String>,
    // Serialized entity of the active tab, stashed while the save dialog
    // is up so both the Browse and typed-path flows write the same bytes.
    pub(crate) pending_save: Option<String>,
    // Which tab the stashed save belongs to and its content fingerprint at
    // that moment; applied to the tab once the file is written.
    pub(crate) pending_save_fingerprint: Option<(usize, u64)>,
    pub(crate) error_message: Option<String>,
    pub(crate) latex_export_dialog_open: bool,
    pub(crate) latex_export_code: Option<String>,
    pub(crate) regex_export_dialog_open: bool,
    pub(crate) regex_export_code: Option<String>,
    pub(crate) toasts: Vec<Toast>,
    pub(crate) next_toast_id: u64,
    // Size of the canvas on screen, as last reported by the canvas.
    pub(crate) canvas_viewport: Option<Size>,
    // Tab rename: which tab is being renamed and the draft name.
    pub(crate) renaming_tab: Option<usize>,
    pub(crate) tab_rename_text: String,
    pub(crate) last_tab_click: Option<(usize, Instant)>,
    // Logical window size, for the responsive header, dock and status bar.
    pub(crate) window_size: Option<Size>,
    // A close waiting for the user to confirm discarding unsaved changes.
    pub(crate) pending_close: Option<CloseRequest>,
}

/* What the unsaved-changes dialog is about to close. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CloseRequest {
    App,
    Tab(usize),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let mut app = Self::default();
        app.theme_mode = Settings::load().theme;
        app.tabs.push(Box::new(Tab::new()));
        app.tabs[0].refresh_insight();
        // The startup picker blocks the GUI until a module is chosen.
        app.startup_picker_open = true;
        // Learn the real window size right away; resize events follow.
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

        // Timers only exist while they have work: auto-play ticks while a
        // panel is playing (pauses, halts and tab switches stop the clock
        // by themselves), toast ticks while a toast is alive.
        let resizes = iced::window::resize_events().map(|(_, size)| Message::WindowResized(size));
        let close_requests = iced::window::close_requests().map(|_| Message::WindowCloseRequested);
        let mut subscriptions = vec![keyboard, resizes, close_requests];
        if self.run_tick_interval().is_some() {
            subscriptions.push(
                iced::time::every(Duration::from_millis(RUN_TICK_MILLIS)).map(|_| Message::RunTick),
            );
        }
        if !self.toasts.is_empty() {
            subscriptions.push(iced::time::every(TOAST_TICK).map(|_| Message::ToastTick));
        }
        Subscription::batch(subscriptions)
    }

    /* Milliseconds between auto-play ticks, when the active tab has a
     * running panel that should keep stepping; None otherwise. */
    pub(crate) fn run_tick_interval(&self) -> Option<Duration> {
        let (loaded, finished, playing) = self.get_active_tab().run_state();
        (playing && loaded && !finished).then(|| Duration::from_millis(RUN_TICK_MILLIS))
    }

    pub(crate) fn get_active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    pub(crate) fn get_active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    /* Canvas size on screen: the canvas's own report, else an estimate
     * from the window (header, dock and status bar take about 260px). */
    pub(crate) fn viewport(&self) -> Size {
        self.canvas_viewport
            .or_else(|| self.window_size.map(|size| Size::new(size.width, (size.height - 260.0).max(200.0))))
            .unwrap_or(DEFAULT_VIEWPORT)
    }

    /* Scrolls the tab strip so the active tab is in view. */
    pub(crate) fn reveal_active_tab(&self) -> Task<Message> {
        let x = if self.tabs.len() > 1 { self.active_tab as f32 / (self.tabs.len() - 1) as f32 } else { 0.0 };
        iced::widget::scrollable::snap_to(
            iced::widget::scrollable::Id::new(super::toolbars::TAB_STRIP),
            iced::widget::scrollable::RelativeOffset { x, y: 0.0 },
        )
    }

    /* Logical window width driving the responsive breakpoints. */
    pub(crate) fn layout_width(&self) -> f32 {
        self.window_size.map(|size| size.width).unwrap_or(1280.0)
    }

    /* True while any modal dialog is up on the active tab or globally, or
     * while the startup picker forces a choice. Used to block canvas input
     * and canvas shortcuts behind popups. */
    pub(crate) fn any_modal_open(&self) -> bool {
        self.startup_picker_open
            || self.load_dialog_open
            || self.save_dialog_open
            || self.shortcuts_open
            || self.pending_close.is_some()
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
                    || tab.regex_dialog_open
            }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        // High-frequency messages never change the machine; everything
        // else refreshes the status-bar insight of the (new) active tab.
        let structural = !matches!(
            message,
            Message::RunTick
                | Message::ToastTick
                | Message::KeyReleased(_)
                | Message::KeyPressed {
                    key: iced::keyboard::Key::Named(
                        iced::keyboard::key::Named::Control
                            | iced::keyboard::key::Named::Shift
                            | iced::keyboard::key::Named::Alt,
                    ),
                    ..
                }
                | Message::Noop
                | Message::WindowResized(_)
                | Message::Canvas(
                    crate::state_machine::CanvasMessage::MoveState { .. }
                        | crate::state_machine::CanvasMessage::Scrolled(_)
                        | crate::state_machine::CanvasMessage::Zoomed { .. }
                        | crate::state_machine::CanvasMessage::Viewport(_)
                )
        );
        let tabs_before = (self.active_tab, self.tabs.len());
        let mut task = self.dispatch(message);
        if self.tabs.is_empty() {
            return task;
        }
        if (self.active_tab, self.tabs.len()) != tabs_before {
            task = Task::batch([task, self.reveal_active_tab()]);
        }
        let tab = self.get_active_tab_mut();
        if structural {
            tab.refresh_insight();
        }
        // Steps, resets and loads move the run's current states; repaint
        // the canvas glow only when they actually changed.
        let highlight = tab.active_run_states();
        if highlight != tab.run_highlight {
            tab.run_highlight = highlight;
            tab.state_machine.request_redraw();
        }
        task
    }

    fn dispatch(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Canvas(canvas_message) => self.handle_canvas_message(canvas_message),
            Message::SelectTool(tool) => self.select_tool(tool),
            Message::KeyPressed { key, modifiers, captured } => self.handle_key_pressed(key, modifiers, captured),
            Message::KeyReleased(key) => self.handle_key_released(key),
            Message::Clear => self.clear_active_tab(),
            Message::EditTextChanged(text) => self.edit_text_changed(text),
            Message::FinishEditing => self.finish_editing(),
            Message::CancelEditing => self.cancel_editing(),
            Message::ToggleMenu(menu) => self.toggle_menu(menu),
            Message::CloseMenus => self.close_menus(),
            Message::DismissModal => self.dismiss_modal(),
            Message::Noop => Task::none(),
            Message::ChooseStartupModule(index) => self.choose_startup_module(index),
            Message::StartupOpenFile => {
                let chosen = self.choose_startup_module(0);
                Task::batch([chosen, self.open_load_dialog()])
            }
            Message::ExitApp | Message::WindowCloseRequested => self.request_app_close(),
            Message::ConfirmCloseDiscard => self.confirm_close_discard(),
            Message::ConfirmCloseSave => self.confirm_close_save(),
            Message::CancelClose => {
                self.pending_close = None;
                Task::none()
            }
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
            Message::ToggleShortcuts => {
                self.open_menu = None;
                self.shortcuts_open = !self.shortcuts_open;
                Task::none()
            }
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
            Message::LoadPathChanged(text) => self.load_path_changed(text),
            Message::LoadBrowseClicked => self.load_browse_clicked(),
            Message::LoadBrowseResult { result } => self.load_browse_result(result),
            Message::LoadPathSubmitted => self.load_path_submitted(),
            Message::CancelLoadDialog => self.cancel_load_dialog(),
            Message::OpenSaveDialog => {
                self.open_menu = None;
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
            Message::OpenRegexDialog => self.open_regex_dialog(),
            Message::RegexTextChanged(text) => self.regex_text_changed(text),
            Message::SubmitRegex => self.submit_regex(),
            Message::CancelRegex => self.cancel_regex(),
            Message::NewTab(family) => self.new_tab(family),
            Message::RemoveTab(index) => self.request_tab_close(index),
            Message::SwitchTab(index) => self.switch_tab(index),
            Message::TabRenameChanged(text) => {
                self.tab_rename_text = text;
                Task::none()
            }
            Message::TabRenameSubmit => self.commit_tab_rename(),
            Message::TmInputChanged(text) => self.tm_input_changed(text),
            Message::TmLoadInput => self.tm_load_input(),
            Message::TmStep => self.tm_step(),
            Message::TmReset => self.tm_reset(),
            Message::TmTogglePlay => self.tm_toggle_play(),
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

    /* Flips light/dark, remembers the choice, and drops every canvas
     * cache so drawings repaint in the new palette. */
    fn toggle_theme(&mut self) -> Task<Message> {
        self.theme_mode = self.theme_mode.toggled();
        Settings { theme: self.theme_mode }.save();
        for tab in &mut self.tabs {
            tab.state_machine.request_full_redraw();
        }
        Task::none()
    }

    fn zoom_by(&mut self, factor: f32) -> Task<Message> {
        let viewport = self.viewport();
        let center = iced::Point::new(viewport.width / 2.0, viewport.height / 2.0);
        let tab = self.get_active_tab_mut();
        let states = std::mem::take(&mut tab.states);
        tab.state_machine.zoom_around(factor, center, &states, viewport);
        tab.states = states;
        Task::none()
    }

    fn zoom_reset(&mut self) -> Task<Message> {
        let zoom = self.get_active_tab().state_machine.zoom();
        self.zoom_by(1.0 / zoom)
    }

    pub(crate) fn fit_view(&mut self) -> Task<Message> {
        let viewport = self.viewport();
        let tab = self.get_active_tab_mut();
        let states = std::mem::take(&mut tab.states);
        tab.state_machine.fit_to(&states, viewport);
        tab.states = states;
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let mut content = column![
            self.create_header(),
            horizontal_rule(1).style(theme::divider),
        ];

        if tab.machine.is_grammar() {
            content = content.push(self.create_grammar_panel());
        } else {
            content = content.push(self.create_workspace());
            content = content.push(self.create_run_dock());
        }
        content = content.push(self.create_status_bar());

        let mut layers: Vec<Element<'_, Message>> = vec![content.into()];

        // Dropdown menus float below the header; the click-away layer
        // leaves the header itself interactive so another trigger can be
        // clicked directly.
        if let Some(menu) = self.open_menu {
            layers.push(self.create_menu_layer(menu));
        }
        layers.extend(self.overlays());
        if !self.toasts.is_empty() {
            layers.push(self.create_toast_layer());
        }
        // The startup picker sits above everything and cannot be dismissed.
        if self.startup_picker_open {
            layers.push(self.create_startup_picker());
        }

        container(stack(layers))
            .style(theme::app_background)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /* Modal dialogs in stacking order (later entries sit on top); the
     * order mirrors `dismiss_modal`. */
    fn overlays(&self) -> Vec<Element<'_, Message>> {
        let tab = self.get_active_tab();
        let mut layers = Vec::new();
        if tab.editing_state.is_some() || tab.editing_transition.is_some() {
            layers.push(self.create_edit_dialog());
        }
        if tab.check_input_dialog_open {
            layers.push(self.create_check_input_dialog());
        }
        if tab.regex_dialog_open {
            layers.push(self.create_regex_dialog());
        }
        // The .ce dialogs sit above the app but below the error popup so
        // load failures remain visible.
        if self.load_dialog_open {
            layers.push(self.create_load_dialog());
        }
        if self.save_dialog_open {
            layers.push(self.create_save_dialog());
        }
        if self.latex_export_dialog_open {
            layers.push(self.create_latex_export_dialog());
        }
        if self.regex_export_dialog_open {
            layers.push(self.create_regex_export_dialog());
        }
        if tab.pending_transition_dialog_open || tab.editing_transition_dialog_open {
            layers.push(self.create_edit_dialog());
        }
        if self.shortcuts_open {
            layers.push(self.create_shortcuts_dialog());
        }
        if self.pending_close.is_some() {
            layers.push(self.create_close_dialog());
        }
        if self.error_message.is_some() {
            layers.push(self.create_error_popup());
        }
        layers
    }
}
