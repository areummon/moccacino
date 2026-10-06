---
name: verify
description: Launch and drive the moccacino desktop GUI (Iced, automata workbench) the way a user does — on a private Xvfb display with xdotool clicks/keys, screenshots and clipboard reads as evidence. Use to prove any user-visible change (canvas editing, run panels, operations, .ce load/save, grammar tab, exports) in the real app instead of only `cargo test`.
---

# Verify moccacino in the real GUI

moccacino is a single-window Iced 0.13 desktop app; the user touches a canvas,
header menus, dialogs and a bottom run dock. There is no accessibility tree and
no debug port, so the harness is X11 input injection on an **isolated Xvfb
display**: it never moves the user's mouse, never steals focus on their
Hyprland desktop, and two instances can run side by side (`MOCA_ID`).

Everything goes through one helper, `.claude/skills/verify/scripts/moca`
(run from the repo root; it finds the repo itself). It pulls Xvfb, xdotool,
imagemagick and xclip from nixpkgs on first use (cached in
`$MOCA_VERIFY_ROOT/tools.path`) and runs the app inside the flake devshell.

Feature recipes live in [`features/`](features/README.md); read the index
before driving, and use the matching file.

## Launch

```bash
export MOCA_VERIFY_ROOT=<your scratchpad>/moca-verify   # default: ${TMPDIR:-/tmp}/moca-verify
M=.claude/skills/verify/scripts/moca
$M up          # cargo build (devshell) → Xvfb :90+ → app → waits for the "Moccacino" window
```

Ready when it prints `up: instance=… display=:N window=…` (it already waited
for the window to map and ~1.5 s for the first lavapipe frame). The app always
opens on the **startup picker** (Finite / Turing / Pushdown / Grammar cards).

What `up` isolates, and why:

- `DISPLAY=:N` (first free from :90), `WAYLAND_DISPLAY` unset → winit's X11 backend inside Xvfb.
- `VK_DRIVER_FILES` narrowed to lavapipe: Xvfb has no DRI3, software Vulkan is the deterministic path.
- `XDG_CONFIG_HOME=$RUN/config`: theme settings never touch `~/.config/moccacino`.
- `DBUS_SESSION_BUS_ADDRESS=disabled:`: the dialogs' **Browse…** buttons (rfd → xdg portal) cannot pop a file chooser on the real desktop; they fail into an in-app error instead.
- cwd = the evidence dir: the save dialog's default path is relative (`Automaton.ce`), so a bare Enter writes evidence, not repo files.

Parallel instance: `MOCA_ID=b $M up` (own display, run dir and evidence dir).
`up` refuses if that `MOCA_ID` is already running — never double-drive one instance.

## Doctor

```bash
$M doctor
```

Read-only. Checks Xvfb and app pids, that the window exists (expects
`1280x820`), that the binary is newer than every `.rs` file (otherwise
`WARN … moca down && moca up rebuilds`), no `panicked` in the app log, and that
the window is not blank (pixel stddev > 0.01). Ends with `healthy` or
`unhealthy` (exit 1). Run it first whenever anything looks off; `$M log` shows
stdout/stderr (a wgpu panic in `instance.rs` is the driver environment, see
AGENTS.md, not a code bug).

## Drive

All coordinates are **window-relative pixels of the 1280×820 window** (the
full-width layout: brand, menus and tabs on one header row). Each command waits
`MOCA_SETTLE` (0.35 s) afterwards so the next frame is drawn.

| Command | Does |
|---|---|
| `$M click X Y [shift\|alt\|ctrl\|double]…` | left click; modifiers are held 0.2 s before the click (see gotcha) |
| `$M drag X1 Y1 X2 Y2` | press, move in 8 steps, release (move states, pan) |
| `$M size W H` | resize the window (responsive layouts; see Gotchas) |
| `$M move X Y` | hover only (also: park the pointer away before a shot to drop hover rings/tooltips) |
| `$M key K…` | xdotool keysyms: `Return Escape Tab Delete Right space 1 2 ctrl+s ctrl+o ctrl+Tab ctrl+shift+Tab ctrl+a F1` |
| `$M type TEXT` | types into the focused widget; unicode like `ε` works |
| `$M shot NAME` | window PNG → `$EVID/NNN-NAME.png` (prints the path; Read it to look) |
| `$M clip [NAME]` | prints the X clipboard; with NAME also saves `$EVID/NAME.txt` |
| `$M note TEXT` | appends a timestamped line to `$EVID/notes.txt` |
| `$M env` | DISPLAY/WID/paths for ad-hoc `xdotool` (prepend `TOOLS_PATH` to `PATH`, unset `LD_LIBRARY_PATH`) |

