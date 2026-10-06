# Run panel

Every machine tab docks a run panel below the canvas. The user types an input, loads it, and steps or plays the run. The panel shows the current state, the step count, and a verdict badge. Finite machines show an input ribbon, pushdown automata show a stack lane plus the ribbon, and Turing machines show tape strips. Nondeterministic machines show parallel lanes that advance level by level, capped at 12.

## Sub-features

- `run-load`: Load (or `Return` in the field) loads the word. The button becomes `Reload`, and the badge reads `Running…`.
- `run-step`: Step (`Right`) advances one step, or one level for nondeterministic machines.
- `run-play`: Play (`space`) auto-plays on a 250 ms timer until the run halts. The control then shows ▶ again.
- `run-reload-paused`: Load or Reload starts the new run paused, even right after an auto-played run.
- `run-reset`: Reset returns to the loaded start configuration.
- `run-verdict`: the badge reads `Accepted` or `Rejected`, with `state <name>` and `steps <n>`, and the canvas glows the active state.
- `run-finite`, `run-pda`, `run-tm`: the per-family views (ribbon, stack lane, tape strip).

## How to get to it (user POV)

- Use the dock at the bottom of any machine tab (not grammar tabs). The `⌄` at its right collapses it.
- Click the field `Input word (blank = ε)`, type a word, and press **Load**. Then use the ↺ / ⏭ / ▶ controls.

## Driving it with moca

Preconditions:

- A machine tab is active. Either draw one (see [canvas-editing](./canvas-editing.md)) or load `examples/demo.ce` (see [files](./files.md)), which opens `even-number-of-as` (DFA), `ends-with-b` (NFA), `anbn` (PDA), and `contains-a-zero` (TM).

- **Load.** Run `$M click 285 725`, `$M type 110`, and `$M key Return`. The button reads `Reload`, and the badge reads `Running…` (finite or PDA) or the run starts (TM).
- **Step.** Run `$M click 640 600` to put focus on the canvas, then `$M key Right` once per step. The `state` and `steps` labels change, and the active state glows.
- **Play.** Run `$M key space`, then wait about 2 seconds (for example `sleep 2`). For the TM `contains-a-zero` with input `110`, the badge reads `Accepted` with `state found` and `steps 3`, the tape shows `1 1 1` with the head cell highlighted, and the control shows ▶, not a lit Pause.
- **Reload starts paused.** Run `$M click 272 682` (the field of the grown TM dock), `MOCA_SETTLE=0.05 $M key Return`, then `sleep 1.5` and `$M shot reloaded`. The badge reads `Running…` with `steps 0`. Press `space` (after `$M click 640 600`) to play it again.
- **Verdict on a finite DFA.** For `q0 -a-> q1` with `q1` accepting and input `a`, one `Right` shows `Running… state q1 steps 1`. A second `Right` gives the verdict.
- **Proof.** Run `$M shot run-before` after Load and `$M shot run-verdict` after the halt. The verdict shot shows the badge, state, steps, and the tape, stack, or ribbon.

## Gotchas

- The dock grows after Load (ribbon, stack, or tape rows), which shifts its input and buttons up to about y 682–704. Re-shoot before you click dock buttons. Use `Return` in the field and the keys `Right`/`space` instead.
- `Right` and `space` only act when the canvas has focus. After you type in the field, run `$M click 640 600` first. Otherwise `space` types a space into the input.
- `Running…` is not a verdict. A finite run needs one more step after it consumes the last symbol.
- An auto-play halt returns the control to ▶ (Play, disabled), and Load or Reload always starts the new run paused. A lit Pause icon next to a verdict badge, or a reloaded run that advances without `space`, is a regression.
- The PDA in `demo.ce` uses the `input;pop/push` labels. Its verdict for `aabb` should be `Accepted`.
