/* Design tokens and reusable widget styles for the whole GUI. The look is
 * soft and warm: pastel accents (peach, sage, lavender, butter) over cream
 * paper in the light theme and warm cocoa in the dark one. Every color
 * lives in a `Palette`; style functions receive iced's `&Theme` and pick
 * the matching palette through `palette(theme)`, so switching the app
 * theme restyles everything without threading colors through the views.
 * Canvases (the state-machine editor, icons) resolve colors the same way
 * inside `draw`. */

use iced::font::Weight;
use iced::widget::{button, container, rule, scrollable, text, text_editor, text_input};
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

/* ---- Fonts ---- */

/* Rounded, friendly UI face; bundled in assets/fonts and registered in
 * main.rs. Symbols it lacks (ε, arrows) fall back per glyph. */
pub(crate) const UI: Font = Font::with_name("Nunito");
pub(crate) const SEMIBOLD: Font = Font { weight: Weight::Semibold, ..UI };
pub(crate) const BOLD: Font = Font { weight: Weight::Bold, ..UI };
/* Monospace for tapes, stacks, labels and generated code. */
pub(crate) const MONO: Font = Font::with_name("JetBrains Mono");

/* ---- Shape ---- */

pub(crate) const RADIUS_SM: f32 = 8.0;
pub(crate) const RADIUS_MD: f32 = 10.0;
pub(crate) const RADIUS_LG: f32 = 14.0;
pub(crate) const RADIUS_XL: f32 = 18.0;
pub(crate) const RADIUS_PILL: f32 = 999.0;

/* ---- Palettes ---- */

#[derive(Debug, Clone, Copy)]
pub(crate) struct Palette {
    /* Window background behind every surface. */
    pub(crate) bg: Color,
    /* Cards, header, dock, dialogs. */
    pub(crate) surface: Color,
    /* Resting fill of neutral buttons. */
    pub(crate) raised: Color,
    /* Hover fill of neutral/ghost controls. */
    pub(crate) hover: Color,
    /* Inputs and code boxes. */
    pub(crate) sunken: Color,
    pub(crate) border: Color,
    pub(crate) border_soft: Color,
    pub(crate) text: Color,
    pub(crate) text_dim: Color,
    pub(crate) text_faint: Color,
    pub(crate) accent: Color,
    pub(crate) accent_hover: Color,
    pub(crate) accent_soft: Color,
    /* Text drawn on an `accent` fill. */
    pub(crate) accent_text: Color,
    /* Text/strokes that sit on `accent_soft`. */
    pub(crate) accent_strong: Color,
    pub(crate) success: Color,
    pub(crate) success_soft: Color,
    pub(crate) danger: Color,
    pub(crate) danger_soft: Color,
    pub(crate) warning: Color,
    pub(crate) warning_soft: Color,
    pub(crate) shadow: Color,
    pub(crate) scrim: Color,
    pub(crate) selection: Color,
    /* State-machine canvas. */
    pub(crate) canvas_bg: Color,
    pub(crate) canvas_grid: Color,
    pub(crate) node_fill: Color,
    pub(crate) node_stroke: Color,
    pub(crate) edge: Color,
    pub(crate) label_bg: Color,
    /* One pastel per machine family (tab dots, node tint, picker cards). */
    pub(crate) family_finite: Color,
    pub(crate) family_pda: Color,
    pub(crate) family_tm: Color,
    pub(crate) family_grammar: Color,
}

/* Cream paper: linen background, warm cocoa text, peach accent. */
pub(crate) const LIGHT: Palette = Palette {
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

/* Warm cocoa: brown-charcoal surfaces, oat text, the same pastels a touch
 * quieter so they glow instead of shout. */
pub(crate) const DARK: Palette = Palette {
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

    /* The iced theme handed to the runtime: its base palette drives the
     * default text color and built-in widget fallbacks, and its darkness
     * is how `palette(theme)` finds its way back to our tokens. */
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

/* The palette behind an iced theme produced by `ThemeMode::iced_theme`. */
pub(crate) fn palette(theme: &Theme) -> &'static Palette {
    if theme.extended_palette().is_dark {
        &DARK
    } else {
        &LIGHT
    }
}

/* ---- Machine families ---- */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Finite,
    Pushdown,
    Turing,
    Grammar,
}

impl Family {
    pub(crate) const ALL: [Family; 4] =
        [Family::Finite, Family::Turing, Family::Pushdown, Family::Grammar];

    pub(crate) fn color(self, p: &Palette) -> Color {
        match self {
            Family::Finite => p.family_finite,
            Family::Pushdown => p.family_pda,
            Family::Turing => p.family_tm,
            Family::Grammar => p.family_grammar,
        }
    }

