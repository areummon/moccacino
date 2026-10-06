
use crate::gui::icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum EditorTool {
    #[default]
    Arrow,
    State,
    Transition,
    Delete,
}

impl EditorTool {
    pub(crate) const ALL: [EditorTool; 4] =
        [EditorTool::Arrow, EditorTool::State, EditorTool::Transition, EditorTool::Delete];

    pub(crate) fn name(self) -> &'static str {
        match self {
            EditorTool::Arrow => "Select",
            EditorTool::State => "State",
            EditorTool::Transition => "Transition",
            EditorTool::Delete => "Delete",
        }
    }

    pub(crate) fn icon(self) -> Icon {
        match self {
            EditorTool::Arrow => Icon::Select,
            EditorTool::State => Icon::State,
            EditorTool::Transition => Icon::Transition,
            EditorTool::Delete => Icon::Delete,
        }
    }

    pub(crate) fn shortcut(self) -> &'static str {
        match self {
            EditorTool::Arrow => "1",
            EditorTool::State => "2",
            EditorTool::Transition => "3",
            EditorTool::Delete => "4",
        }
    }

    pub(crate) fn tooltip(self) -> &'static str {
        match self {
            EditorTool::Arrow => {
                "Drag to move states · drag empty space to pan · double-click a state to \
                 rename · Shift+click: final · Alt+click: initial · click a transition to \
                 edit its labels"
            }
            EditorTool::State => "Click empty canvas to create a state there",
            EditorTool::Transition => {
                "Click the source state, then the target state, to add a transition \
                 (click empty space to cancel)"
            }
            EditorTool::Delete => "Click a state or a transition to remove it (hold Del for a quick delete)",
        }
    }

    pub(crate) fn hint(self) -> &'static str {
        match self {
            EditorTool::Arrow => "Drag states · drag space to pan · double-click to rename",
            EditorTool::State => "Click anywhere to place a state",
            EditorTool::Transition => "Click a source state, then its target",
            EditorTool::Delete => "Click a state or transition to remove it",
        }
    }
}
