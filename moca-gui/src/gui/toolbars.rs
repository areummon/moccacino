
use iced::widget::{
    button, column, container, horizontal_space, mouse_area, row, scrollable, stack, text_input, tooltip,
    vertical_rule, Space,
};
use iced::widget::text::Wrapping;
use iced::{Alignment, Element, Length, Padding};

use super::icons::{icon, Icon, Ink};
use super::message::{Menu, Message};
use super::tab::TabMachine;
use super::widgets::{self, text, tip};
use super::workspace::TAB_RENAME_INPUT;
use crate::gui::theme::{self, Family, ThemeMode, Tone};
use crate::state_machine::{CanvasContext, EditorTool};

const HEADER_HEIGHT: f32 = 52.0;
const TAB_ROW_HEIGHT: f32 = 44.0;
const HEADER_PADDING_X: f32 = 12.0;
const BRAND_WIDTH: f32 = 128.0;
const BRAND_WIDTH_COMPACT: f32 = 34.0;
const BRAND_TEXT_MIN_WIDTH: f32 = 1000.0;
const STACKED_HEADER_MAX_WIDTH: f32 = 800.0;
const MENU_GAP: f32 = 2.0;
const MENUS: [(Menu, &str, f32); 4] = [
    (Menu::New, "New", 70.0),
    (Menu::Operations, "Operations", 112.0),
    (Menu::File, "File", 60.0),
    (Menu::Export, "Export", 78.0),
];
const ICON_BUTTON: f32 = 32.0;
const RIGHT_CLUSTER_SPACING: f32 = 4.0;
const DROPDOWN_WIDTH: f32 = 272.0;
pub(crate) const TAB_STRIP: &str = "tab-strip";
const DOCK_COMPACT_MAX_WIDTH: f32 = 1150.0;
const PALETTE_ICONS_ONLY_MAX_WIDTH: f32 = 760.0;
const STATUS_HINT_MIN_WIDTH: f32 = 1000.0;
const STATUS_COUNTS_MIN_WIDTH: f32 = 700.0;

impl super::app::App {
    fn header_layout(&self) -> (f32, bool) {
        let width = self.layout_width();
        let brand = if width >= BRAND_TEXT_MIN_WIDTH { BRAND_WIDTH } else { BRAND_WIDTH_COMPACT };
        (brand, width < STACKED_HEADER_MAX_WIDTH)
    }

    pub(crate) fn create_header(&self) -> Element<'_, Message> {
        let (brand_width, stacked) = self.header_layout();
        let mut brand_row = row![icon(Icon::Logo, 24.0, Ink::Text)].spacing(8).align_y(Alignment::Center);
        if brand_width == BRAND_WIDTH {
            brand_row = brand_row.push(text("moccacino").size(17).font(theme::BOLD));
        }
        let brand = container(brand_row).width(Length::Fixed(brand_width));

        let mut menus = row![].spacing(MENU_GAP).align_y(Alignment::Center);
        for (menu, label, width) in MENUS {
            let open = self.open_menu == Some(menu);
            menus = menus.push(
                button(
                    row![
                        text(label).size(14).font(theme::SEMIBOLD),
                        icon(Icon::ChevronDown, 12.0, if open { Ink::Accent } else { Ink::Faint }),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center),
                )
                .on_press(Message::ToggleMenu(menu))
                .width(Length::Fixed(width))
                .padding([7, 10])
                .style(theme::menu_trigger(open)),
            );
        }

        let new_tab_open = self.open_menu == Some(Menu::NewFromTabs);
        let plus = tip(
            button(icon(Icon::Plus, 16.0, if new_tab_open { Ink::Accent } else { Ink::Dim }))
                .on_press(Message::ToggleMenu(Menu::NewFromTabs))
                .width(Length::Fixed(ICON_BUTTON))
                .padding(8)
                .style(theme::menu_trigger(new_tab_open)),
            "New tab  (Ctrl+T)",
            tooltip::Position::Bottom,
        );
        let help = tip(
            button(icon(Icon::Keyboard, 16.0, Ink::Dim))
                .on_press(Message::ToggleShortcuts)
                .width(Length::Fixed(ICON_BUTTON))
                .padding(8)
                .style(theme::button_ghost),
            "Keyboard shortcuts  (F1)",
            tooltip::Position::Bottom,
        );
        let (theme_icon, theme_tip) = match self.theme_mode {
            ThemeMode::Light => (Icon::Moon, "Switch to the dark theme  (Ctrl+Shift+L)"),
            ThemeMode::Dark => (Icon::Sun, "Switch to the light theme  (Ctrl+Shift+L)"),
        };
        let theme_toggle = tip(
            button(icon(theme_icon, 16.0, Ink::Dim))
                .on_press(Message::ToggleTheme)
                .width(Length::Fixed(ICON_BUTTON))
                .padding(8)
                .style(theme::button_ghost),
            theme_tip,
            tooltip::Position::Bottom,
        );

