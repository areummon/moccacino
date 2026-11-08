
use iced::widget::{
    button, column, container, horizontal_space, mouse_area, row, scrollable, stack, text_input, tooltip,
    vertical_rule, Space,
};
use iced::widget::text::Wrapping;
use iced::{Alignment, Element, Length, Padding};

use super::feature_icon::feature_icon;
use super::icons::{icon, Icon, Ink};
use super::message::{Menu, Message};
use super::metrics::{
    BRAND_TEXT_MIN_WIDTH, DOCK_COMPACT_MAX_WIDTH, HEADER_HEIGHT, PALETTE_ICONS_ONLY_MAX_WIDTH, STACKED_HEADER_MAX_WIDTH,
    STATUS_BAR_HEIGHT, STATUS_COUNTS_MIN_WIDTH, STATUS_HINT_MIN_WIDTH, TAB_ROW_HEIGHT,
};
use super::shortcuts::{hint, Action};
use super::widgets::{self, text, tip};
use super::tab::Tab;
use super::workspace::{TAB_RENAME_INPUT, TAB_STRIP};
use crate::theme::{self, ThemeMode, Tone};
use crate::family::Family;
use crate::state_machine::{CanvasContext, EditorTool};

const HEADER_PADDING_X: f32 = 12.0;
const BRAND_WIDTH: f32 = 128.0;
const BRAND_WIDTH_COMPACT: f32 = 34.0;
const MENU_GAP: f32 = 2.0;
const MENUS: [(Menu, &str, f32); 4] = [
    (Menu::New, "New", 70.0),
    (Menu::Operations, "Operations", 112.0),
    (Menu::File, "File", 60.0),
    (Menu::Export, "Export", 78.0),
];
const ICON_BUTTON: f32 = 32.0;
const RIGHT_CLUSTER_SPACING: f32 = 4.0;
const NEW_TAB_GAP: f32 = 8.0;
/// Inset of the floating canvas surfaces (tool palette, Clear, zoom chip), so all three are 40 px tall.
const FLOATING_PADDING: u16 = 4;
const DROPDOWN_WIDTH: f32 = 304.0;
/// Width of the leading icon column in a menu, so labels in one menu share a left edge.
const MENU_ICON_SLOT: f32 = 20.0;
const FAMILY_ICON_SLOT: f32 = 30.0;

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
            brand_row = brand_row.push(text("moccacino").size(theme::TEXT_LG).font(theme::BOLD));
        }
        let brand = container(brand_row).width(Length::Fixed(brand_width));

        let mut menus = row![].spacing(MENU_GAP).align_y(Alignment::Center);
        for (menu, label, width) in MENUS {
            let open = self.open_menu == Some(menu);
            menus = menus.push(
                button(
                    row![
                        text(label).size(theme::TEXT_BASE).font(theme::SEMIBOLD),
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
            format!("New tab  ({})", hint(Action::NewTabMenu)),
            tooltip::Position::Bottom,
        );
        let help = tip(
            button(icon(Icon::Keyboard, 16.0, Ink::Dim))
                .on_press(Message::ToggleShortcuts)
                .width(Length::Fixed(ICON_BUTTON))
                .padding(8)
                .style(theme::button_ghost),
            format!("Keyboard shortcuts  ({})", hint(Action::Shortcuts)),
            tooltip::Position::Bottom,
        );
        let (theme_icon, theme_tip) = match self.theme_mode {
            ThemeMode::Light => (Icon::Moon, "dark"),
            ThemeMode::Dark => (Icon::Sun, "light"),
        };
        let theme_toggle = tip(
            button(icon(theme_icon, 16.0, Ink::Dim))
                .on_press(Message::ToggleTheme)
                .width(Length::Fixed(ICON_BUTTON))
                .padding(8)
                .style(theme::button_ghost),
            format!("Switch to the {theme_tip} theme  ({})", hint(Action::ToggleTheme)),
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
        top = top.push(plus).push(Space::with_width(NEW_TAB_GAP)).push(help).push(theme_toggle);

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
        container(header).width(Length::Fill).style(theme::app_background).into()
    }

    fn create_tab_strip(&self) -> Element<'_, Message> {
        let closable = self.tabs.len() > 1;
        let mut strip = row![].spacing(4).align_y(Alignment::Center);
        for (index, tab) in self.tabs.iter().enumerate() {
            let active = index == self.active_tab;
            let family = tab.family();

            let name: Element<'_, Message> = if self.renaming_tab == Some(index) {
                text_input("Tab name", &self.tab_rename_text)
                    .id(TAB_RENAME_INPUT)
                    .on_input(Message::TabRenameChanged)
                    .on_submit(Message::TabRenameSubmit)
                    .size(theme::TEXT_MD)
                    .padding([2, 6])
                    .width(Length::Fixed(140.0))
                    .style(theme::input)
                    .into()
            } else {
                text(tab.name.clone())
                    .size(theme::TEXT_MD)
                    .font(if active { theme::SEMIBOLD } else { theme::UI })
                    .into()
            };

            let mut content = row![widgets::family_dot(family, 8.0), name]
                .spacing(7)
                .align_y(Alignment::Center);
            if tab.insight.unsaved {
                content = content.push(
                    widgets::dot(6.0, theme::unsaved_dot),
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
                .padding(Padding { top: 7.5, bottom: 7.5, left: 12.0, right: if closable { 6.0 } else { 12.0 } })
                .style(theme::tab_pill(active));
            strip = strip.push(pill);
        }

        scrollable(container(strip).padding([4, 2]))
            .id(scrollable::Id::new(TAB_STRIP))
            .direction(scrollable::Direction::Horizontal(
                widgets::thin_scrollbar(3.0).margin(0),
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
                let right = HEADER_PADDING_X + 2.0 * ICON_BUTTON + NEW_TAB_GAP + 3.0 * RIGHT_CLUSTER_SPACING;
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
        let is_finite = tab.family() == Family::Finite;
        let has_states = !tab.states.is_empty();
        match menu {
            Menu::New | Menu::NewFromTabs => {
                let mut items = column![].spacing(2);
                for family in Family::ALL {
                    items = items.push(
                        button(
                            row![
                                feature_icon(family, FAMILY_ICON_SLOT, false),
                                column![
                                    text(family.title()).size(theme::TEXT_BASE).font(theme::SEMIBOLD),
                                    text(family.blurb()).size(theme::TEXT_SM).style(theme::text_faint),
                                ]
                                .spacing(1),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                        )
                        .on_press(Message::NewTab(family))
                        .width(Length::Fill)
                        .padding(MENU_ITEM_PADDING)
                        .style(theme::menu_item),
                    );
                }
                items
                    .push(menu_separator())
                    .push(menu_item_in(
                        FAMILY_ICON_SLOT,
                        Icon::Folder,
                        "Open .ce file…",
                        Some(hint(Action::Open)),
                        Some(Message::OpenLoadDialog),
                        None,
                    ))
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
                        nfa_to_dfa_reason.is_none().then_some(Message::NfaToDfa),
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
                menu_item(Icon::Folder, "Open .ce file…", Some(hint(Action::Open)), Some(Message::OpenLoadDialog), None),
                menu_item(Icon::Check, "Save tab as .ce…", Some(hint(Action::Save)), Some(Message::OpenSaveDialog), None),
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
                    family: tab.family(),
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
                        text("An empty canvas").size(theme::TEXT_LG).font(theme::SEMIBOLD).style(theme::text_dim),
                        row![
                            text("Press").size(theme::TEXT_MD).style(theme::text_faint),
                            widgets::kbd(hint(Action::Tool(EditorTool::State))),
                            text("and click to place states, then").size(theme::TEXT_MD).style(theme::text_faint),
                            widgets::kbd(hint(Action::Tool(EditorTool::Transition))),
                            text("to connect them").size(theme::TEXT_MD).style(theme::text_faint),
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
                    container(widgets::text_button(
                        Some((Icon::Delete, Ink::Danger)),
                        "Clear",
                        Some(Message::Clear),
                        theme::button_danger_ghost,
                    ))
                    .padding(FLOATING_PADDING)
                    .style(theme::floating),
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
            let ink = if is_active { Ink::Accent } else { Ink::Dim };
            let mut label = row![icon(tool_icon(tool), widgets::ICON_SIZE, ink)].spacing(6).align_y(Alignment::Center);
            if captions {
                label = label.push(text(tool.name()).size(theme::TEXT_BASE).font(theme::SEMIBOLD).wrapping(Wrapping::None));
            }
            tools = tools.push(tip(
                button(label)
                .on_press(Message::SelectTool(tool))
                .padding([7, 12])
                .style(theme::toggle_pill(is_active)),
                format!("{}  ({})\n{}", tool.name(), hint(Action::Tool(tool)), tool.tooltip()),
                tooltip::Position::Bottom,
            ));
        }
        container(tools).padding(FLOATING_PADDING).style(theme::floating).into()
    }

    fn create_zoom_chip(&self) -> Element<'_, Message> {
        let zoom = self.get_active_tab().state_machine.zoom();
        let chip = row![
            tip(widgets::icon_button(Icon::Minus, Some(Message::ZoomOut), Ink::Dim), format!("Zoom out  ({})", hint(Action::ZoomOut)), tooltip::Position::Top),
            tip(
                button(text(format!("{:>3.0}%", zoom * 100.0)).size(theme::TEXT_SM).font(theme::MONO))
                    .on_press(Message::ZoomReset)
                    .padding([8, 4])
                    .style(theme::button_ghost),
                format!("Reset to 100%  ({})", hint(Action::ZoomReset)),
                tooltip::Position::Top,
            ),
            tip(widgets::icon_button(Icon::Plus, Some(Message::ZoomIn), Ink::Dim), format!("Zoom in  ({})", hint(Action::ZoomIn)), tooltip::Position::Top),
            tip(widgets::icon_button(Icon::Fit, Some(Message::FitView), Ink::Dim), format!("Fit to view  ({})", hint(Action::Fit)), tooltip::Position::Top),
        ]
        .spacing(0)
        .align_y(Alignment::Center);
        container(chip).padding(FLOATING_PADDING).style(theme::floating).into()
    }

    pub(crate) fn dock_is_compact(&self) -> bool {
        self.layout_width() < DOCK_COMPACT_MAX_WIDTH
    }

    pub(crate) fn create_run_dock(&self) -> Element<'_, Message> {
        container(self.create_run_panel()).padding(Padding { top: 0.0, right: 12.0, bottom: 8.0, left: 12.0 }).into()
    }

    pub(crate) fn create_status_bar(&self) -> Element<'_, Message> {
        let tab = self.get_active_tab();
        let (left, right) = if tab.is_grammar() { grammar_status(tab) } else { self.automaton_status(tab) };
        container(row![left, horizontal_space(), right].spacing(24).align_y(Alignment::Center))
            .padding([0, 14])
            .width(Length::Fill)
            .height(Length::Fixed(STATUS_BAR_HEIGHT))
            .center_y(Length::Fixed(STATUS_BAR_HEIGHT))
            .style(theme::status_bar)
            .into()
    }

    fn automaton_status<'a>(&self, tab: &'a Tab) -> (Element<'a, Message>, Element<'a, Message>) {
        let tool = tab.active_tool();
        let mut left = row![
            icon(tool_icon(tool), 13.0, Ink::Dim),
            text(tool.name()).size(theme::TEXT_SM).font(theme::SEMIBOLD).wrapping(Wrapping::None),
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        if self.layout_width() >= STATUS_HINT_MIN_WIDTH {
            left = left.push(text(tool.hint()).size(theme::TEXT_SM).style(theme::text_faint).wrapping(Wrapping::None));
        }

        let mut right = row![].spacing(10).align_y(Alignment::Center);
        if self.layout_width() >= STATUS_COUNTS_MIN_WIDTH {
            let transition_count: usize = tab.transitions.values().map(|labels| labels.len()).sum();
            right = right.push(
                text(format!(
                    "{} state{} · {} transition{}",
                    tab.states.len(),
                    if tab.states.len() == 1 { "" } else { "s" },
                    transition_count,
                    if transition_count == 1 { "" } else { "s" },
                ))
                .size(theme::TEXT_SM)
                .style(theme::text_faint)
                .wrapping(Wrapping::None),
            );
        }
        if !tab.states.is_empty() {
            let determinism = if tab.insight.deterministic { "Deterministic" } else { "Nondeterministic" };
            right = right.push(widgets::chip(determinism, Tone::Neutral)).push(health_chip(&tab.insight.problem));
        }
        right = right.push(
            text(format!("{:.0}%", tab.state_machine.zoom() * 100.0))
                .size(theme::TEXT_SM)
                .font(theme::MONO)
                .style(theme::text_faint),
        );
        (left.into(), right.into())
    }
}

fn grammar_status(tab: &Tab) -> (Element<'_, Message>, Element<'_, Message>) {
    let left = row![
        widgets::family_dot(Family::Grammar, 7.0),
        text("Grammar editor").size(theme::TEXT_SM),
        text(format!("{} productions", tab.insight.production_count)).size(theme::TEXT_SM).style(theme::text_faint),
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    (left.into(), health_chip(&tab.insight.problem))
}

fn health_chip(problem: &Option<String>) -> Element<'_, Message> {
    let Some(problem) = problem else {
        return widgets::chip_with_icon(Icon::Check, Ink::Success, "Ready to run", Tone::Success);
    };
    let short: String = if problem.chars().count() > 48 {
        format!("{}…", problem.chars().take(47).collect::<String>())
    } else {
        problem.clone()
    };
    tip(widgets::chip_with_icon(Icon::Warn, Ink::Warning, short, Tone::Warning), problem.clone(), tooltip::Position::Top)
}

const MENU_ITEM_PADDING: [u16; 2] = [7, 10];

fn menu_item<'a>(
    glyph: Icon,
    label: &'a str,
    shortcut: Option<String>,
    message: Option<Message>,
    note: Option<&'a str>,
) -> Element<'a, Message> {
    menu_item_in(MENU_ICON_SLOT, glyph, label, shortcut, message, note)
}

fn menu_item_in<'a>(
    slot: f32,
    glyph: Icon,
    label: &'a str,
    shortcut: Option<String>,
    message: Option<Message>,
    note: Option<&'a str>,
) -> Element<'a, Message> {
    let enabled = message.is_some();
    let mut body = column![text(label).size(theme::TEXT_BASE).font(theme::SEMIBOLD)].spacing(1);
    if let Some(note) = note {
        body = body.push(text(note).size(theme::TEXT_XS).style(theme::text_faint));
    }
    let mut line = row![
        container(icon(glyph, widgets::ICON_SIZE, if enabled { Ink::Dim } else { Ink::Faint })).center_x(Length::Fixed(slot)),
        body,
        horizontal_space(),
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    if let Some(shortcut) = shortcut {
        line = line.push(text(shortcut).size(theme::TEXT_XS).font(theme::MONO).style(theme::text_faint));
    }
    button(line)
        .on_press_maybe(message)
        .width(Length::Fill)
        .padding(MENU_ITEM_PADDING)
        .style(theme::menu_item)
        .into()
}

fn menu_separator<'a>() -> Element<'a, Message> {
    container(iced::widget::horizontal_rule(1).style(theme::divider))
        .padding([4, 6])
        .into()
}

fn tool_icon(tool: EditorTool) -> Icon {
    match tool {
        EditorTool::Arrow => Icon::Select,
        EditorTool::State => Icon::State,
        EditorTool::Transition => Icon::Transition,
        EditorTool::Delete => Icon::Delete,
    }
}
