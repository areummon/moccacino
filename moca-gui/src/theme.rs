
use iced::font::Weight;
use iced::widget::{button, container, rule, scrollable, text, text_editor, text_input};
use crate::family::Family;
use iced::{Background, Border, Color, Font, Shadow, Theme, Vector};

const fn hex(rgb: u32) -> Color {
    Color::from_rgb(
        ((rgb >> 16) & 0xff) as f32 / 255.0,
        ((rgb >> 8) & 0xff) as f32 / 255.0,
        (rgb & 0xff) as f32 / 255.0,
    )
}

const fn hexa(rgb: u32, a: f32) -> Color {
    let c = hex(rgb);
    Color { a, ..c }
}

pub(crate) const UI: Font = Font::with_name("Nunito");
pub(crate) const SEMIBOLD: Font = Font { weight: Weight::Semibold, ..UI };
pub(crate) const BOLD: Font = Font { weight: Weight::Bold, ..UI };
pub(crate) const MONO: Font = Font::with_name("JetBrains Mono");

/// The type scale. GUI text sizes come only from here, so equal roles read equal.
pub(crate) const TEXT_XS: f32 = 11.0;
pub(crate) const TEXT_SM: f32 = 12.0;
pub(crate) const TEXT_MD: f32 = 13.0;
pub(crate) const TEXT_BASE: f32 = 14.0;
pub(crate) const TEXT_LG: f32 = 16.0;
pub(crate) const TEXT_XL: f32 = 18.0;
pub(crate) const TEXT_DISPLAY: f32 = 22.0;

const RADIUS_SM: f32 = 8.0;
const RADIUS_MD: f32 = 10.0;
const RADIUS_LG: f32 = 14.0;
const RADIUS_XL: f32 = 18.0;
const RADIUS_PILL: f32 = 999.0;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Palette {
    pub(crate) bg: Color,
    pub(crate) surface: Color,
    pub(crate) raised: Color,
    pub(crate) hover: Color,
    pub(crate) sunken: Color,
    pub(crate) border: Color,
    pub(crate) border_soft: Color,
    pub(crate) text: Color,
    pub(crate) text_dim: Color,
    pub(crate) text_faint: Color,
    pub(crate) accent: Color,
    pub(crate) accent_hover: Color,
    pub(crate) accent_soft: Color,
    pub(crate) accent_text: Color,
    pub(crate) accent_strong: Color,
    pub(crate) success: Color,
    pub(crate) success_soft: Color,
    pub(crate) danger: Color,
    pub(crate) danger_soft: Color,
    pub(crate) on_danger: Color,
    pub(crate) warning: Color,
    pub(crate) warning_soft: Color,
    pub(crate) shadow: Color,
    pub(crate) scrim: Color,
    pub(crate) selection: Color,
    pub(crate) canvas_bg: Color,
    pub(crate) canvas_grid: Color,
    pub(crate) node_fill: Color,
    pub(crate) node_stroke: Color,
    pub(crate) edge: Color,
    pub(crate) label_bg: Color,
    pub(crate) family_finite: Color,
    pub(crate) family_pda: Color,
    pub(crate) family_tm: Color,
    pub(crate) family_grammar: Color,
}

const LIGHT: Palette = Palette {
    bg: hex(0xF6EFE4),
    surface: hex(0xFFFBF5),
    raised: hex(0xFBF4EA),
    hover: hex(0xF3E8D9),
    sunken: hex(0xF4ECE0),
    border: hex(0xE4D6C2),
    border_soft: hex(0xEEE4D5),
    text: hex(0x4A3F35),
    text_dim: hex(0x7D6E60),
    text_faint: hex(0xA99A89),
    accent: hex(0xF2A488),
    accent_hover: hex(0xEE9576),
    accent_soft: hex(0xFCE5DA),
    accent_text: hex(0x4A2618),
    accent_strong: hex(0xB9603F),
    success: hex(0x5E9A6E),
    success_soft: hex(0xDDF0E2),
    danger: hex(0xC25F5F),
    danger_soft: hex(0xFADEDC),
    on_danger: hex(0xFFFBF5),
    warning: hex(0xB5842A),
    warning_soft: hex(0xFAEFD2),
    shadow: hexa(0x5A4432, 0.14),
    scrim: hexa(0x3C2D23, 0.28),
    selection: hexa(0xF2A488, 0.35),
    canvas_bg: hex(0xFFFCF7),
    canvas_grid: hex(0xE9DDCC),
    node_fill: hex(0xFFF7EE),
    node_stroke: hex(0x8C7A6B),
    edge: hex(0x9C8B7C),
    label_bg: hex(0xFFF8F0),
    family_finite: hex(0xA8D5BA),
    family_pda: hex(0xC9B8E8),
    family_tm: hex(0xF7C6A3),
    family_grammar: hex(0xF3DE8A),
};

