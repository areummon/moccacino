use iced::keyboard::{key::Named, Key, Modifiers};

use crate::state_machine::EditorTool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Section {
    Canvas,
    ViewAndRun,
    App,
}

impl Section {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Section::Canvas => "CANVAS",
            Section::ViewAndRun => "VIEW & RUN",
            Section::App => "APP",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    Tool(EditorTool),
    ToggleDelete,
    HoldDelete,
    Escape,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    Fit,
    PlayPause,
    Step,
    NewTabMenu,
    CloseTab,
    PreviousTab,
    NextTab,
    Open,
    Save,
    ToggleTheme,
    Shortcuts,
    ParseGrammar,
    StartupMove(i32),
    StartupChoose,
}

impl Action {
    pub(crate) fn needs_canvas(self) -> bool {
        matches!(
            self,
            Action::Tool(_) | Action::ToggleDelete | Action::HoldDelete | Action::Fit | Action::PlayPause | Action::Step
        )
    }
}

#[derive(Debug, Clone, Copy)]
enum Keycap {
    Char(&'static str),
    Named(Named),
    Mouse(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Chord {
    ctrl: bool,
    shift: bool,
    key: Keycap,
}

const fn key(key: Keycap) -> Chord {
    Chord { ctrl: false, shift: false, key }
}

const fn ctrl(key: Keycap) -> Chord {
    Chord { ctrl: true, shift: false, key }
}

const fn ctrl_shift(key: Keycap) -> Chord {
    Chord { ctrl: true, shift: true, key }
}

impl Chord {
    pub(crate) fn caps(self) -> Vec<String> {
        let mut caps = Vec::new();
        if self.ctrl {
            caps.push("Ctrl".to_string());
        }
        if self.shift {
            caps.push("Shift".to_string());
        }
        caps.extend(match self.key {
            Keycap::Char("-") => vec!["−".to_string()],
            Keycap::Char("=") => vec!["+".to_string()],
            Keycap::Char(c) => vec![c.to_uppercase()],
            Keycap::Named(named) => vec![named_cap(named).to_string()],
            Keycap::Mouse(words) => words.split(' ').map(str::to_string).collect(),
        });
        caps
    }

    fn matches(self, pressed: &Key, modifiers: Modifiers) -> bool {
        if self.ctrl != modifiers.command() || (self.shift && !modifiers.shift()) {
            return false;
        }
        match (self.key, pressed.as_ref()) {
            (Keycap::Char(expected), Key::Character(typed)) => typed.to_lowercase() == expected,
            (Keycap::Named(expected), Key::Named(named)) => expected == named,
            _ => false,
        }
    }
}

fn named_cap(named: Named) -> &'static str {
    match named {
        Named::Tab => "Tab",
        Named::Delete => "Del",
        Named::Escape => "Esc",
        Named::Space => "Space",
        Named::ArrowLeft => "←",
        Named::ArrowRight => "→",
        Named::ArrowUp => "↑",
        Named::ArrowDown => "↓",
        Named::Enter => "Enter",
        Named::F1 => "F1",
        _ => "?",
    }
}

pub(crate) struct Shortcut {
    pub(crate) section: Section,
    pub(crate) chords: &'static [Chord],
    pub(crate) label: &'static str,
    pub(crate) action: Option<Action>,
}

const fn entry(section: Section, chords: &'static [Chord], label: &'static str, action: Action) -> Shortcut {
    Shortcut { section, chords, label, action: Some(action) }
}

const fn gesture(section: Section, chords: &'static [Chord], label: &'static str) -> Shortcut {
    Shortcut { section, chords, label, action: None }
}

pub(crate) const SHORTCUTS: &[Shortcut] = &[
    entry(Section::Canvas, &[key(Keycap::Char("1"))], "Select / move tool", Action::Tool(EditorTool::Arrow)),
    entry(Section::Canvas, &[key(Keycap::Char("2"))], "State tool", Action::Tool(EditorTool::State)),
    entry(Section::Canvas, &[key(Keycap::Char("3"))], "Transition tool", Action::Tool(EditorTool::Transition)),
    entry(Section::Canvas, &[key(Keycap::Char("4"))], "Delete tool", Action::Tool(EditorTool::Delete)),
    entry(Section::Canvas, &[key(Keycap::Named(Named::Tab))], "Toggle Select ↔ Delete", Action::ToggleDelete),
    entry(Section::Canvas, &[key(Keycap::Named(Named::Delete))], "Hold for a quick delete", Action::HoldDelete),
    gesture(Section::Canvas, &[key(Keycap::Mouse("Shift click"))], "Toggle accepting state"),
    gesture(Section::Canvas, &[key(Keycap::Mouse("Alt click"))], "Make initial state"),
    entry(Section::Canvas, &[key(Keycap::Named(Named::Escape))], "Back to Select", Action::Escape),
    gesture(Section::ViewAndRun, &[key(Keycap::Mouse("Ctrl wheel"))], "Zoom around the cursor"),
    entry(Section::ViewAndRun, &[ctrl(Keycap::Char("+")), ctrl(Keycap::Char("="))], "Zoom in", Action::ZoomIn),
    entry(Section::ViewAndRun, &[ctrl(Keycap::Char("-"))], "Zoom out", Action::ZoomOut),
    entry(Section::ViewAndRun, &[ctrl(Keycap::Char("0"))], "Reset zoom", Action::ZoomReset),
    entry(Section::ViewAndRun, &[key(Keycap::Char("f"))], "Fit the machine to the view", Action::Fit),
    entry(Section::ViewAndRun, &[key(Keycap::Named(Named::Space))], "Play / pause the run", Action::PlayPause),
    entry(Section::ViewAndRun, &[key(Keycap::Named(Named::ArrowRight))], "Step the run", Action::Step),
    entry(Section::App, &[ctrl(Keycap::Char("t"))], "New tab", Action::NewTabMenu),
    entry(Section::App, &[ctrl(Keycap::Char("w"))], "Close tab", Action::CloseTab),
    entry(Section::App, &[ctrl_shift(Keycap::Named(Named::Tab))], "Previous tab", Action::PreviousTab),
    entry(Section::App, &[ctrl(Keycap::Named(Named::Tab))], "Next tab", Action::NextTab),
    entry(Section::App, &[ctrl(Keycap::Char("o"))], "Open a .ce file", Action::Open),
    entry(Section::App, &[ctrl(Keycap::Char("s"))], "Save the tab as .ce", Action::Save),
    entry(Section::App, &[ctrl_shift(Keycap::Char("l"))], "Light / dark theme", Action::ToggleTheme),
    entry(Section::App, &[ctrl(Keycap::Named(Named::Enter))], "Parse the grammar (in its editor)", Action::ParseGrammar),
    entry(Section::App, &[key(Keycap::Named(Named::F1)), key(Keycap::Char("?"))], "This cheat sheet", Action::Shortcuts),
];

pub(crate) const STARTUP_SHORTCUTS: &[Shortcut] = &[
    entry(Section::App, &[key(Keycap::Named(Named::ArrowLeft))], "Previous module", Action::StartupMove(-1)),
    entry(Section::App, &[key(Keycap::Named(Named::ArrowRight))], "Next module", Action::StartupMove(1)),
    entry(Section::App, &[key(Keycap::Named(Named::ArrowUp))], "Module above", Action::StartupMove(-2)),
    entry(Section::App, &[key(Keycap::Named(Named::ArrowDown))], "Module below", Action::StartupMove(2)),
    entry(Section::App, &[key(Keycap::Named(Named::Enter))], "Open the chosen module", Action::StartupChoose),
    entry(Section::App, &[key(Keycap::Named(Named::Escape))], "Quit", Action::Escape),
];

pub(crate) fn find(table: &[Shortcut], pressed: &Key, modifiers: Modifiers) -> Option<(Action, bool)> {
    table.iter().find_map(|shortcut| {
        let chord = shortcut.chords.iter().find(|chord| chord.matches(pressed, modifiers))?;
        Some((shortcut.action?, chord.ctrl))
    })
}

fn shortcut_for(action: Action) -> Option<&'static Shortcut> {
    SHORTCUTS.iter().chain(STARTUP_SHORTCUTS).find(|shortcut| shortcut.action == Some(action))
}

pub(crate) fn matches(action: Action, pressed: &Key, modifiers: Modifiers) -> bool {
    shortcut_for(action).is_some_and(|shortcut| shortcut.chords.iter().any(|chord| chord.matches(pressed, modifiers)))
}

pub(crate) fn hint(action: Action) -> String {
    shortcut_for(action)
        .and_then(|shortcut| shortcut.chords.first())
        .map(|chord| chord.caps().join("+"))
        .unwrap_or_default()
}
