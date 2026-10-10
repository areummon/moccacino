# Canvas editing

The user draws a machine on the canvas with JFLAP-style tools. They add states, connect them with labeled transitions, mark accepting and initial states, rename, move, and delete. The status bar counts states and transitions and shows determinism.

## Sub-features

- `edit-add-state`: the State tool adds `q0`, `q1`, … on empty clicks. A new state becomes initial only while the tab has no initial state. Names are never reused after a delete; only `Clear` restarts the numbering at `q0`.
- `edit-add-transition`: the Transition tool clicks a source, then a target, and the `New transition` dialog asks for a label (blank = ε).
- `edit-final`: Shift+click with Select toggles an accepting state (double circle).
- `edit-initial`: Alt+click with Select toggles the initial marker. Alt+click on a new state moves the marker there; Alt+click on the current initial state removes it and leaves the machine with no initial state (Save then fails with "has no initial state").
- `edit-rename`: double-click a state with Select to open `Rename state`.
- `edit-label`: click a transition's label with Select to open `Transition labels`: one row per label, `Add label`, a delete icon per row, `Save`. Saving with every row cleared removes the edge.
- `edit-move-pan`: drag a state to move it, or drag empty space to pan.
- `edit-delete`: the Delete tool (`4`, `Tab`, or hold `Delete`) removes the clicked state with its edges, or one edge. `Tab` toggles Delete↔Select; releasing a held `Delete` restores the tool active before the hold.
- `edit-clear`: the `Clear` button (top right of the canvas, at 1225 82; shown only while the tab has states) empties the tab, resets zoom, the run and the name counter, and keeps the machine family.

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
- **Toggle initial.** Run `$M click 750 350 alt` (marker moves to `q1`), then `$M click 750 350 alt` again: no state has the marker. Run `$M click 400 350 alt` to restore it on `q0`.
- **Rename and relabel.** Run `$M click 400 350 double`, `$M key ctrl+a`, `$M type s`, `$M key Return`: `q0` reads `s`. Run `$M click 575 334` (the `a` label pill): `Transition labels` opens with focus in the row. Run `$M key ctrl+a`, `$M type b`, `$M key Return`.
- **Delete.** Run `$M key 4` and `$M click 750 350`. `q1` and its edge disappear, and the status bar reads `1 state · 0 transitions`. `$M key Tab` returns to Select.
- **Clear.** Run `$M click 1225 82`, then `$M key 2` and `$M click 400 350`: the new state is `q0` again, with the initial arrow.

## Gotchas

- Shift and Alt clicks only count when the modifier is down before the click. `$M click … shift` handles this, but a raw `xdotool keydown shift click 1` silently does nothing.
- A hovered state draws a halo ring that looks like an accepting double circle. Park the pointer before you judge, or trust the saved `.ce` file.
- While a run is loaded, the active state glows, which also hides the accepting ring.
- Tool keys are inert while a dialog or text field has focus. Run `$M key Escape` first, or `$M click 640 600` only when Select is already active (with the State tool that click adds a state).
- The status bar's transition count counts labels, not edges.
- With the State tool, a click on an existing state, edge or label does not add a state. Pick empty spots at least about 70 px apart (states have a radius of about 30 px).