const DARK: Palette = Palette {
    bg: hex(0x1F1A17),
    surface: hex(0x2A2420),
    raised: hex(0x332C27),
    hover: hex(0x3D3530),
    sunken: hex(0x221D1A),
    border: hex(0x4A4038),
    border_soft: hex(0x3A322C),
    text: hex(0xEFE4D6),
    text_dim: hex(0xC2B3A2),
    text_faint: hex(0x8F7F70),
    accent: hex(0xF2A488),
    accent_hover: hex(0xF6B79E),
    accent_soft: hex(0x4B342A),
    accent_text: hex(0x2A1810),
    accent_strong: hex(0xF6B79E),
    success: hex(0x9CCFAA),
    success_soft: hex(0x2D3E32),
    danger: hex(0xEE9A9A),
    danger_soft: hex(0x4A2D2D),
    on_danger: hex(0x1F1A17),
    warning: hex(0xEBC477),
    warning_soft: hex(0x463B25),
    shadow: hexa(0x000000, 0.38),
    scrim: hexa(0x0A0604, 0.52),
    selection: hexa(0xF2A488, 0.30),
    canvas_bg: hex(0x241F1B),
    canvas_grid: hex(0x3B332D),
    node_fill: hex(0x362D27),
    node_stroke: hex(0xD6C4AF),
    edge: hex(0xB8A690),
    label_bg: hex(0x2E2722),
    family_finite: hex(0x8FBFA2),
    family_pda: hex(0xB3A2D6),
    family_tm: hex(0xE8B18C),
    family_grammar: hex(0xDCC677),
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ThemeMode {
    #[default]
    Light,
    Dark,
}

impl ThemeMode {
    pub(crate) fn toggled(self) -> Self {
        match self {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        }
    }

    pub(crate) fn palette(self) -> &'static Palette {
        match self {
            ThemeMode::Light => &LIGHT,
            ThemeMode::Dark => &DARK,
        }
    }

    pub(crate) fn iced_theme(self) -> Theme {
        let p = self.palette();
        let name = match self {
            ThemeMode::Light => "Moccacino Light",
            ThemeMode::Dark => "Moccacino Dark",
        };
        Theme::custom(
            name.to_string(),
            iced::theme::Palette {
                background: p.bg,
                text: p.text,
                primary: p.accent,
                success: p.success,
                danger: p.danger,
            },
        )
    }
}

pub(crate) fn palette(theme: &Theme) -> &'static Palette {
    if theme.extended_palette().is_dark {
        &DARK
    } else {
        &LIGHT
    }
}

pub(crate) fn family_color(p: &Palette, family: Family) -> Color {
    match family {
        Family::Finite => p.family_finite,
        Family::Pushdown => p.family_pda,
        Family::Turing => p.family_tm,
        Family::Grammar => p.family_grammar,
    }
}

pub(crate) fn mix(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

pub(crate) fn alpha(color: Color, a: f32) -> Color {
    Color { a, ..color }
}

fn soft_shadow(p: &Palette, y: f32, blur: f32) -> Shadow {
    Shadow {
        color: p.shadow,
        offset: Vector::new(0.0, y),
        blur_radius: blur,
    }
}

fn border(color: Color, width: f32, radius: f32) -> Border {
    Border {
        color,
        width,
        radius: radius.into(),
    }
}

fn flat(background: Color, text_color: Color, radius: f32) -> button::Style {
    button::Style {
        background: Some(background.into()),
        text_color,
        border: border(Color::TRANSPARENT, 0.0, radius),
        shadow: Shadow::default(),
    }
}

fn pill(color: Color) -> container::Style {
    container::Style {
        background: Some(color.into()),
        border: border(Color::TRANSPARENT, 0.0, RADIUS_PILL),
        ..Default::default()
    }
}

pub(crate) fn text_dim(theme: &Theme) -> text::Style {
    text::Style { color: Some(palette(theme).text_dim) }
}

pub(crate) fn text_faint(theme: &Theme) -> text::Style {
    text::Style { color: Some(palette(theme).text_faint) }
}

pub(crate) fn text_danger(theme: &Theme) -> text::Style {
    text::Style { color: Some(palette(theme).danger) }
}

pub(crate) fn button_primary(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let background = match status {
        button::Status::Hovered => p.accent_hover,
        button::Status::Disabled => alpha(p.accent, 0.45),
        _ => p.accent,
    };
    button::Style {
        background: Some(background.into()),
        text_color: p.accent_text,
        border: border(Color::TRANSPARENT, 0.0, RADIUS_MD),
        shadow: if matches!(status, button::Status::Pressed | button::Status::Disabled) {
            Shadow::default()
        } else {
            soft_shadow(p, 1.0, 4.0)
        },
    }
}

pub(crate) fn button_secondary(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let (background, text_color) = match status {
        button::Status::Hovered | button::Status::Pressed => (p.hover, p.text),
        button::Status::Disabled => (p.raised, p.text_faint),
        _ => (p.raised, p.text),
    };
    button::Style {
        background: Some(background.into()),
        text_color,
        border: border(p.border, 1.0, RADIUS_MD),
        shadow: Shadow::default(),
    }
}

pub(crate) fn button_ghost(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    match status {
        button::Status::Hovered | button::Status::Pressed => flat(p.hover, p.text, RADIUS_SM),
        button::Status::Disabled => flat(Color::TRANSPARENT, p.text_faint, RADIUS_SM),
        _ => flat(Color::TRANSPARENT, p.text, RADIUS_SM),
    }
}

pub(crate) fn button_danger(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let background = match status {
        button::Status::Hovered => mix(p.danger, p.text, 0.12),
        _ => p.danger,
    };
    flat(background, p.on_danger, RADIUS_MD)
}

pub(crate) fn button_danger_ghost(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => p.danger_soft,
        _ => Color::TRANSPARENT,
    };
    flat(background, p.danger, RADIUS_SM)
}

