# Exports

The Export menu turns the active machine into text the user pastes elsewhere: LaTeX/TikZ code, a regular expression (finite automata), or the vision-LLM prompt that turns a diagram photo into a `.ce` file. Each export lands on the clipboard.

## Sub-features

- `exp-latex`: **LaTeX / TikZ** opens the `LaTeX export` dialog with code, and **Copy** (895, 608) puts it on the clipboard.
- `exp-regex`: **Regular expression** opens a dialog with an equivalent regex and a Copy button (finite tabs).
- `exp-llm`: **Copy LLM prompt** copies the prompt directly and shows a toast.

## How to get to it (user POV)

- **Export ▾** (430, 25), then LaTeX (470, 78), Regular expression (470, 111), or Copy LLM prompt (470, 160).

## Driving it with moca

Preconditions:

- A machine tab has content, for example the two-state DFA from [canvas-editing](./canvas-editing.md).

- **LaTeX.** Run `$M click 430 25` and `$M click 470 78`. The dialog shows `\begin{tikzpicture}` with a `\node[state, initial]` and a `\node[state, accepting]` per state and a `\path[->]` per edge. Run `$M shot latex`.
- **Copy.** Run `$M click 895 608`, then `$M clip latex`. Stdout starts with `% Paste this into your LaTeX document`, and `$EVID/latex.txt` holds the full code.
- **Regex.** Open Export, click (470, 111), shoot the dialog, click its Copy button, and run `$M clip regex`. Check the regex against the machine, for example by building it back with Operations → Build from regex and running the same words.
- **Proof.** The dialog shot and the `$EVID/*.txt` clipboard file must agree with each other and with the canvas: the same state names, accepting flags, and labels.

## Gotchas

- The clipboard belongs to the Xvfb display and the running app. Read it with `$M clip` before `$M down`, because it dies with the instance.
- The LaTeX dialog's Copy button sits at the bottom of a tall card. If the text is longer, re-shoot to find it.
