# Grammar tab

Grammar tabs are text-based: a productions editor on the left and a "Test a word" card on the right. The user parses the grammar, checks a word with CYK, shows a leftmost derivation, and converts the grammar to Chomsky Normal Form in a new tab.

## Sub-features

- `gram-parse`: **Parse** (72, 734, or `ctrl+Return` in the editor) reads the editor. Success fills the result card with `Parsed OK. Start symbol: S. …`; a parse error opens the `That didn't work` modal (`Invalid grammar: missing '->' in "bogus" (line 1)`).
- `gram-check`: **Check** (823, 187, or `Return` in the word field) shows `Accepted "<w>" ✓` or `Rejected "<w>" ✗`. CYK splits the word into single characters.
- `gram-derive`: **Derive** (901, 187) shows a leftmost derivation, one sentential form per line with spaced symbols: `S` / `=> a S b` / `=> a a S b b` / `=> a a b b`. It gives up after 50,000 steps.
- `gram-cnf`: **To CNF** (151, 734) opens a `CNF` tab with the fresh start variable `S00`, and its card reads `Chomsky normal form ready. Start symbol: S00.`
- Check, Derive and To CNF re-parse the editor themselves; Parse is not a prerequisite.

## How to get to it (user POV)

- Pick the **Grammar** card on the startup picker (`Down Right Return`, or click 772, 500), use **New → Grammar**, or load a `.ce` file with a grammar entity (its card reads `Loaded from .ce file. Start symbol: S.`).

## Driving it with moca

Preconditions:

- A grammar tab is active, for example `anbn 2` from `examples/demo.ce` (`S -> a S b | ε`). A fresh grammar tab is prefilled with `S -> a S b | ε` / `S -> ε`; to type other rules, click the editor (380, 400), `$M key ctrl+a`, then `type` them, using `Return` for new lines.

- **Check.** Run `$M click 1015 140`, `$M type aabb`, and `$M click 823 187`. The result card reads `Accepted "aabb" ✓`.
- **Derive.** Run `$M click 901 187`. The result card lists `S`, `=> a S b`, `=> a a S b b`, `=> a a b b`, one per line.
- **Reject.** Run `$M click 1015 140`, `$M type b`, `$M click 823 187`: `Rejected "aabbb" ✗`.
- **To CNF.** Run `$M click 151 734`. A `CNF` tab opens with productions starting `S00 -> ε | …`.
- **Parse error.** On the `CNF` tab, run `$M click 380 400`, `$M key ctrl+a`, `$M type bogus`, `$M click 72 734`: the error modal opens. `$M key Escape` closes it.
- **Proof.** Run `$M shot grammar-check` and `$M shot grammar-derive`. For CNF, save the new tab to `$EVID/cnf.ce` and `cat` it.

## Gotchas

- The tool palette and run dock are absent here, and the tool keys are inert.
- The result card shows the last action only. Shoot after each button press.
- Check runs CYK on the CNF and stays fast. Derive on wide grammars (e.g. `S -> S S`) can search up to its 50,000-step cap before reporting `No leftmost derivation of "w" within 50000 steps.`