pub(crate) fn menu_trigger(open: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        match status {
            _ if open => flat(p.accent_soft, p.accent_strong, RADIUS_SM),
            button::Status::Hovered | button::Status::Pressed => flat(p.hover, p.text, RADIUS_SM),
            _ => flat(Color::TRANSPARENT, p.text, RADIUS_SM),
        }
    }
}

pub(crate) fn menu_item(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    match status {
        button::Status::Hovered | button::Status::Pressed => flat(p.accent_soft, p.text, RADIUS_SM),
        button::Status::Disabled => flat(Color::TRANSPARENT, p.text_faint, RADIUS_SM),
        _ => flat(Color::TRANSPARENT, p.text, RADIUS_SM),
    }
}

pub(crate) fn tab_pill(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let (background, text_color, stroke) = if active {
            (p.surface, p.text, p.border)
        } else if hovered {
            (p.hover, p.text, Color::TRANSPARENT)
        } else {
            (Color::TRANSPARENT, p.text_dim, Color::TRANSPARENT)
        };
        button::Style {
            background: Some(background.into()),
            text_color,
            border: border(stroke, if active { 1.0 } else { 0.0 }, RADIUS_MD),
            shadow: if active { soft_shadow(p, 1.0, 5.0) } else { Shadow::default() },
        }
    }
}

pub(crate) fn toggle_pill(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        match (active, hovered) {
            (true, _) => flat(p.accent_soft, p.accent_strong, RADIUS_PILL),
            (false, true) => flat(p.hover, p.text, RADIUS_PILL),
            (false, false) => flat(Color::TRANSPARENT, p.text_dim, RADIUS_PILL),
        }
    }
}

pub(crate) fn option_card(selected: bool, tint: Color) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let background = if selected {
            mix(p.surface, tint, 0.22)
        } else if hovered {
            mix(p.surface, tint, 0.12)
        } else {
            p.surface
        };
        button::Style {
            background: Some(background.into()),
            text_color: p.text,
            border: border(
                if selected { tint } else { p.border_soft },
                if selected { 2.0 } else { 1.0 },
                RADIUS_LG,
            ),
            shadow: if selected || hovered { soft_shadow(p, 3.0, 12.0) } else { soft_shadow(p, 1.0, 4.0) },
        }
    }
}

pub(crate) fn tab_close(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    match status {
        button::Status::Hovered | button::Status::Pressed => flat(p.danger_soft, p.danger, RADIUS_PILL),
        _ => flat(Color::TRANSPARENT, p.text_faint, RADIUS_PILL),
    }
}

pub(crate) fn app_background(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.bg.into()),
        text_color: Some(p.text),
        ..Default::default()
    }
}

pub(crate) fn card(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.surface.into()),
        border: border(p.border_soft, 1.0, RADIUS_LG),
        shadow: soft_shadow(p, 2.0, 10.0),
        ..Default::default()
    }
}

pub(crate) fn floating(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.surface.into()),
        border: border(p.border_soft, 1.0, RADIUS_PILL),
        shadow: soft_shadow(p, 4.0, 16.0),
        ..Default::default()
    }
}

pub(crate) fn menu_panel(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.surface.into()),
        border: border(p.border_soft, 1.0, RADIUS_LG),
        shadow: soft_shadow(p, 8.0, 24.0),
        ..Default::default()
    }
}

pub(crate) fn dialog(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.surface.into()),
        text_color: Some(p.text),
        border: border(p.border_soft, 1.0, RADIUS_XL),
        shadow: soft_shadow(p, 12.0, 36.0),
    }
}