    pub(crate) fn title(self) -> &'static str {
        match self {
            Family::Finite => "Finite Automaton",
            Family::Pushdown => "Pushdown Automaton",
            Family::Turing => "Turing Machine",
            Family::Grammar => "Grammar",
        }
    }

    pub(crate) fn blurb(self) -> &'static str {
        match self {
            Family::Finite => "DFAs and NFAs, regex, minimization",
            Family::Pushdown => "Stack-powered context-free recognizers",
            Family::Turing => "Tapes, heads and multi-tape machines",
            Family::Grammar => "Context-free rules, CNF and derivations",
        }
    }
}

/* ---- Helpers ---- */

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

/* ---- Text ---- */

pub(crate) fn text_dim(theme: &Theme) -> text::Style {
    text::Style { color: Some(palette(theme).text_dim) }
}

pub(crate) fn text_faint(theme: &Theme) -> text::Style {
    text::Style { color: Some(palette(theme).text_faint) }
}

pub(crate) fn text_danger(theme: &Theme) -> text::Style {
    text::Style { color: Some(palette(theme).danger) }
}

/* ---- Buttons ---- */

/* Filled peach call to action (dialog confirm, Load, active Play). */
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

/* Neutral bordered button. */
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

/* Borderless button that only shows a fill on hover. */
pub(crate) fn button_ghost(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let (background, text_color) = match status {
        button::Status::Hovered | button::Status::Pressed => (p.hover, p.text),
        button::Status::Disabled => (Color::TRANSPARENT, p.text_faint),
        _ => (Color::TRANSPARENT, p.text),
    };
    button::Style {
        background: Some(background.into()),
        text_color,
        border: border(Color::TRANSPARENT, 0.0, RADIUS_SM),
        shadow: Shadow::default(),
    }
}

/* Filled destructive button (confirming a discard). */
pub(crate) fn button_danger(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let background = match status {
        button::Status::Hovered => mix(p.danger, p.text, 0.12),
        _ => p.danger,
    };
    button::Style {
        background: Some(background.into()),
        text_color: if theme.extended_palette().is_dark { p.bg } else { p.surface },
        border: border(Color::TRANSPARENT, 0.0, RADIUS_MD),
        shadow: Shadow::default(),
    }
}

/* Ghost variant for destructive actions (Clear canvas, remove label). */
pub(crate) fn button_danger_ghost(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => p.danger_soft,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(background.into()),
        text_color: p.danger,
        border: border(Color::TRANSPARENT, 0.0, RADIUS_SM),
        shadow: Shadow::default(),
    }
}

/* Header menu triggers: ghost buttons that stay tinted while their
 * dropdown is open. */
pub(crate) fn menu_trigger(open: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        let background = match status {
            _ if open => p.accent_soft,
            button::Status::Hovered | button::Status::Pressed => p.hover,
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(background.into()),
            text_color: if open { p.accent_strong } else { p.text },
            border: border(Color::TRANSPARENT, 0.0, RADIUS_SM),
            shadow: Shadow::default(),
        }
    }
}

/* Rows inside dropdown menus. */
pub(crate) fn menu_item(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let (background, text_color) = match status {
        button::Status::Hovered | button::Status::Pressed => (p.accent_soft, p.text),
        button::Status::Disabled => (Color::TRANSPARENT, p.text_faint),
        _ => (Color::TRANSPARENT, p.text),
    };
    button::Style {
        background: Some(background.into()),
        text_color,
        border: border(Color::TRANSPARENT, 0.0, RADIUS_SM),
        shadow: Shadow::default(),
    }
}

/* Header tabs: the active one is a raised card, the rest are quiet. */
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

/* Tool palette entries and other on/off pills. */
pub(crate) fn toggle_pill(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let (background, text_color) = match (active, hovered) {
            (true, _) => (p.accent_soft, p.accent_strong),
            (false, true) => (p.hover, p.text),
            (false, false) => (Color::TRANSPARENT, p.text_dim),
        };
        button::Style {
            background: Some(background.into()),
            text_color,
            border: border(Color::TRANSPARENT, 0.0, RADIUS_PILL),
            shadow: Shadow::default(),
        }
    }
}

/* Startup picker cards; `selected` mirrors the keyboard highlight. */
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

/* Tiny round close button inside tab pills. */
pub(crate) fn tab_close(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let (background, text_color) = match status {
        button::Status::Hovered | button::Status::Pressed => (p.danger_soft, p.danger),
        _ => (Color::TRANSPARENT, p.text_faint),
    };
    button::Style {
        background: Some(background.into()),
        text_color,
        border: border(Color::TRANSPARENT, 0.0, RADIUS_PILL),
        shadow: Shadow::default(),
    }
}

