/* JFLAP-style canvas editing tools. One active tool per tab decides what a
 * mouse press on the canvas means — select/move, create states, connect
 * states, or delete — instead of one overloaded click model guarded by
 * modifier keys. The active tool lives on the canvas `State` and is
 * switched from the tool bar (below the menu bar) or the keyboard. */

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum EditorTool {
    /* Select/edit: drag states, drag empty space pans, Shift+click toggles
     * final, Alt+click toggles initial, double-click renames, clicking a
     * transition opens its label editor. */
    #[default]
    Arrow,
    /* Click empty canvas creates a state at the cursor. */
    State,
    /* Click a source state, then a target state, to request a new
     * transition label. */
    Transition,
    /* Click a state or transition to remove it. */
    Delete,
}

impl EditorTool {
    /* Toolbar caption including the number-key shortcut. */
    pub(crate) fn label(self) -> &'static str {
        match self {
            EditorTool::Arrow => "↖ Select  1",
            EditorTool::State => "◯ State  2",
            EditorTool::Transition => "→ Transition  3",
            EditorTool::Delete => "× Delete  4",
        }
    }

    /* One-line description of the tool's interactions, shown as the
     * toolbar button's tooltip. */
    pub(crate) fn tooltip(self) -> &'static str {
        match self {
            EditorTool::Arrow => {
                "Drag to move states · drag empty space to pan · double-click a state to \
                 rename · Shift+click: final · Alt+click: initial · click a transition to \
                 edit its labels"
            }
            EditorTool::State => "Click empty canvas to create a state here",
            EditorTool::Transition => {
                "Click the source state, then the target state, to add a transition \
                 (click empty space to cancel)"
            }
            EditorTool::Delete => "Click a state or a transition to remove it",
        }
    }
}
