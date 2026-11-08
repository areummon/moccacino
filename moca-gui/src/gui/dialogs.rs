use iced::widget::{button, column, container, horizontal_space, mouse_area, row, scrollable, Space};
use iced::{Alignment, Element, Length};

use super::metrics::{DIALOG_CHROME_RESERVE, SHORTCUTS_TWO_COLUMN_MIN_WIDTH};
use super::workspace::CloseRequest;
use super::entity_loader::{PathDialog, LOAD_INPUT, SAVE_INPUT};
use super::feature_icon::feature_icon;
use super::icons::{icon, Icon, Ink};
use super::message::Message;
use super::message::Modal;
use super::shortcuts::{self, Action, Section, SHORTCUTS};
use super::widgets::{self, dialog_card, modal_layer, text, DIALOG_LG, DIALOG_MD, DIALOG_SM};
use crate::theme;
use crate::family::Family;
use crate::platform;

pub(crate) const EDIT_TEXT_INPUT: &str = "edit-text";
pub(crate) const EDIT_LABEL_INPUT: &str = "edit-label";
pub(crate) const CHECK_INPUT: &str = "check-input";
pub(crate) const REGEX_INPUT: &str = "regex-input";

const STARTUP_CARD_W: f32 = 250.0;
/// Tall enough for a two-line blurb, so every card in a row matches.
const STARTUP_CARD_H: f32 = 154.0;
const STARTUP_COLS: usize = 2;
const STARTUP_GAP: f32 = 14.0;
const STARTUP_PADDING: f32 = 28.0;

fn field<'a>(
    id: impl Into<String>,
    placeholder: &str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
    on_submit: Message,
) -> Element<'a, Message> {
    widgets::input(placeholder, value).id(id.into()).on_input(on_input).on_submit(on_submit)
        .into()
}

fn error_line<'a>(error: Option<&String>) -> Element<'a, Message> {
    match error {
        Some(error) => row![icon(Icon::Warn, 14.0, Ink::Danger), text(error.clone()).size(theme::TEXT_MD).style(theme::text_danger)]
            .spacing(6)
            .align_y(Alignment::Center)
            .into(),
        None => Space::with_height(18).into(),
    }
}

fn label_hint(family: Family) -> &'static str {
    match family {
        Family::Finite => "A symbol such as a — leave blank for ε",
        Family::Pushdown => "input;pop/push — e.g. a;Z/AZ  (ε = no-op)",
        Family::Turing => "read;write/dir — e.g. 0;1/R  (dir: L, R or S)",
        Family::Grammar => "",
    }
}

fn cancel(modal: Modal) -> Element<'static, Message> {
    widgets::ghost("Cancel", Some(Message::CloseModal(modal))).into()
}

