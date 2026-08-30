/* Central palette and reusable widget styles for the whole GUI. Every
 * color lives here so the scheme stays consistent and editable in one
 * place; the three brand colors are tan #c0b490 (text and strokes), aqua
 * #abf5ed (primary accent) and deep blue #003e83 (secondary accent), on
 * neutral Notion-style dark gray surfaces. Supporting colors stay in the
 * same mood: dimmed tan for secondary text and a terracotta red family
 * for destructive/error states. */

use iced::widget::{button, container, scrollable, text_input};
use iced::{Background, Border, Color};

const fn hex(rgb: u32) -> Color {
    Color::from_rgb(
        ((rgb >> 16) & 0xff) as f32 / 255.0,
        ((rgb >> 8) & 0xff) as f32 / 255.0,
        (rgb & 0xff) as f32 / 255.0,
    )
}

/* #c0b490 — primary text, labels, canvas strokes. */
pub const CREAM: Color = hex(0xc0b490);
/* #abf5ed — primary accent: active tab, primary actions, highlights. */
pub const GREEN: Color = hex(0xabf5ed);
/* Brand blue #003e83 lifted for hover fills, tape head, selection — dark
 * enough for tan text, bright enough to stand out over the surfaces. */
pub const TEAL: Color = hex(0x0a55b8);
/* State circles: the brand blue exactly, so the tan stroke and label read
 * against the canvas while staying clearly darker than the tan. */
pub const NODE_FILL: Color = hex(0x003e83);

/* Neutral, Notion-style dark gray surfaces. */
pub const BG: Color = hex(0x191919);
pub const PANEL: Color = hex(0x202020);
pub const RAISED: Color = hex(0x2a2a2a);
pub const RAISED_HOVER: Color = hex(0x383838);
pub const INSET: Color = hex(0x161616);
pub const BORDER: Color = hex(0x3d3d3d);

/* Text tiers (dimmed tan). */
pub const TEXT_DIM: Color = hex(0xa99c7c);
pub const TEXT_FAINT: Color = hex(0x837b62);

/* Semantic colors. */
pub const ACCEPT: Color = hex(0xd8fbf5);
/* Warm terracotta family (hue ~13°) bridging the tan's warmth: bright
 * enough for error text on navy, dark enough for tan text on buttons. */
pub const REJECT: Color = hex(0xe5826a);
pub const DANGER: Color = hex(0xa84a33);
pub const DANGER_HOVER: Color = hex(0xc05c42);
pub const SCRIM: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.55);

/* ---- Buttons ---- */

/* Top-level bar buttons: transparent until hovered; `open` highlights
 * the button whose dropdown menu is currently shown. */
pub fn bar_button(open: bool, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => RAISED_HOVER,
        _ if open => RAISED,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(background.into()),
        text_color: CREAM,
        border: Border {
            color: GREEN,
            width: if open { 1.0 } else { 0.0 },
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/* Items inside a dropdown menu. */
pub fn menu_item(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => TEAL,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(background.into()),
        text_color: CREAM,
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/* Keyboard-highlighted row of a chooser list (startup picker): the brand
 * green fill marks the row Enter will open. */
pub fn menu_item_selected(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => ACCEPT,
        _ => GREEN,
    };
    button::Style {
        background: Some(background.into()),
        text_color: BG,
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/* Green call-to-action (Play when running, submit-style buttons). */
pub fn primary_button(status: button::Status) -> button::Style {
    let (background, text_color) = match status {
        button::Status::Hovered | button::Status::Pressed => (ACCEPT, BG),
        _ => (GREEN, BG),
    };
    button::Style {
        background: Some(background.into()),
        text_color,
        border: Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/* Neutral toolbar/panel buttons. */
pub fn secondary_button(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => RAISED_HOVER,
        _ => RAISED,
    };
    button::Style {
        background: Some(background.into()),
        text_color: CREAM,
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/* Destructive buttons (Clear, close ×). */
pub fn danger_button(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => DANGER_HOVER,
        _ => DANGER,
    };
    button::Style {
        background: Some(background.into()),
        text_color: CREAM,
        border: Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/* Tabs: the active one wears the brand green. */
pub fn tab_button(active: bool, status: button::Status) -> button::Style {
    let background = match (active, status) {
        (true, _) => GREEN,
        (false, button::Status::Hovered | button::Status::Pressed) => RAISED_HOVER,
        (false, _) => PANEL,
    };
    button::Style {
        background: Some(background.into()),
        text_color: if active { BG } else { CREAM },
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: iced::border::Radius {
                top_left: 6.0.into(),
                top_right: 6.0.into(),
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/* ---- Containers ---- */

pub fn panel_box() -> container::Style {
    container::Style {
        background: Some(PANEL.into()),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn inset_box() -> container::Style {
    container::Style {
        background: Some(INSET.into()),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub fn menu_panel() -> container::Style {
    container::Style {
        background: Some(PANEL.into()),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

pub fn dialog_box() -> container::Style {
    container::Style {
        background: Some(PANEL.into()),
        border: Border {
            color: GREEN,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

/* Dimmed layer behind modal dialogs. */
pub fn scrim() -> container::Style {
    container::Style {
        background: Some(SCRIM.into()),
        ..Default::default()
    }
}

/* ---- Inputs ---- */

pub fn input(status: text_input::Status) -> text_input::Style {
    let (border_color, border_width) = match status {
        text_input::Status::Focused => (GREEN, 1.5),
        text_input::Status::Hovered => (BORDER, 1.0),
        _ => (BORDER, 1.0),
    };
    text_input::Style {
        background: Background::Color(INSET),
        border: Border {
            color: border_color,
            width: border_width,
            radius: 6.0.into(),
        },
        icon: CREAM,
        placeholder: TEXT_FAINT,
        value: CREAM,
        selection: Color { a: 0.35, ..TEAL },
    }
}

pub fn editor(status: iced::widget::text_editor::Status) -> iced::widget::text_editor::Style {
    let (border_color, border_width) = match status {
        iced::widget::text_editor::Status::Focused => (GREEN, 1.5),
        iced::widget::text_editor::Status::Hovered => (BORDER, 1.0),
        _ => (BORDER, 1.0),
    };
    iced::widget::text_editor::Style {
        background: Background::Color(INSET),
        border: Border {
            color: border_color,
            width: border_width,
            radius: 6.0.into(),
        },
        icon: CREAM,
        placeholder: TEXT_FAINT,
        value: CREAM,
        selection: Color { a: 0.35, ..TEAL },
    }
}

/* Discreet scrollbars for dropdown menus. */
pub fn menu_scroll() -> scrollable::Style {
    let rail = scrollable::Rail {
        background: Some(Color::TRANSPARENT.into()),
        border: Border::default(),
        scroller: scrollable::Scroller {
            color: BORDER,
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
        },
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
    }
}