pub(crate) fn tooltip(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.text.into()),
        text_color: Some(p.surface),
        border: border(Color::TRANSPARENT, 0.0, RADIUS_SM),
        shadow: soft_shadow(p, 3.0, 10.0),
    }
}

pub(crate) fn sunken(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.sunken.into()),
        border: border(p.border_soft, 1.0, RADIUS_MD),
        ..Default::default()
    }
}

pub(crate) fn scrim(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette(theme).scrim.into()),
        ..Default::default()
    }
}

pub(crate) fn status_bar(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.bg.into()),
        text_color: Some(p.text_dim),
        ..Default::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tone {
    Neutral,
    Accent,
    Success,
    Danger,
    Warning,
}

impl Tone {
    fn colors(self, p: &Palette) -> (Color, Color) {
        match self {
            Tone::Neutral => (p.hover, p.text_dim),
            Tone::Accent => (p.accent_soft, p.accent_strong),
            Tone::Success => (p.success_soft, p.success),
            Tone::Danger => (p.danger_soft, p.danger),
            Tone::Warning => (p.warning_soft, p.warning),
        }
    }
}

pub(crate) fn chip(tone: Tone) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let (background, text_color) = tone.colors(palette(theme));
        container::Style { text_color: Some(text_color), ..pill(background) }
    }
}

pub(crate) fn family_dot(family: Family) -> impl Fn(&Theme) -> container::Style {
    move |theme| pill(family_color(palette(theme), family))
}

pub(crate) fn unsaved_dot(theme: &Theme) -> container::Style {
    pill(palette(theme).accent_strong)
}

pub(crate) fn toast(tone: Tone) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let p = palette(theme);
        let (_, stripe) = tone.colors(p);
        container::Style {
            background: Some(mix(p.surface, stripe, 0.10).into()),
            text_color: Some(p.text),
            border: border(mix(p.border_soft, stripe, 0.45), 1.0, RADIUS_LG),
            shadow: soft_shadow(p, 6.0, 20.0),
        }
    }
}

fn field_style(theme: &Theme, focused: bool, hovered: bool) -> text_input::Style {
    let p = palette(theme);
    let (stroke, width) = match (focused, hovered) {
        (true, _) => (p.accent, 1.5),
        (false, true) => (p.text_faint, 1.0),
        (false, false) => (p.border, 1.0),
    };
    text_input::Style {
        background: Background::Color(p.surface),
        border: border(stroke, width, RADIUS_MD),
        icon: p.text_dim,
        placeholder: p.text_faint,
        value: p.text,
        selection: p.selection,
    }
}

pub(crate) fn input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    field_style(
        theme,
        matches!(status, text_input::Status::Focused),
        matches!(status, text_input::Status::Hovered),
    )
}

pub(crate) fn editor(theme: &Theme, status: text_editor::Status) -> text_editor::Style {
    let style = field_style(
        theme,
        matches!(status, text_editor::Status::Focused),
        matches!(status, text_editor::Status::Hovered),
    );
    text_editor::Style {
        background: style.background,
        border: style.border,
        icon: style.icon,
        placeholder: style.placeholder,
        value: style.value,
        selection: style.selection,
    }
}

pub(crate) fn cell(focus: bool, spent: bool) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let p = palette(theme);
        let (background, stroke, text_color) = if focus {
            (p.accent_soft, p.accent, p.accent_strong)
        } else if spent {
            (p.sunken, p.border_soft, p.text_faint)
        } else {
            (p.surface, p.border, p.text)
        };
        container::Style {
            background: Some(background.into()),
            text_color: Some(text_color),
            border: border(stroke, if focus { 1.5 } else { 1.0 }, 6.0),
            ..Default::default()
        }
    }
}

pub(crate) fn scroll(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let p = palette(theme);
    let active = !matches!(status, scrollable::Status::Active);
    let rail = scrollable::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable::Scroller {
            color: if active { p.text_faint } else { p.border },
            border: border(Color::TRANSPARENT, 0.0, RADIUS_PILL),
        },
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
    }
}

pub(crate) fn divider(theme: &Theme) -> rule::Style {
    rule::Style {
        color: palette(theme).border_soft,
        width: 1,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
    }
}

pub(crate) fn scroll_subtle(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let mut style = scroll(theme, status);
    if matches!(status, scrollable::Status::Active) {
        style.horizontal_rail.scroller.color = Color::TRANSPARENT;
        style.vertical_rail.scroller.color = Color::TRANSPARENT;
    }
    style
}

pub(crate) fn shaping_for(content: &str) -> text::Shaping {
    let covered = content.chars().all(|c| {
        (c as u32) < 0x100 || (0x2000..=0x206F).contains(&(c as u32)) || c == '\u{2212}'
    });
    if covered {
        text::Shaping::Basic
    } else {
        text::Shaping::Advanced
    }
}