        let mut top = row![brand, menus].spacing(RIGHT_CLUSTER_SPACING).align_y(Alignment::Center);
        if stacked {
            top = top.push(horizontal_space());
        } else {
            top = top
                .push(container(vertical_rule(1).style(theme::divider)).height(Length::Fixed(22.0)).padding([0, 8]))
                .push(self.create_tab_strip());
        }
        top = top.push(plus).push(Space::with_width(8)).push(help).push(theme_toggle);

        let top = container(top)
            .padding([0.0, HEADER_PADDING_X])
            .height(Length::Fixed(HEADER_HEIGHT))
            .center_y(Length::Fixed(HEADER_HEIGHT))
            .width(Length::Fill);
        let header: Element<'_, Message> = if stacked {
            column![
                top,
                container(self.create_tab_strip())
                    .padding([0.0, HEADER_PADDING_X])
                    .height(Length::Fixed(TAB_ROW_HEIGHT))
                    .center_y(Length::Fixed(TAB_ROW_HEIGHT)),
            ]
            .into()
        } else {
            top.into()
        };
        container(header).width(Length::Fill).style(theme::header).into()
    }

    fn create_tab_strip(&self) -> Element<'_, Message> {
        let closable = self.tabs.len() > 1;
        let mut strip = row![].spacing(4).align_y(Alignment::Center);
        for (index, tab) in self.tabs.iter().enumerate() {
            let active = index == self.active_tab;
            let family = tab.machine.family();

            let name: Element<'_, Message> = if self.renaming_tab == Some(index) {
                text_input("Tab name", &self.tab_rename_text)
                    .id(TAB_RENAME_INPUT)
                    .on_input(Message::TabRenameChanged)
                    .on_submit(Message::TabRenameSubmit)
                    .size(13)
                    .padding([2, 6])
                    .width(Length::Fixed(140.0))
                    .style(theme::input)
                    .into()
            } else {
                text(tab.name.clone())
                    .size(13)
                    .font(if active { theme::SEMIBOLD } else { theme::UI })
                    .into()
            };

            let mut content = row![widgets::family_dot(family, 8.0), name]
                .spacing(7)
                .align_y(Alignment::Center);
            if tab.insight.unsaved {
                content = content.push(
                    container(Space::new(Length::Fixed(6.0), Length::Fixed(6.0))).style(theme::unsaved_dot),
                );
            }
            if closable {
                content = content.push(
                    button(icon(Icon::Close, 11.0, if active { Ink::Dim } else { Ink::Faint }))
                        .on_press(Message::RemoveTab(index))
                        .padding(3)
                        .style(theme::tab_close),
                );
            }

            let pill = button(content)
                .on_press(Message::SwitchTab(index))
                .padding(Padding { top: 6.0, bottom: 6.0, left: 12.0, right: if closable { 6.0 } else { 12.0 } })
                .style(theme::tab_pill(active));
            strip = strip.push(pill);
        }

        scrollable(container(strip).padding([4, 2]))
            .id(scrollable::Id::new(TAB_STRIP))
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::new().width(3).scroller_width(3).margin(0),
            ))
            .width(Length::Fill)
            .style(theme::scroll_subtle)
            .into()
    }

    pub(crate) fn create_menu_layer(&self, menu: Menu) -> Element<'_, Message> {
        let (brand_width, stacked) = self.header_layout();
        let panel = container(self.create_menu(menu))
            .padding(6)
            .width(Length::Fixed(DROPDOWN_WIDTH))
            .style(theme::menu_panel);

        let placed: Element<'_, Message> = match menu {
            Menu::NewFromTabs => {
                let right = HEADER_PADDING_X + 2.0 * ICON_BUTTON + 8.0 + 3.0 * RIGHT_CLUSTER_SPACING;
                container(panel)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right)
                    .padding(Padding { top: 4.0, right, bottom: 0.0, left: 0.0 })
                    .into()
            }
            _ => {
                let mut left = HEADER_PADDING_X + brand_width + RIGHT_CLUSTER_SPACING;
                for (other, _, width) in MENUS {
                    if other == menu {
                        break;
                    }
                    left += width + MENU_GAP;
                }
                let left = left.min((self.layout_width() - DROPDOWN_WIDTH - 8.0).max(0.0));
                container(panel)
                    .width(Length::Fill)
                    .padding(Padding { top: 4.0, right: 0.0, bottom: 0.0, left })
                    .into()
            }
        };

        column![
            Space::with_height(Length::Fixed(if stacked { HEADER_HEIGHT + TAB_ROW_HEIGHT } else { HEADER_HEIGHT })),
            mouse_area(container(placed).width(Length::Fill).height(Length::Fill)).on_press(Message::CloseMenus),
        ]
        .into()
    }

    fn create_menu(&self, menu: Menu) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let is_finite = matches!(tab.machine, TabMachine::Finite(_));
        let has_states = !tab.states.is_empty();
        match menu {
            Menu::New | Menu::NewFromTabs => {
                let mut items = column![].spacing(2);
                for family in Family::ALL {
                    items = items.push(
                        button(
                            row![
                                container(icon(Icon::Family(family), 18.0, Ink::Text))
                                    .padding(6)
                                    .style(theme::family_badge(family)),
                                column![
                                    text(family.title()).size(14).font(theme::SEMIBOLD),
                                    text(family.blurb()).size(12).style(theme::text_faint),
                                ]
                                .spacing(1),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                        )
                        .on_press(Message::NewTab(family))
                        .width(Length::Fill)
                        .padding([6, 8])
                        .style(theme::menu_item),
                    );
                }
                items
                    .push(menu_separator())
                    .push(menu_item(Icon::Folder, "Open .ce file…", Some("Ctrl+O"), Some(Message::OpenLoadDialog), None))
                    .into()
            }
            Menu::Operations => {
                let nfa_to_dfa_reason = if !is_finite {
                    Some("Finite automata only")
                } else if tab.insight.deterministic {
                    Some("Already deterministic")
                } else {
                    None
                };
                let minimize_reason = if !is_finite {
                    Some("Finite automata only")
                } else if !tab.insight.deterministic {
                    Some("Needs a DFA — convert first")
                } else if !has_states {
                    Some("The canvas is empty")
                } else {
                    None
                };
                column![
                    menu_item(Icon::Check, "Check input…", None, Some(Message::CheckInput), None),
                    menu_separator(),
                    menu_item(
                        Icon::Family(Family::Finite),
                        "NFA → DFA",
                        None,
                        nfa_to_dfa_reason.is_none().then_some(Message::DfaToNfa),
                        nfa_to_dfa_reason,
                    ),
                    menu_item(
                        Icon::Fit,
                        "Minimize DFA",
                        None,
                        minimize_reason.is_none().then_some(Message::Minimize),
                        minimize_reason,
                    ),
                    menu_separator(),
                    menu_item(Icon::Plus, "Build from regex…", None, Some(Message::OpenRegexDialog), None),
                ]
                .spacing(2)
                .into()
            }
            Menu::File => column![
                menu_item(Icon::Folder, "Open .ce file…", Some("Ctrl+O"), Some(Message::OpenLoadDialog), None),
                menu_item(Icon::Check, "Save tab as .ce…", Some("Ctrl+S"), Some(Message::OpenSaveDialog), None),
            ]
            .spacing(2)
            .into(),
            Menu::Export => {
                let regex_reason = (!is_finite).then_some("Finite automata only");
                column![
                    menu_item(Icon::Transition, "LaTeX / TikZ…", None, Some(Message::OpenLatexExport), None),
                    menu_item(
                        Icon::Family(Family::Grammar),
                        "Regular expression…",
                        None,
                        regex_reason.is_none().then_some(Message::OpenRegexExport),
                        regex_reason,
                    ),
                    menu_separator(),
                    menu_item(
                        Icon::Keyboard,
                        "Copy LLM prompt",
                        None,
                        Some(Message::CopyLlmPrompt),
                        Some("Turns a photo of a diagram into a .ce file"),
                    ),
                ]
                .spacing(2)
                .into()
            }
        }
    }

    pub(crate) fn create_workspace(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let active_states = &tab.run_highlight;
        let canvas = tab
            .state_machine
            .view(
                &tab.states,
                &tab.transitions,
                tab.initial_state,
                &tab.final_states,
                CanvasContext {
                    active_states,
                    family: tab.machine.family(),
                    interactive: !self.any_modal_open() && self.open_menu.is_none(),
                    known_viewport: self.canvas_viewport.unwrap_or(iced::Size::ZERO),
                },
            )
            .map(Message::Canvas);

        let mut layers: Vec<Element<'_, Message>> = vec![canvas];

        if tab.states.is_empty() {
            layers.push(
                container(
                    column![
                        icon(Icon::State, 40.0, Ink::Faint),
                        text("An empty canvas").size(17).font(theme::SEMIBOLD).style(theme::text_dim),
                        row![
                            text("Press").size(13).style(theme::text_faint),
                            widgets::kbd("2"),
                            text("and click to place states, then").size(13).style(theme::text_faint),
                            widgets::kbd("3"),
                            text("to connect them").size(13).style(theme::text_faint),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    ]
                    .spacing(10)
                    .align_x(Alignment::Center),
                )
                .center(Length::Fill)
                .into(),
            );
        }

        layers.push(
            container(mouse_area(self.create_tool_palette()).on_press(Message::Noop))
                .width(Length::Fill)
                .padding([14, 0])
                .align_x(iced::alignment::Horizontal::Center)
                .into(),
        );

        if !tab.states.is_empty() || !tab.transitions.is_empty() {
            layers.push(
                container(tip(
                    button(widgets::labeled(Some((Icon::Delete, Ink::Danger)), "Clear"))
                        .on_press(Message::Clear)
                        .padding([6, 12])
                        .style(theme::button_danger_ghost),
                    "Remove every state and transition",
                    tooltip::Position::Left,
                ))
                .width(Length::Fill)
                .padding(14)
                .align_x(iced::alignment::Horizontal::Right)
                .into(),
            );
        }

        layers.push(
            container(mouse_area(self.create_zoom_chip()).on_press(Message::Noop))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(14)
                .align_x(iced::alignment::Horizontal::Right)
                .align_y(iced::alignment::Vertical::Bottom)
                .into(),
        );

        stack(layers).width(Length::Fill).height(Length::Fill).into()
    }

    fn create_tool_palette(&self) -> Element<'_, Message> {
        let active = self.get_active_tab().active_tool();
        let captions = self.layout_width() >= PALETTE_ICONS_ONLY_MAX_WIDTH;
        let mut tools = row![].spacing(2).align_y(Alignment::Center);
        for tool in EditorTool::ALL {
            let is_active = tool == active;
            let ink = match (is_active, tool) {
                (true, _) => Ink::Accent,
                (false, _) => Ink::Dim,
            };
            let mut label = row![icon(tool.icon(), 16.0, ink)].spacing(6).align_y(Alignment::Center);
            if captions {
                label = label.push(text(tool.name()).size(13).font(theme::SEMIBOLD).wrapping(Wrapping::None));
            }
            tools = tools.push(tip(
                button(label)
                .on_press(Message::SelectTool(tool))
                .padding([7, 12])
                .style(theme::toggle_pill(is_active)),
                format!("{}  ({})\n{}", tool.name(), tool.shortcut(), tool.tooltip()),
                tooltip::Position::Bottom,
            ));
        }
        container(tools).padding(4).style(theme::floating).into()
    }

    fn create_zoom_chip(&self) -> Element<'_, Message> {
        let zoom = self.get_active_tab().state_machine.zoom();
        let chip = row![
            tip(widgets::icon_button(Icon::Minus, Some(Message::ZoomOut), Ink::Dim), "Zoom out  (Ctrl+−)", tooltip::Position::Top),
            tip(
                button(text(format!("{:>3.0}%", zoom * 100.0)).size(12).font(theme::MONO))
                    .on_press(Message::ZoomReset)
                    .padding([6, 4])
                    .style(theme::button_ghost),
                "Reset to 100%  (Ctrl+0)",
                tooltip::Position::Top,
            ),
            tip(widgets::icon_button(Icon::Plus, Some(Message::ZoomIn), Ink::Dim), "Zoom in  (Ctrl++)", tooltip::Position::Top),
            tip(widgets::icon_button(Icon::Fit, Some(Message::FitView), Ink::Dim), "Fit to view  (F)", tooltip::Position::Top),
        ]
        .spacing(0)
        .align_y(Alignment::Center);
        container(chip).padding([2, 6]).style(theme::floating).into()
    }

    pub(crate) fn dock_is_compact(&self) -> bool {
        self.layout_width() < DOCK_COMPACT_MAX_WIDTH
    }

    pub(crate) fn create_run_dock(&self) -> Element<'_, Message> {
        let dock = match self.get_active_tab().machine {
            TabMachine::Finite(_) => self.create_finite_panel(),
            TabMachine::Pushdown(_) => self.create_pda_panel(),
            TabMachine::Turing(_) => self.create_tm_panel(),
            TabMachine::Grammar(_) => return Space::new(0, 0).into(),
        };
        container(dock).padding(Padding { top: 0.0, right: 12.0, bottom: 8.0, left: 12.0 }).into()
    }

    pub(crate) fn create_status_bar(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let insight = &tab.insight;
        let mut left = row![].spacing(10).align_y(Alignment::Center);
        let mut right = row![].spacing(10).align_y(Alignment::Center);

        if tab.machine.is_grammar() {
            left = left
                .push(widgets::family_dot(Family::Grammar, 7.0))
                .push(text("Grammar editor").size(12))
                .push(text(format!("{} productions", insight.production_count)).size(12).style(theme::text_faint));
        } else {
            let tool = tab.active_tool();
            left = left
                .push(icon(tool.icon(), 13.0, Ink::Dim))
                .push(text(tool.name()).size(12).font(theme::SEMIBOLD).wrapping(Wrapping::None));
            if self.layout_width() >= STATUS_HINT_MIN_WIDTH {
                left = left.push(text(tool.hint()).size(12).style(theme::text_faint).wrapping(Wrapping::None));
            }
            let transition_count: usize = tab.transitions.values().map(|labels| labels.len()).sum();
            if self.layout_width() >= STATUS_COUNTS_MIN_WIDTH {
            right = right.push(
                text(format!(
                    "{} state{} · {} transition{}",
                    tab.states.len(),
                    if tab.states.len() == 1 { "" } else { "s" },
                    transition_count,
                    if transition_count == 1 { "" } else { "s" },
                ))
                .size(12)
                .style(theme::text_faint)
                .wrapping(Wrapping::None),
            );
            }
            if !tab.states.is_empty() {
                right = right.push(widgets::chip(
                    if insight.deterministic { "Deterministic" } else { "Nondeterministic" },
                    Tone::Neutral,
                ));
            }
        }

        let health: Element<'_, Message> = match &insight.problem {
            None if tab.states.is_empty() && !tab.machine.is_grammar() => Space::new(0, 0).into(),
            None => widgets::chip_with_icon(Icon::Check, Ink::Success, "Ready to run", Tone::Success),
            Some(problem) => {
                let short: String = if problem.chars().count() > 48 {
                    format!("{}…", problem.chars().take(47).collect::<String>())
                } else {
                    problem.clone()
                };
                tip(
                    widgets::chip_with_icon(Icon::Warn, Ink::Accent, short, Tone::Warning),
                    problem.clone(),
                    tooltip::Position::Top,
                )
            }
        };
        right = right.push(health);

        if !tab.machine.is_grammar() {
            right = right.push(
                text(format!("{:.0}%", tab.state_machine.zoom() * 100.0))
                    .size(12)
                    .font(theme::MONO)
                    .style(theme::text_faint),
            );
        }

        container(row![left, horizontal_space(), right].spacing(24).align_y(Alignment::Center))
            .padding([5, 14])
            .width(Length::Fill)
            .style(theme::status_bar)
            .into()
    }
}

fn menu_item<'a>(
    glyph: Icon,
    label: &'a str,
    shortcut: Option<&'a str>,
    message: Option<Message>,
    note: Option<&'a str>,
) -> Element<'a, Message> {
    let enabled = message.is_some();
    let mut body = column![text(label).size(14).font(theme::SEMIBOLD)].spacing(1);
    if let Some(note) = note {
        body = body.push(text(note).size(11).style(theme::text_faint));
    }
    let mut line = row![
        container(icon(glyph, 16.0, if enabled { Ink::Dim } else { Ink::Faint })).padding([0, 2]),
        body,
        horizontal_space(),
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    if let Some(shortcut) = shortcut {
        line = line.push(text(shortcut).size(11).font(theme::MONO).style(theme::text_faint));
    }
    button(line)
        .on_press_maybe(message)
        .width(Length::Fill)
        .padding([7, 10])
        .style(theme::menu_item)
        .into()
}

fn menu_separator<'a>() -> Element<'a, Message> {
    container(iced::widget::horizontal_rule(1).style(theme::divider))
        .padding([4, 6])
        .into()
}
