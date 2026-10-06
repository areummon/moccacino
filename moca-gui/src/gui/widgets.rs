
use iced::widget::text::Wrapping;
use iced::widget::{button, column, container, horizontal_space, row, tooltip, Space};
use iced::{Alignment, Element, Length};

use super::icons::{icon, Icon, Ink};
use super::message::Message;
use crate::gui::theme::{self, Family, Tone};

pub(crate) fn text<'a>(content: impl iced::widget::text::IntoFragment<'a>) -> iced::widget::Text<'a> {
    let content = content.into_fragment();
    let shaping = shaping_for(&content);
    iced::widget::text(content).shaping(shaping)
}

pub(crate) fn shaping_for(content: &str) -> iced::widget::text::Shaping {
    let covered = content.chars().all(|c| {
        (c as u32) < 0x100 || (0x2000..=0x206F).contains(&(c as u32)) || c == '\u{2212}'
    });
    if covered {
        iced::widget::text::Shaping::Basic
    } else {
        iced::widget::text::Shaping::Advanced
    }
}

pub(crate) fn tip<'a>(
    content: impl Into<Element<'a, Message>>,
    label: impl Into<String>,
    position: tooltip::Position,
) -> Element<'a, Message> {
    tooltip(
        content,
        container(text(label.into()).size(12)).padding([5, 9]).max_width(320),
        position,
    )
    .gap(6)
    .style(theme::tooltip)
    .into()
}

pub(crate) fn icon_button<'a>(glyph: Icon, message: Option<Message>, ink: Ink) -> button::Button<'a, Message> {
    button(icon(glyph, 16.0, ink))
        .on_press_maybe(message)
        .padding(6)
        .style(theme::button_ghost)
}

pub(crate) fn labeled<'a>(glyph: Option<(Icon, Ink)>, label: impl Into<String>) -> Element<'a, Message> {
    let label = text(label.into()).size(14).font(theme::SEMIBOLD).wrapping(Wrapping::None);
    match glyph {
        Some((glyph, ink)) => row![icon(glyph, 15.0, ink), label]
            .spacing(6)
            .align_y(Alignment::Center)
            .into(),
        None => label.into(),
    }
}

pub(crate) fn primary<'a>(label: impl Into<String>, message: Option<Message>) -> button::Button<'a, Message> {
    button(labeled(None, label))
        .on_press_maybe(message)
        .padding([7, 16])
        .style(theme::button_primary)
}

pub(crate) fn secondary<'a>(label: impl Into<String>, message: Option<Message>) -> button::Button<'a, Message> {
    button(labeled(None, label))
        .on_press_maybe(message)
        .padding([7, 14])
        .style(theme::button_secondary)
}

pub(crate) fn ghost<'a>(label: impl Into<String>, message: Option<Message>) -> button::Button<'a, Message> {
    button(labeled(None, label))
        .on_press_maybe(message)
        .padding([7, 14])
        .style(theme::button_ghost)
}

pub(crate) fn chip<'a>(label: impl Into<String>, tone: Tone) -> Element<'a, Message> {
    container(text(label.into()).size(12).font(theme::SEMIBOLD).wrapping(Wrapping::None))
        .padding([3, 10])
        .style(theme::chip(tone))
        .into()
}

pub(crate) fn chip_with_icon<'a>(glyph: Icon, ink: Ink, label: impl Into<String>, tone: Tone) -> Element<'a, Message> {
    container(
        row![icon(glyph, 12.0, ink), text(label.into()).size(12).font(theme::SEMIBOLD).wrapping(Wrapping::None)]
            .spacing(5)
            .align_y(Alignment::Center),
    )
    .padding([3, 10])
    .style(theme::chip(tone))
    .into()
}

pub(crate) fn family_dot<'a>(family: Family, size: f32) -> Element<'a, Message> {
    container(Space::new(Length::Fixed(size), Length::Fixed(size)))
        .style(theme::family_dot(family))
        .into()
}

pub(crate) fn kbd<'a>(label: impl Into<String>) -> Element<'a, Message> {
    container(text(label.into()).size(11).font(theme::MONO).wrapping(Wrapping::None))
        .padding([1, 6])
        .style(theme::sunken)
        .into()
}

pub(crate) fn caption<'a>(label: impl Into<String>) -> Element<'a, Message> {
    text(label.into()).size(12).font(theme::SEMIBOLD).style(theme::text_faint).into()
}