/* ---- Containers ---- */

pub(crate) fn app_background(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.bg.into()),
        text_color: Some(p.text),
        ..Default::default()
    }
}

pub(crate) fn header(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.bg.into()),
        ..Default::default()
    }
}

/* Rounded surface card with a gentle lift (dock, grammar cards). */
pub(crate) fn card(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.surface.into()),
        border: border(p.border_soft, 1.0, RADIUS_LG),
        shadow: soft_shadow(p, 2.0, 10.0),
        ..Default::default()
    }
}

/* Floating overlays on the canvas (tool palette, zoom chip). */
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

/* Recessed box for code, derivations, tapes and lanes. */
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

/* Pastel chip; `tone` picks the fill/text pair. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tone {
    Neutral,
    Accent,
    Success,
    Danger,
    Warning,
}

pub(crate) fn chip(tone: Tone) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let p = palette(theme);
        let (background, text_color) = match tone {
            Tone::Neutral => (p.hover, p.text_dim),
            Tone::Accent => (p.accent_soft, p.accent_strong),
            Tone::Success => (p.success_soft, p.success),
            Tone::Danger => (p.danger_soft, p.danger),
            Tone::Warning => (p.warning_soft, p.warning),
        };
        container::Style {
            background: Some(background.into()),
            text_color: Some(text_color),
            border: border(Color::TRANSPARENT, 0.0, RADIUS_PILL),
            ..Default::default()
        }
    }
}

/* Small colored dot marking a machine family. */
pub(crate) fn family_dot(family: Family) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let p = palette(theme);
        container::Style {
            background: Some(family.color(p).into()),
            border: border(Color::TRANSPARENT, 0.0, RADIUS_PILL),
            ..Default::default()
        }
    }
}

/* Accent dot marking a tab with unsaved changes. */
pub(crate) fn unsaved_dot(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette(theme).accent_strong.into()),
        border: border(Color::TRANSPARENT, 0.0, RADIUS_PILL),
        ..Default::default()
    }
}

/* Round badge behind a family glyph (startup picker cards). */
pub(crate) fn family_badge(family: Family) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let p = palette(theme);
        container::Style {
            background: Some(mix(p.surface, family.color(p), 0.55).into()),
            border: border(Color::TRANSPARENT, 0.0, RADIUS_PILL),
            ..Default::default()
        }
    }
}

pub(crate) fn toast(tone: Tone) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let p = palette(theme);
        let stripe = match tone {
            Tone::Neutral => p.border,
            Tone::Accent => p.accent,
            Tone::Success => p.success,
            Tone::Danger => p.danger,
            Tone::Warning => p.warning,
        };
        container::Style {
            background: Some(mix(p.surface, stripe, 0.10).into()),
            text_color: Some(p.text),
            border: border(mix(p.border_soft, stripe, 0.45), 1.0, RADIUS_LG),
            shadow: soft_shadow(p, 6.0, 20.0),
        }
    }
}

/* Cells of tapes, input ribbons and stacks. `focus` marks the head / next
 * symbol / stack top, `spent` dims consumed input. */
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

/* ---- Inputs ---- */

pub(crate) fn input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let p = palette(theme);
    let (stroke, width) = match status {
        text_input::Status::Focused => (p.accent, 1.5),
        text_input::Status::Hovered => (p.text_faint, 1.0),
        _ => (p.border, 1.0),
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

pub(crate) fn editor(theme: &Theme, status: text_editor::Status) -> text_editor::Style {
    let p = palette(theme);
    let (stroke, width) = match status {
        text_editor::Status::Focused => (p.accent, 1.5),
        text_editor::Status::Hovered => (p.text_faint, 1.0),
        _ => (p.border, 1.0),
    };
    text_editor::Style {
        background: Background::Color(p.surface),
        border: border(stroke, width, RADIUS_MD),
        icon: p.text_dim,
        placeholder: p.text_faint,
        value: p.text,
        selection: p.selection,
    }
}

/* Slim, rounded scrollbars that stay out of the way. */
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

/* Scrollbars that only appear while the pointer is over the area (tab
 * strip), so they never read as an underline. */
pub(crate) fn scroll_subtle(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let mut style = scroll(theme, status);
    if matches!(status, scrollable::Status::Active) {
        style.horizontal_rail.scroller.color = Color::TRANSPARENT;
        style.vertical_rail.scroller.color = Color::TRANSPARENT;
    }
    style
}
