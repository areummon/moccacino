# Exports

The Export menu turns the active machine into text the user pastes elsewhere: LaTeX/TikZ code, a regular expression (finite automata), or the vision-LLM prompt that turns a diagram photo into a `.ce` file. LaTeX and regex land on the clipboard through their dialog's Copy button; the LLM prompt is copied directly.

## Sub-features

- `exp-latex`: **LaTeX / TikZ** opens the `LaTeX export` dialog with code (machine tabs: TikZ; grammar tabs: an `align*` block), and **Copy** (895, 608; the body scrolls, so the button never moves) puts it on the clipboard with the toast `LaTeX copied to the clipboard`. An empty canvas gives `Nothing to export: the canvas is empty.`
- `exp-regex`: **Regular expression** (finite tabs only) opens `Equivalent regular expression` (state elimination, not simplified: `s -b-> q1` gives `εbε`), and **Copy** (804, 467) copies it with the toast `Regular expression copied`.
- `exp-llm`: **Copy LLM prompt** copies the prompt directly and shows the toast `LLM prompt copied`.

## How to get to it (user POV)

- **Export ▾** (430, 25), then LaTeX (470, 78), Regular expression (470, 111), or Copy LLM prompt (470, 160). These positions hold on finite tabs; elsewhere the disabled regex item grows a `Finite automata only` subtitle and pushes the LLM item down, so shoot the menu first.

## Driving it with moca

Preconditions:

- A machine tab has content, for example the two-state DFA from [canvas-editing](./canvas-editing.md).

- **LaTeX.** Run `$M click 430 25` and `$M click 470 78`. The dialog shows `\begin{center}` / `\begin{tikzpicture}` with a `\node[state, initial]` and a `\node[state, accepting]` per state and a `\path[->]` per edge. Run `$M shot latex`.
- **Copy.** Run `$M click 895 608`, then `$M clip latex`. Stdout starts with `% Paste this into your LaTeX document`, and `$EVID/latex.txt` holds the full code.
- **Regex.** Open Export, click (470, 111), shoot the dialog, run `$M click 804 467` (Copy), and run `$M clip regex`.
- **LLM prompt.** Open Export, click (470, 160), then `$M clip llm`: it starts `You are given an image of one or more computational models`. Check the regex against the machine, for example by building it back with Operations → Build from regex and running the same words.
- **Proof.** The dialog shot and the `$EVID/*.txt` clipboard file must agree with each other and with the canvas: the same state labels, accepting flags, and edge labels.

## Gotchas

- The clipboard belongs to the Xvfb display and the running app. Read it with `$M clip` before `$M down`, because it dies with the instance.
- TikZ node ids are always `q0…qN` by state index. The user's state name appears only in the math label (`{$s$}`, digits subscripted: `q10` → `$q_{10}$`), so compare labels, not ids.
