# Operations

The Operations menu checks a word against the whole machine at once (any family) and runs the classic finite-automaton conversions. NFA → DFA and Minimize DFA each open the result in a new tab. Build from regex compiles a regular expression into an automaton in a new tab.

## Sub-features

- `ops-check`: **Check input…** opens the `Check an input` dialog. Submitting shows a toast such as `'abb' is rejected` or `is accepted`; a machine that fails validation (for example, no initial state) opens the error modal instead. Grammar tabs answer with CYK on the last parsed grammar, so re-Parse after editing the text.
- `ops-nfa-dfa`: **NFA → DFA** (subset construction) opens a new tab named `DFA`. It is disabled with `Already deterministic` on a DFA and `Finite automata only` elsewhere.
- `ops-minimize`: **Minimize DFA** (Hopcroft) opens a new tab named `Minimized`. It is disabled with `Needs a DFA — convert first`, `The canvas is empty`, or `Finite automata only`.
- `ops-regex`: **Build from regex…** opens the `Build from a regular expression` dialog. **Build** opens a `Regex: <pattern>` tab.

## How to get to it (user POV)

- Header **Operations ▾** (265, 25), then the item.

## Driving it with moca

Preconditions:

- `examples/demo.ce` is loaded (see [files](./files.md)). Tabs run in this order: `Automaton` (from the picker), `even-number-of-as`, `ends-with-b`, `anbn` (PDA), `contains-a-zero` (TM), `abb`, and `anbn 2` (grammar, active after the load). Count `ctrl+shift+Tab` presses back from the last tab: 2 reach the TM, 3 the PDA, and 4 the NFA.

- **Open menu.** Run `$M click 265 25`, then `$M shot ops-menu`. Read the item positions from the shot. Check input is always at (300, 78).
- **Check input.** Run `$M click 300 78`, `$M type abb`, and `$M key Return`. Immediately run `$M shot check`. A toast at the bottom right reads `'abb' is rejected` (on `contains-a-zero`) or `accepted`.
- **NFA → DFA.** On `ends-with-b` (NFA; the status bar reads `Nondeterministic`), open the menu and click **NFA → DFA** (y≈130 when Minimize shows a reason). A `DFA` tab opens, and its status bar reads `Deterministic`. Its menu shows `Already deterministic` under NFA → DFA and an enabled **Minimize DFA** at y≈172, which opens a `Minimized` tab.
- **Build from regex.** On any tab (the item is never disabled), open the menu and click **Build from regex…** (y≈217 on the NFA). Run `$M type "(a|b)*abb"` and `$M key Return`. A `Regex: (a|b)*abb` tab opens with a Thompson NFA, and the status bar reads `14 states · 16 transitions` and `Nondeterministic`.
- **Proof.** Shoot the result tab, then save it with `ctrl+s` to `$EVID/<name>.ce` and `cat` the file. To check language equivalence, run the result's dock on a few words: for example, `abb` and `aabb` are accepted and `ab` is rejected.

## Gotchas

- Disabled items grow a reason subtitle, which pushes the items below them down by about 16 px. Always read positions from a fresh menu shot.
- The Check input toast fades after a few seconds. Shoot it in the same breath as `Return`.
- Generated tabs (DFA, minimized, regex) count as unsaved as soon as they are non-empty, and their pills show the dot.
