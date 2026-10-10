# moccacino verification map

This directory is the maintained source for verifying moccacino's user-facing behavior. Read this index before driving the app, then use the matching feature file as the recipe. All commands use `M=.claude/skills/verify/scripts/moca` from the repo root (see `../SKILL.md`).

## Baseline preconditions

- `$M up` started this instance, and `$M doctor` reports `healthy`.
- The app shows the startup picker (fresh `up`), unless a recipe says otherwise. The picker owns the keyboard, so every shortcut except arrows, `Return` and `Escape` (which quits the app) does nothing until a module is chosen.
- `MOCA_VERIFY_ROOT` points at a scratch directory, and `EVID=$MOCA_VERIFY_ROOT/evidence/${MOCA_ID:-default}`.
- Never drive an instance that this verification run did not start.

## Driving conventions

- Coordinates are window-relative pixels of the 1280×820 window.
- Prefer keys over coordinates: tool keys `1`–`4`, `ctrl+o`/`ctrl+s`, `ctrl+Tab`, `Return` in fields.
- `shot` an open menu before you click its lower items, because their positions move.
- Click empty canvas (`$M click 640 600`) to move focus off a text field before you use the single-key shortcuts.
- Park the pointer (`$M move 1000 200`) before a proof shot.

## Proof and skip reporting

- Capture a shot before and after the action that matters.
- Back a pixel claim with a side effect wherever one exists: a saved `.ce` file (`cat`), the clipboard (`$M clip name`), or the dock badge or toast text.
- Record the feature ID and entry point with `$M note "<id> via <entry>"`.
- Report an unreachable entry point with the command you tried. Do not count it as verified through another path.

## Feature entry contract

Each feature file starts with an H1 and a paragraph that describes the behavior. It then has four H2 sections: `Sub-features`, `How to get to it (user POV)`, `Driving it with moca` (starts with `Preconditions:`), and `Gotchas`.

## Features

- [Canvas editing](./canvas-editing.md): add states and transitions, set accepting and initial states, rename, move, delete, and clear.
- [Run panel](./run-panel.md): load, step, play, and the verdict for finite, pushdown, and Turing machines.
- [Operations](./operations.md): Check input, NFA → DFA, Minimize DFA, and Build from regex.
- [Files (.ce)](./files.md): load a multi-entity `.ce` file and save a tab.
- [Grammar tab](./grammar.md): parse, CYK check, leftmost derivation, and To CNF.
- [Exports](./exports.md): LaTeX/TikZ, regular expression, and the LLM prompt, all to the clipboard.
- [Web build](./web.md): the wasm build in headless Chromium (boot, file download, clipboard, persisted theme), driven by `scripts/web` instead of `moca`.

Not mapped yet: unsaved-changes close confirmation (`ctrl+w` on a dirty tab), theme toggle, zoom/fit, tab rename (double-click a pill), and the shortcut sheet (`F1`).
