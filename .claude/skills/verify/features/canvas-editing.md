# Canvas editing

The user draws a machine on the canvas with JFLAP-style tools. They add states, connect them with labeled transitions, mark accepting and initial states, rename, move, and delete. The status bar counts states and transitions and shows determinism.

## Sub-features

- `edit-add-state`: the State tool adds `q0`, `q1`, … on empty clicks. A new state becomes initial only when it is the tab's only state and no initial state is set; deleting the initial state while others remain leaves the tab without one until Alt+click. Names are never reused after a delete; only `Clear` restarts the numbering at `q0`.
- `edit-add-transition`: the Transition tool clicks a source, then a target, and the `New transition` dialog asks for a label (blank = ε).
- `edit-final`: Shift+click with Select toggles an accepting state (double circle).
- `edit-initial`: Alt+click with Select makes a state initial (arrow marker). It never clears the marker: Alt+click on the current initial state changes nothing.
- `edit-rename`: double-click a state with Select to open `Rename state`.
- `edit-label`: click a transition's label with Select to open `Transition labels`: one row per label, `Add label`, a delete icon per row, `Save`. Saving with every row cleared removes the edge.
- `edit-move-pan`: drag a state to move it, or drag empty space to pan.
- `edit-delete`: the Delete tool (`4`, `Tab`, or hold `Delete`) removes the clicked state with its edges, or one edge. `Tab` toggles Delete↔Select; releasing a held `Delete` restores the tool active before the hold.
- `edit-clear`: the `Clear` button (top right of the canvas, at 1220 87, level with the tool palette, right of the undo and redo arrows; shown while the tab has states or transitions) empties the tab, resets zoom, the run, the name counter and the tool (back to Select), and keeps the machine family. A toast names the undo shortcut.
- `edit-select`: with Select, `ctrl`+click toggles a state in the selection and `ctrl`+drag on empty space selects every state whose center is inside the rectangle; a plain click on a state selects only it, a plain click on empty space or `Escape` clears. `ctrl+a` selects all. Selected states carry an accent ring and the status bar shows `N selected`. Dragging a selected state moves the whole selection (one undo step). `Delete` with a selection deletes it (one undo step) instead of arming the hold-delete tool.
- `edit-context-menu`: right-click a state for `Rename…`, `Mark accepting`/`Unmark accepting`, `Make initial` (disabled on the initial state) and `Delete state`; with two or more states selected the menu is the single item `Delete selected states`. Right-click a transition for `Edit labels…` and `Delete transition`. The menu opens at the pointer, works with every tool, and closes on `Escape` or a click outside. Right-clicking an unselected state selects it.
- `edit-undo`: `ctrl+z` steps back one canvas edit and `ctrl+shift+z` or `ctrl+y` steps forward. Every edit is one step: add, delete, rename, relabel, accepting, initial, a whole drag, and `Clear`. The arrows left of `Clear` (1131 87 and 1163 87) do the same and are faint when there is nothing to step to. History is per tab, and a loaded or generated tab starts with none.

## How to get to it (user POV)

- Pick a module on the startup picker, or **New → <family>**.
- Use the floating tool palette (Select / State / Transition / Delete, at y≈86) or the keys `1`–`4`.

## Driving it with moca

Preconditions:

- A fresh `$M up` shows the startup picker.

- **Open a finite tab.** Press `Return` (Finite is preselected). Run `$M key Return`. The `Automaton` tab and an empty canvas appear.
- **Add states.** Run `$M key 2`, then `$M click 400 350` and `$M click 750 350`. States `q0` (with the initial arrow) and `q1` appear, and the status bar reads `2 states · 0 transitions`.
- **Add transition.** Run `$M key 3`, then `$M click 400 350` and `$M click 750 350`. The `New transition` dialog opens with focus in `Label`. Run `$M type a` and `$M key Return`. An edge labeled `a` appears, and the status bar reads `1 transition`.
- **Mark accepting.** Run `$M key Escape` and `$M click 750 350 shift`. Run `$M move 1000 200`, then `$M shot dfa`. `q1` is drawn as a double circle.
- **Proof (side effect).** Run `$M key ctrl+s`, `$M key ctrl+a`, `$M type "$EVID/edit.ce"`, and `$M key Return`. Then run `cat "$EVID/edit.ce"`. It shows `states: q0, q1`, `transitions: (q0, a) -> q1`, `initial: q0`, and `final: q1`.
- **Move initial.** Run `$M click 750 350 alt` (marker moves to `q1`), then `$M click 750 350 alt` again: `q1` keeps it. Run `$M click 400 350 alt` to move it back to `q0`.
- **Rename and relabel.** Run `$M click 400 350 double`, `$M key ctrl+a`, `$M type s`, `$M key Return`: `q0` reads `s`. Run `$M click 575 334` (the `a` label pill): `Transition labels` opens with focus in the row. Run `$M key ctrl+a`, `$M type b`, `$M key Return`.
- **Delete.** Run `$M key 4` and `$M click 750 350`. `q1` and its edge disappear, and the status bar reads `1 state · 0 transitions`. `$M key Tab` returns to Select. Adding a state now (`$M key 2`, `$M click 400 500`) gives `q2` with no initial arrow while `s` (the old `q0`) is still initial.
- **Clear.** Run `$M click 1220 87`, then `$M key 2` and `$M click 400 350`: the new state is `q0` again, with the initial arrow.
- **Select and move.** With states at 300 300, 500 300 and 500 480: `$M drag 220 220 600 380 ctrl` rings the first two and the status bar reads `2 selected`. `$M click 500 480 ctrl` adds the third. `$M drag 300 300 360 400` moves all three by the same offset; `$M key ctrl+z` puts them back.
- **Context menu.** `$M click 500 300 right`, then `$M shot menu` and click an item (the panel is 288 px wide and opens with its top-left corner at the pointer; for one selected state the rows sit about 22, 56, 90 and 135 px below the pointer: Rename 560 322, accepting 560 356, initial 560 390, Delete 560 435 for a click at 500 300. On the initial state the disabled `Make initial` row grows an `Already the initial state` subtitle and pushes Delete down about 15 px. Near the right or bottom edge the panel is pulled back inside the canvas, so shoot it first). `$M key ctrl+a`, `$M click 500 300 right` shows only `Delete selected states`.
- **Undo and redo.** Run `$M key ctrl+z` twice: the new state goes, then the cleared machine comes back. `$M key ctrl+shift+z` clears it again and `$M key ctrl+z` restores it. `$M shot` after each step.

## Gotchas

- Shift and Alt clicks only count when the modifier is down before the click. `$M click … shift` handles this, but a raw `xdotool keydown shift click 1` silently does nothing.
- A hovered state draws a halo ring that looks like an accepting double circle. Park the pointer before you judge, or trust the saved `.ce` file.
- A selected state draws an accent ring too, and any plain click on a state selects it (the status bar shows `1 selected`). `$M key Escape` clears the selection before a proof shot.
- While a run is loaded, the active state glows and its accepting ring turns accent-colored, so it is harder to spot.
- Undo and redo are inert while a dialog is open and on grammar tabs. A drag becomes an undo step at the next key press or click, not on mouse release.
- Tool keys are inert while a dialog or text field has focus. Run `$M key Escape` first, or `$M click 40 640` only when Select is already active (with the State tool that click adds a state).
- The status bar's transition count counts labels, not edges.
- With the State tool, a click on an existing state, edge or label does not add a state. Pick empty spots at least about 70 px apart (states have a radius of about 30 px).