pub(crate) fn hint<'a>(label: impl Into<String>) -> Element<'a, Message> {
    text(label.into()).size(13).style(theme::text_faint).into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome {
    Idle,
    Running,
    Accepted,
    Rejected,
}

impl Outcome {
    pub(crate) fn from_finished(finished: Option<bool>) -> Self {
        match finished {
            Some(true) => Outcome::Accepted,
            Some(false) => Outcome::Rejected,
            None => Outcome::Running,
        }
    }
}

pub(crate) fn outcome_chip<'a>(outcome: Outcome) -> Element<'a, Message> {
    match outcome {
        Outcome::Idle => chip("Ready", Tone::Neutral),
        Outcome::Running => chip("Running…", Tone::Accent),
        Outcome::Accepted => chip_with_icon(Icon::Check, Ink::Success, "Accepted", Tone::Success),
        Outcome::Rejected => chip_with_icon(Icon::Close, Ink::Danger, "Rejected", Tone::Danger),
    }
}

pub(crate) fn stat<'a>(label: impl Into<String>, value: impl Into<String>) -> Element<'a, Message> {
    row![
        text(label.into()).size(12).style(theme::text_faint).wrapping(Wrapping::None),
        text(value.into()).size(13).font(theme::SEMIBOLD).wrapping(Wrapping::None),
    ]
    .spacing(5)
    .align_y(Alignment::Center)
    .into()
}

pub(crate) struct RunMessages {
    pub(crate) play: Message,
    pub(crate) step: Message,
    pub(crate) reset: Message,
}

pub(crate) fn run_controls<'a>(playing: bool, loaded: bool, finished: bool, messages: RunMessages) -> Element<'a, Message> {
    let can_advance = loaded && !finished;
    let reset = tip(
        icon_button(Icon::Reset, loaded.then_some(messages.reset), Ink::Text),
        "Reset the run",
        tooltip::Position::Top,
    );
    let step = tip(
        icon_button(Icon::Step, can_advance.then_some(messages.step), Ink::Text),
        "Step once  (→)",
        tooltip::Position::Top,
    );
    let play: Element<'a, Message> = if playing {
        button(icon(Icon::Pause, 16.0, Ink::OnAccent))
            .on_press(messages.play)
            .padding(6)
            .style(theme::button_primary)
            .into()
    } else {
        icon_button(Icon::Play, can_advance.then_some(messages.play), Ink::Text).into()
    };
    let play = tip(play, if playing { "Pause  (Space)" } else { "Play  (Space)" }, tooltip::Position::Top);

    container(row![reset, step, play].spacing(2).align_y(Alignment::Center))
        .padding(2)
        .style(theme::sunken)
        .into()
}

pub(crate) fn dock_header<'a>(
    family: Family,
    title: &'a str,
    collapsed: bool,
    compact: bool,
    controls: Element<'a, Message>,
    status: Element<'a, Message>,
) -> Element<'a, Message> {
    let chevron = tip(
        icon_button(
            if collapsed { Icon::ChevronUp } else { Icon::ChevronDown },
            Some(Message::ToggleDock),
            Ink::Dim,
        ),
        if collapsed { "Expand the run panel" } else { "Collapse the run panel" },
        tooltip::Position::Top,
    );
    let title = row![family_dot(family, 9.0), text(title).size(14).font(theme::BOLD).wrapping(Wrapping::None)]
        .spacing(8)
        .align_y(Alignment::Center);
    if compact {
        return column![
            row![title, Space::with_width(12), status, horizontal_space(), chevron]
                .spacing(8)
                .align_y(Alignment::Center),
            controls,
        ]
        .spacing(10)
        .into();
    }
    row![title, Space::with_width(8), controls, Space::with_width(8), status, horizontal_space(), chevron]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
}

pub(crate) const DIALOG_SM: f32 = 360.0;
pub(crate) const DIALOG_MD: f32 = 460.0;
pub(crate) const DIALOG_LG: f32 = 640.0;

pub(crate) fn dialog_card<'a>(
    title: impl Into<String>,
    subtitle: Option<String>,
    body: Element<'a, Message>,
    actions: Vec<Element<'a, Message>>,
    width: f32,
) -> Element<'a, Message> {
    let mut header = column![text(title.into()).size(18).font(theme::BOLD)].spacing(4);
    if let Some(subtitle) = subtitle {
        header = header.push(text(subtitle).size(13).style(theme::text_dim));
    }
    let mut footer = row![horizontal_space()].spacing(8).align_y(Alignment::Center);
    for action in actions {
        footer = footer.push(action);
    }
    container(column![header, body, footer].spacing(16))
        .padding(22)
        .width(Length::Fill)
        .max_width(width)
        .style(theme::dialog)
        .into()
}

pub(crate) fn modal_layer<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    iced::widget::mouse_area(
        container(iced::widget::mouse_area(content).on_press(Message::Noop))
            .center(Length::Fill)
            .padding(16)
            .style(theme::scrim),
    )
    .on_press(Message::DismissModal)
    .into()
}

pub(crate) fn blocking_layer<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    iced::widget::mouse_area(
        container(iced::widget::mouse_area(content).on_press(Message::Noop))
            .center(Length::Fill)
            .padding(16)
            .style(theme::scrim),
    )
    .on_press(Message::Noop)
    .into()
}

