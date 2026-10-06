# Grammar tab

Grammar tabs are text-based: a productions editor on the left and a "Test a word" card on the right. The user parses the grammar, checks a word with CYK, shows a leftmost derivation, and converts the grammar to Chomsky Normal Form in a new tab.

## Sub-features

- `gram-parse`: **Parse** (72, 734) reads the editor. The result card reports the start symbol or a parse error.
- `gram-check`: **Check** (823, 187) shows `Accepted "<w>" ✓` or a rejection.
- `gram-derive`: **Derive** (901, 187) shows a leftmost derivation `S => a S b => …`.
- `gram-cnf`: **To CNF** (151, 734) opens a `CNF` tab with a fresh start variable `S0…`.

## How to get to it (user POV)

- Pick the **Grammar** card on the startup picker (`Right Right` … or click 772, 500), use **New → Grammar**, or load a `.ce` file with a grammar entity.

## Driving it with moca

Preconditions:

- A grammar tab is active, for example `anbn 2` from `examples/demo.ce` (`S -> a S b | ε`). For a fresh grammar tab, click the editor (380, 400) and `type` the rules, using `Return` for new lines.

- **Check.** Run `$M click 1015 140`, `$M type aabb`, and `$M click 823 187`. The result card reads `Accepted "aabb" ✓`.
- **Derive.** Run `$M click 901 187`. The result card lists `S`, `=> a S b`, and so on, ending in `aabb`.
- **To CNF.** Run `$M click 151 734`. A new tab opens with CNF productions.
- **Proof.** Run `$M shot grammar-check` and `$M shot grammar-derive`. For CNF, save the new tab to `$EVID/cnf.ce` and `cat` it.

## Gotchas

- The tool palette and run dock are absent here, and the tool keys are inert.
- The result card shows the last action only. Shoot after each button press.
- Exhaustive rejections on wide grammars (e.g. `S -> S S`) can take a long time. Keep inputs short.
