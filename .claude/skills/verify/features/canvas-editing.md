# Canvas editing

The user draws a machine on the canvas with JFLAP-style tools. They add states, connect them with labeled transitions, mark accepting and initial states, rename, move, and delete. The status bar counts states and transitions and shows determinism.

## Sub-features

- `edit-add-state`: the State tool adds `q0`, `q1`, … on empty clicks. The first state is initial.
- `edit-add-transition`: the Transition tool clicks a source, then a target, and the `New transition` dialog asks for a label (blank = ε).
- `edit-final`: Shift+click with Select toggles an accepting state (double circle).
- `edit-initial`: Alt+click with Select makes a state initial (arrow marker).
- `edit-rename`: double-click a state with Select to rename it.
- `edit-label`: click a transition with Select to open its label editor.
- `edit-move-pan`: drag a state to move it, or drag empty space to pan.
- `edit-delete`: the Delete tool (`4`, `Tab`, or hold `Delete`) removes the clicked state or edge.
- `edit-clear`: the `Clear` button (top right of the canvas) empties the tab and keeps the machine family.

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
- **Delete.** Run `$M key 4` and `$M click 750 350`. `q1` and its edge disappear, and the status bar reads `1 state`.

## Gotchas

- Shift and Alt clicks only count when the modifier is down before the click. `$M click … shift` handles this, but a raw `xdotool keydown shift click 1` silently does nothing.
- A hovered state draws a halo ring that looks like an accepting double circle. Park the pointer before you judge, or trust the saved `.ce` file.
- While a run is loaded, the active state glows, which also hides the accepting ring.
- Tool keys are inert while a dialog or text field has focus. Run `$M click 640 600` (empty canvas, Select tool) or `$M key Escape` first.
- With the State tool, a click on an existing state does not add a state. Pick empty spots at least about 70 px apart (states have a radius of about 30 px).