pub(crate) fn run_input<'a>(
    value: &str,
    on_input: fn(String) -> Message,
    on_load: Message,
    loaded: bool,
) -> Element<'a, Message> {
    row![
        iced::widget::text_input("Input word (blank = ε)", value)
            .on_input(on_input)
            .on_submit(on_load.clone())
            .padding([7, 10])
            .size(14)
            .font(theme::MONO)
            .width(Length::Fixed(210.0))
            .style(theme::input),
        tip(
            if loaded { secondary("Reload", Some(on_load)) } else { primary("Load", Some(on_load)) },
            "Load the word and start stepping  (Enter)",
            tooltip::Position::Top,
        ),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

pub(crate) fn run_dock<'a>(
    family: Family,
    title: &'a str,
    collapsed: bool,
    compact: bool,
    controls: Element<'a, Message>,
    status: Element<'a, Message>,
    body: Element<'a, Message>,
) -> Element<'a, Message> {
    let mut content = column![dock_header(family, title, collapsed, compact, controls, status)].spacing(12);
    if !collapsed {
        content = content.push(body);
    }
    container(content)
        .padding([10, 14])
        .width(Length::Fill)
        .style(theme::card)
        .into()
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum CellSize {
    Regular,
    Compact,
}

impl CellSize {
    fn metrics(self) -> (f32, f32, f32) {
        match self {
            CellSize::Regular => (28.0, 30.0, 15.0),
            CellSize::Compact => (20.0, 22.0, 12.0),
        }
    }
}

pub(crate) fn cell<'a>(symbol: impl Into<String>, size: CellSize, focus: bool, spent: bool, min_width: Option<f32>) -> Element<'a, Message> {
    let (width, height, font_size) = size.metrics();
    container(text(symbol.into()).font(theme::MONO).size(font_size))
        .width(Length::Fixed(min_width.unwrap_or(width)))
        .height(Length::Fixed(height))
        .center_x(Length::Fixed(min_width.unwrap_or(width)))
        .center_y(Length::Fixed(height))
        .style(theme::cell(focus, spent))
        .into()
}

pub(crate) fn ribbon<'a>(input: &str, consumed: usize, size: CellSize) -> Element<'a, Message> {
    if input.is_empty() {
        return cell("ε", size, consumed == 0, false, None);
    }
    let mut cells = row![].spacing(3).align_y(Alignment::Center);
    for (index, c) in input.chars().enumerate() {
        cells = cells.push(cell(c.to_string(), size, index == consumed, index < consumed, None));
    }
    cells.into()
}

pub(crate) fn tape<'a>(window: &str, head_offset: usize, size: CellSize) -> Element<'a, Message> {
    let mut cells = row![].spacing(3).align_y(Alignment::Center);
    for (index, c) in window.chars().enumerate() {
        cells = cells.push(cell(c.to_string(), size, index == head_offset, false, None));
    }
    cells.into()
}

pub(crate) fn stack_strip<'a>(entries: &[String], visible: usize) -> Element<'a, Message> {
    let shown = entries.len().min(visible);
    let hidden = entries.len() - shown;
    let mut cells = row![].spacing(3).align_y(Alignment::Center);
    if hidden > 0 {
        cells = cells.push(text(format!("+{hidden} ")).size(12).style(theme::text_faint));
    }
    if entries.is_empty() {
        cells = cells.push(text("empty").size(12).style(theme::text_faint));
    }
    for (offset, entry) in entries[hidden..].iter().enumerate() {
        let is_top = offset + 1 == shown;
        let width = (entry.chars().count() as f32 * 9.0 + 16.0).max(28.0);
        cells = cells.push(cell(entry.clone(), CellSize::Regular, is_top, false, Some(width)));
    }
    cells.into()
}

pub(crate) fn strip<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    iced::widget::scrollable(container(content).padding(iced::Padding { top: 0.0, right: 0.0, bottom: 8.0, left: 0.0 }))
        .direction(iced::widget::scrollable::Direction::Horizontal(
            iced::widget::scrollable::Scrollbar::new().width(4).scroller_width(4),
        ))
        .width(Length::Fill)
        .style(theme::scroll)
        .into()
}

pub(crate) fn lanes<'a>(rows: Vec<Element<'a, Message>>, hidden: usize) -> Element<'a, Message> {
    const ROW_HEIGHT: f32 = 30.0;
    let visible_rows = rows.len();
    let mut list = column![].spacing(6);
    for row in rows {
        list = list.push(row);
    }
    if hidden > 0 {
        list = list.push(hint(format!("… {hidden} more branches")));
    }
    let height = ((visible_rows as f32 + if hidden > 0 { 1.0 } else { 0.0 }) * (ROW_HEIGHT + 6.0)).min(200.0);
    container(
        iced::widget::scrollable(container(list).padding([2, 4]))
            .direction(iced::widget::scrollable::Direction::Both {
                vertical: iced::widget::scrollable::Scrollbar::new().width(4).scroller_width(4),
                horizontal: iced::widget::scrollable::Scrollbar::new().width(4).scroller_width(4),
            })
            .height(Length::Fixed(height))
            .style(theme::scroll),
    )
    .padding(8)
    .style(theme::sunken)
    .into()
}

pub(crate) fn lane<'a>(index: usize, state: String, detail: Element<'a, Message>) -> Element<'a, Message> {
    row![
        container(text(format!("{:>2}", index + 1)).size(11).font(theme::MONO).style(theme::text_faint))
            .width(Length::Fixed(20.0)),
        container(text(state).size(12).font(theme::MONO))
            .padding([2, 8])
            .style(theme::chip(Tone::Accent)),
        detail,
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}
