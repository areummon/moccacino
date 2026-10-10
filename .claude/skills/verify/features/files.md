# Files (.ce)

The user saves the active tab to a plain-text `.ce` file and loads `.ce` files that can hold any mix of machines, regexes, and grammars. A load opens every entity in its own tab. A save clears the tab's unsaved dot. The format is described in `docs/ce-format.md`.

## Sub-features

- `file-load`: **Open .ce file…** loads the file. Each entity opens as a tab, and a toast reads `Loaded N entities from <file>`.
- `file-save`: **Save tab as .ce…** writes the active tab to the typed path (`.ce` is appended when the path has no extension), the toast reads `Saved <path>`, and the tab's unsaved dot disappears. Machine tabs need at least one state, an initial state and a passing validation; grammar tabs save as `entity: grammar`.
- `file-startup-open`: the startup picker's `Open a .ce file…` link (replaces the picker with a Finite `Automaton` tab, then opens the load dialog).
- `file-browse`: the **Browse…** button uses the xdg portal file chooser. With D-Bus disabled on purpose, it fails into an inline red line in the dialog: `no file chosen (the system file dialog may be unavailable on this system — type or paste the file path instead)`.
- The web build has no path dialogs: Open goes straight to the browser picker and Save downloads the file. See [web](./web.md).

## How to get to it (user POV)

- **File ▾** (357, 25), then Open (420, 78) or Save (428, 111).
- `ctrl+o` and `ctrl+s`.
- Startup picker: `Open a .ce file…` (456, 615).

## Driving it with moca

Preconditions:

- Run from the repo root, so `$PWD/examples/demo.ce` exists. For save, a machine tab has content.

- **Load.** From the startup picker, run `$M key Return` first. Then run `$M key ctrl+o`. A dialog with a path field opens, and the field has focus. Run `$M key ctrl+a`, `$M type "$PWD/examples/demo.ce"`, and `$M key Return`. Six tabs appear after the existing `Automaton` tab (`even-number-of-as`, `ends-with-b`, `anbn`, `contains-a-zero`, `abb`, `anbn 2`), the last one (grammar `anbn 2`) is active, and the toast reads `Loaded 6 entities from demo.ce`.
- **Save.** On a machine tab, run `$M key ctrl+s`. The `Save computational entity` dialog shows `<TabName>.ce` (`/`, `\` and newlines become `-`). Run `$M key ctrl+a`, `$M type "$EVID/saved"`, and `$M key Return`. The toast reads `Saved …/saved.ce`.
- **Proof.** Run `cat "$EVID/saved.ce"`. It shows `entity: dfa|nfa|pda|tm`, `name:`, `states:`, `transitions:`, and `initial:`, plus `final:` when there are accepting states, matching the canvas. The tab pill has lost its dot. Reload the saved file with ctrl+o and compare the shots.

## Gotchas

- On a fresh `up`, the startup picker owns the keyboard, so `ctrl+o` does nothing. Run `$M key Return` (opens a Finite tab) first, or click the picker's `Open a .ce file…`.
- Always type an absolute path. The default `Automaton.ce` is relative to the app's cwd. `moca up` sets that cwd to the evidence dir so that a stray Enter cannot write into the repo, but a typed relative path still lands there.
- Clicking **Browse…** shows the inline error above, because the portal is disabled. That is expected, not a bug. It also takes focus off the path field, so run `$M click 583 395` (the field) before typing a path.
- The load picks up the file as it is on disk. To verify a round trip, save, then load, then compare.