Prefer keyboard handles over coordinates — they are stable across layout
changes:

- Startup picker: arrows move the highlight in the 2×2 grid, `Return` opens it (Finite is preselected), `Escape` quits the app.
- Tools: `1` Select, `2` State, `3` Transition, `4` Delete, `Escape` back to Select. Inert while a dialog is open or a text field has focus.
- `ctrl+o` load dialog, `ctrl+s` save dialog, `ctrl+Tab`/`ctrl+shift+Tab` cycle tabs (in tab order), `ctrl+w` close tab, `ctrl+shift+l` theme, `f` fit view, `F1` shortcut sheet.
- Run dock: `Return` in its input field = Load, `Right` = Step, `space` = Play/Pause (both only when the canvas, not a text field, has focus — click empty canvas first).
- Every dialog autofocuses its text field: `type` then `Return` submits, `Escape` cancels.

Fixed coordinates (1280×820, verified):

| Target | X Y |
|---|---|
| Header menus: New / Operations / File / Export | 175 25 / 265 25 / 357 25 / 430 25 |
| New-tab `+` / shortcuts / theme toggle (right side) | 1167 25 / 1216 25 / 1251 25 |
| File menu items: Open .ce / Save tab as .ce | 420 78 / 428 111 |
| Export menu items: LaTeX/TikZ / Regular expression / Copy LLM prompt | 470 78 / 470 111 / 470 160 |
| Operations: Check input… | 300 78 |
| Run-dock input field (idle dock, every machine family) | 285 725 |
| Empty canvas (safe spot to take focus off a text field) | 640 600 |

The Operations menu's lower items move: a disabled item grows a reason
subtitle (`Finite automata only`, `Already deterministic`, `Needs a DFA`), so
**`shot` the open menu and read the positions before clicking** NFA → DFA,
Minimize or Build from regex (seen at y 130/172/217 on a DFA, 233 for regex on
a TM tab). Tab pills scroll when the active tab changes; switch tabs with
`ctrl+Tab`, not by clicking pills.

## Evidence

Location: `$MOCA_VERIFY_ROOT/evidence/$MOCA_ID/` — numbered screenshots,
`*.txt` clipboard captures, `notes.txt`, files the app saved, and `app.log`
(copied there by `down`). It survives cleanup by design.

Proof standards:

- Drive the real user path (canvas clicks, menus, dialogs, keys). Never construct state behind the GUI's back (e.g. writing a `.ce` file to skip drawing, unless the feature under test *is* loading).
- Capture the action and the resulting state: a `shot` before and after the step that matters, not just the final screen. Park the pointer (`$M move 1000 200`) first so hover rings do not masquerade as state (a hovered state draws a ring that looks like an accepting double circle).
- Verify side effects beside the pixels: save to `$EVID/x.ce` and `cat` it; read exported text with `$M clip`; a toast (bottom right, ~3 s) is the only trace of Check input, so shoot it right away.
- A verdict needs the dock badge (`Accepted`/`Rejected`), not `Running…`: the finite run needs one more Step after the last symbol.
- Crops/montages you make for viewing go outside `$EVID` (or carry no `NNN-` prefix), so the numbering stays a clean timeline.

## Cleanup

```bash
$M down        # kills exactly the app + Xvfb pids recorded by `up`, deletes $RUN (config, logs)
```

It never kills by process name, keeps `$EVID` (prints `evidence kept: …`), and
is safe to rerun. Run it after every failed attempt too. Afterwards `git status`
must show no stray files (only `.claude/` if the skill itself is uncommitted).

## Gotchas

- **Modifier clicks need a hold.** The app learns Shift/Alt from its key subscription, which iced delivers *after* the canvas has handled a same-batch click; an instant `keydown shift; click` toggles nothing. `click … shift` already sleeps 0.2 s between them — do the same in any raw xdotool.
- Ordering inside one gesture matters: after `3` (Transition tool) clicking source then target opens the **New transition** dialog with focus in `Label`; blank = ε.
- First `up` after a clean target recompiles iced/wgpu (minutes); later runs are ~1 s.
- `xdotool type` into the canvas does nothing — only focused text widgets take text.
- `$M size W H` resizes the window to exercise the responsive breakpoints (<1150 stacked dock header, <1000 compact brand, <800 tabs on their own row, <760 icon-only palette). Every coordinate in this skill and in `features/` assumes 1280×820, so resize back with `$M size 1280 820` or re-shoot before you click.