fn modal(content: Element<'_, Message>) -> Element<'_, Message> {
    modal_layer(content, Message::DismissModal)
}

struct PathDialogSpec {
    modal: Modal,
    title: &'static str,
    subtitle: String,
    input_id: &'static str,
    on_input: fn(String) -> Message,
    on_submit: Message,
    on_browse: Message,
    action: &'static str,
}

fn path_dialog<'a>(dialog: &PathDialog, spec: PathDialogSpec) -> Element<'a, Message> {
    let body = column![
        row![
            field(spec.input_id, "/path/to/file.ce", &dialog.path, spec.on_input, spec.on_submit.clone()),
            widgets::text_button(Some((Icon::Folder, Ink::Text)), "Browse…", Some(spec.on_browse), theme::button_secondary),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        error_line(dialog.error.as_ref()),
    ]
    .spacing(8);
    modal(dialog_card(
        spec.title,
        Some(spec.subtitle),
        body.into(),
        vec![cancel(spec.modal), widgets::primary(None, spec.action, Some(spec.on_submit)).into()],
        DIALOG_MD,
    ))
}

impl super::app::App {
    pub(crate) fn create_load_dialog(&self) -> Element<'_, Message> {
        let Some(dialog) = &self.load_dialog else {
            return Space::new(0, 0).into();
        };
        path_dialog(
            dialog,
            PathDialogSpec {
                modal: Modal::Load,
                title: "Open computational entities",
                subtitle: "Every entity in the .ce file opens as its own tab.".to_string(),
                input_id: LOAD_INPUT,
                on_input: Message::LoadPathChanged,
                on_submit: Message::LoadPathSubmitted,
                on_browse: Message::LoadBrowseClicked,
                action: "Open",
            },
        )
    }

    pub(crate) fn create_save_dialog(&self) -> Element<'_, Message> {
        let Some(dialog) = &self.save_dialog else {
            return Space::new(0, 0).into();
        };
        path_dialog(
            &dialog.file,
            PathDialogSpec {
                modal: Modal::Save,
                title: "Save computational entity",
                subtitle: format!("Write “{}” to a .ce file.", self.get_active_tab().name),
                input_id: SAVE_INPUT,
                on_input: Message::SavePathChanged,
                on_submit: Message::SavePathSubmitted,
                on_browse: Message::SaveBrowseClicked,
                action: "Save",
            },
        )
    }

    pub(crate) fn create_startup_picker(&self) -> Element<'_, Message> {
        let p = self.palette();
        let card = |index: usize, family: Family| -> Element<'_, Message> {
            let selected = index == self.startup_selected;
            let card = button(
                column![
                    feature_icon(family, 46.0, selected),
                    text(family.title()).size(theme::TEXT_LG).font(theme::BOLD),
                    text(family.blurb()).size(theme::TEXT_MD).style(theme::text_dim),
                ]
                .spacing(8),
            )
            .on_press(Message::ChooseStartupModule(index))
            .width(Length::Fixed(STARTUP_CARD_W))
            .height(Length::Fixed(STARTUP_CARD_H))
            .padding(18)
            .style(theme::option_card(selected, theme::family_color(p, family)));
            mouse_area(card).on_enter(Message::StartupHover(index)).into()
        };
        let mut grid = column![].spacing(STARTUP_GAP);
        for (row_index, families) in Family::ALL.chunks(STARTUP_COLS).enumerate() {
            let mut cards = row![].spacing(STARTUP_GAP);
            for (column_index, family) in families.iter().enumerate() {
                cards = cards.push(card(row_index * STARTUP_COLS + column_index, *family));
            }
            grid = grid.push(cards);
        }

        let columns = STARTUP_COLS as f32;
        let dialog = container(
            column![
                row![
                    icon(Icon::Logo, 34.0, Ink::Text),
                    column![
                        text("Welcome to Moccacino").size(theme::TEXT_DISPLAY).font(theme::BOLD),
                        text("What would you like to explore today?").size(theme::TEXT_BASE).style(theme::text_dim),
                    ]
                    .spacing(2),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                grid,
                row![
                    widgets::text_button(
                        Some((Icon::Folder, Ink::Text)),
                        "Open a .ce file…",
                        Some(Message::StartupOpenFile),
                        theme::button_ghost,
                    ),
                    horizontal_space(),
                    widgets::kbd(
                        [-1, 1, -2, 2].map(|delta| shortcuts::hint(Action::StartupMove(delta))).join(" "),
                    ),
                    text("choose").size(theme::TEXT_SM).style(theme::text_faint),
                    widgets::kbd(shortcuts::hint(Action::StartupChoose)),
                    text("open").size(theme::TEXT_SM).style(theme::text_faint),
                ]
                .push_maybe((!platform::IS_WEB).then(|| widgets::ghost("Quit", Some(Message::ExitApp))))
                .spacing(6)
                .align_y(Alignment::Center),
            ]
            .spacing(22),
        )
        .padding(STARTUP_PADDING)
        .width(Length::Fixed(columns * STARTUP_CARD_W + (columns - 1.0) * STARTUP_GAP + 2.0 * STARTUP_PADDING))
        .style(theme::dialog);

        modal_layer(dialog.into(), Message::Noop)
    }

    pub(crate) fn create_check_input_dialog(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let subtitle = if tab.is_grammar() {
            "Membership is decided with CYK over the parsed grammar."
        } else {
            "Runs the whole machine at once; use the run panel to watch it step by step."
        };
        modal(dialog_card(
            "Check an input",
            Some(subtitle.to_string()),
            field(
                CHECK_INPUT,
                "Input word (blank = ε)",
                tab.check_input.as_deref().unwrap_or_default(),
                Message::CheckInputTextChanged,
                Message::SubmitCheckInput,
            ),
            vec![cancel(Modal::CheckInput), widgets::primary(None, "Check", Some(Message::SubmitCheckInput)).into()],
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
            text(error_message.clone()).size(theme::TEXT_BASE),
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        modal(dialog_card(
            "That didn't work",
            None,
            body.into(),
            vec![widgets::primary(None, "OK", Some(Message::CloseModal(Modal::Error))).into()],
            DIALOG_MD,
        ))
    }

    pub(crate) fn create_rename_dialog(&self) -> Element<'_, Message> {
        modal(dialog_card(
            "Rename state",
            None,
            field(EDIT_TEXT_INPUT, "Name", &self.get_active_tab().edit_text, Message::EditTextChanged, Message::FinishEditing),
            vec![cancel(Modal::RenameState), widgets::primary(None, "Save", Some(Message::FinishEditing)).into()],
            DIALOG_SM,
        ))
    }

    pub(crate) fn create_new_transition_dialog(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        modal(dialog_card(
            "New transition",
            Some(label_hint(tab.family()).to_string()),
            field(EDIT_TEXT_INPUT, "Label", &tab.edit_text, Message::EditTextChanged, Message::FinishEditing),
            vec![cancel(Modal::NewTransition), widgets::primary(None, "Add", Some(Message::FinishEditing)).into()],
            DIALOG_SM,
        ))
    }

    pub(crate) fn create_transition_labels_dialog(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let Some(edit) = &tab.label_edit else {
            return Space::new(0, 0).into();
        };
        let mut label_inputs = column![].spacing(8);
        for (i, label) in edit.inputs.iter().enumerate() {
            label_inputs = label_inputs.push(
                row![
                    widgets::input("Label", label)
                        .id(format!("{EDIT_LABEL_INPUT}-{i}"))
                        .on_input(move |text| Message::EditTransitionLabelChanged(i, text))
                        .on_submit(Message::SaveEditTransitionLabels)
                        .font(theme::MONO),
                    widgets::tip(
                        widgets::icon_button(Icon::Delete, Some(Message::DeleteEditTransitionLabel(i)), Ink::Danger)
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
            widgets::text_button(
                Some((Icon::Plus, Ink::Accent)),
                "Add label",
                Some(Message::AddEditTransitionLabel),
                theme::button_ghost,
            ),
        ]
        .spacing(8);
        modal(dialog_card(
            "Transition labels",
            Some(label_hint(tab.family()).to_string()),
            container(body).max_height(320.0).into(),
            vec![
                cancel(Modal::TransitionLabels),
                widgets::primary(None, "Save", Some(Message::SaveEditTransitionLabels)).into(),
            ],
            DIALOG_SM,
        ))
    }

    pub(crate) fn create_latex_export_dialog(&self) -> Element<'_, Message> {
        let code = self.latex_export.as_deref().unwrap_or("");
        let subtitle = if self.get_active_tab().is_grammar() {
            "The grammar's productions, ready for a LaTeX document."
        } else {
            "A TikZ drawing of this automaton, ready for a LaTeX document."
        };
        let body = container(
            scrollable(container(text(code.to_string()).font(theme::MONO).size(theme::TEXT_MD)).padding(14))
                .height(Length::Fixed(320.0))
                .style(theme::scroll),
        )
        .style(theme::sunken);
        modal(dialog_card(
            "LaTeX export",
            Some(subtitle.to_string()),
            body.into(),
            vec![
                widgets::ghost("Close", Some(Message::CloseModal(Modal::LatexExport))).into(),
                widgets::primary(Some(Icon::Check), "Copy", Some(Message::CopyLatexExport)).into(),
            ],
            DIALOG_LG,
        ))
    }

    pub(crate) fn create_regex_dialog(&self) -> Element<'_, Message> {
        modal(dialog_card(
            "Build from a regular expression",
            Some("Operators: | * + ?, grouping with ( ), \\c escapes, ε for empty.".to_string()),
            field(
                REGEX_INPUT,
                "e.g. (a|b)*abb",
                self.get_active_tab().regex_input.as_deref().unwrap_or_default(),
                Message::RegexTextChanged,
                Message::SubmitRegex,
            ),
            vec![cancel(Modal::BuildRegex), widgets::primary(None, "Build", Some(Message::SubmitRegex)).into()],
            DIALOG_MD,
        ))
    }

    pub(crate) fn create_regex_export_dialog(&self) -> Element<'_, Message> {
        let code = self.regex_export.as_deref().unwrap_or("");
        let body = widgets::input("", code).on_input(|_| Message::Noop).font(theme::MONO);
        modal(dialog_card(
            "Equivalent regular expression",
            Some("Obtained by state elimination.".to_string()),
            body.into(),
            vec![
                widgets::ghost("Close", Some(Message::CloseModal(Modal::RegexExport))).into(),
                widgets::primary(Some(Icon::Check), "Copy", Some(Message::CopyRegexExport)).into(),
            ],
            DIALOG_MD,
        ))
    }

    pub(crate) fn create_shortcuts_dialog(&self) -> Element<'_, Message> {
        let section = |section: Section| -> Element<'static, Message> {
            let mut list = column![widgets::caption(section.title())].spacing(8);
            for shortcut in SHORTCUTS.iter().filter(|shortcut| shortcut.section == section) {
                let mut caps = row![].spacing(4).align_y(Alignment::Center);
                let mut shown: Vec<Vec<String>> = shortcut.chords.iter().map(|chord| chord.caps()).collect();
                shown.dedup();
                for (index, chord) in shown.into_iter().enumerate() {
                    if index > 0 {
                        caps = caps.push(text("/").size(theme::TEXT_SM).style(theme::text_faint));
                    }
                    for cap in chord {
                        caps = caps.push(widgets::kbd(cap));
                    }
                }
                list = list.push(
                    row![
                        container(caps).width(Length::Fixed(128.0)),
                        text(shortcut.label).size(theme::TEXT_MD).style(theme::text_dim),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                );
            }
            list.width(Length::Fill).into()
        };
        let (canvas, view, app) = (section(Section::Canvas), section(Section::ViewAndRun), section(Section::App));
        let sections: Element<'_, Message> = if self.layout_width() >= SHORTCUTS_TWO_COLUMN_MIN_WIDTH {
            row![canvas, column![view, app].spacing(18).width(Length::Fill)].spacing(28).into()
        } else {
            column![canvas, view, app].spacing(18).into()
        };
        let max_height = self.window_size.map(|size| size.height - DIALOG_CHROME_RESERVE).unwrap_or(560.0).max(200.0);
        let body = container(scrollable(sections).style(theme::scroll)).max_height(max_height);
        modal(dialog_card(
            "Keyboard shortcuts",
            Some("Double-click a tab to rename it.".to_string()),
            body.into(),
            vec![widgets::primary(None, "Done", Some(Message::ToggleShortcuts)).into()],
            DIALOG_LG + 80.0,
        ))
    }

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
                    widgets::family_dot(tab.family(), 8.0),
                    text(tab.name.clone()).size(theme::TEXT_BASE).font(theme::SEMIBOLD),
                    text(note).size(theme::TEXT_SM).style(theme::text_faint),
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
        modal(dialog_card(
            title,
            Some(subtitle),
            body.into(),
            vec![
                cancel(Modal::Close),
                widgets::secondary("Save…", Some(Message::ConfirmCloseSave)).into(),
                widgets::text_button(None, discard_label, Some(Message::ConfirmCloseDiscard), theme::button_danger)
                    .into(),
            ],
            DIALOG_MD,
        ))
    }
}
