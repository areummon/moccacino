# Vision-LLM prompt for generating `.ce` files

Give the prompt below to any vision-capable LLM together with an image of a
state diagram (or TM / PDA / grammar). The model answers with `.ce` file
contents you can save and load via the **File ▾** menu in the top bar. The same text is
available in-app via **Copy LLM Prompt**.

---

You are given an image of one or more computational models (a state diagram of an automaton, a Turing machine, a pushdown automaton, a regular expression, or a context-free grammar).

Transcribe what you see into a `.ce` file with the exact syntax below, then output ONLY the file contents (no explanations, no code fences).

SYNTAX
- One entity per block, each block starts with `entity: <kind>` where kind is one of:
  dfa | nfa | fa | automata | finite | tm | turing | pda | pushdown | regex | grammar | cfg
- Optional `name: <label>` line right after `entity:`.
- Lines starting with `#` are comments. Blank lines are ignored.
- State lists and finals are comma-separated. Multiple `transitions:` / `productions:` lines accumulate.

TRANSITION SYNTAX (different per family — use the right one!)
- dfa/nfa/fa/automata/finite:  (from, symbol) -> to
    symbols may be single characters; use ε for epsilon moves.
- pda/pushdown:  (from, input, pop, push...) -> to
    4 parts: input read, stack symbol popped, stack symbols pushed (comma-separated = pushed in order, last on top). Use ε for no-ops.
    Example: (q0, a, Z, A, Z) -> q1   (read a, pop Z, push A then Z)
- tm/turing:  (from, read, write, dir[, read, write, dir...]) -> to
    one (read, write, dir) group per tape; dir is L, R or S.
    Example single tape: (q0, 0, 1, R) -> q1
    Example two tapes:   (q0, 0, 1, R, _, _, S) -> q1
- regex: a line `regex: <pattern>` with operators | * + ? and ε.
- grammar: `productions: <lhs> -> body | body` lines; ε is an empty body.

COMMON KEYS
- `states: a, b, c` (optional for regex/grammar)
- `initial: <state>` (machines only)
- `final: <state>, <state>` also accepts `finals:` / `halt:` / `final/halt:` (machines only)
- `blank: <char>` (turing only, default _)
- `stack: <symbol>` (pushdown only, default Z)

EXAMPLE FILE
# Balanced parentheses recognizer and a regex, together in one file
entity: pda
name: balanced-parens
states: q0, q1, q2
transitions: (q0, ε, Z, PZ) -> q0, (q0, (, P, PP) -> q0, (q0, ), P, ε) -> q1, (q1, ε, Z, Z) -> q2
initial: q0
final: q2

entity: regex
name: abb
regex: (a|b)*abb

entity: grammar
name: anbn
productions: S -> a S b | ε

TASK
Look at the attached image carefully (states, arrows, labels, initial arrow, double circles for accepting states, stack/tape annotations) and produce the matching `.ce` file. Use short state names exactly as drawn or numbered q0, q1, ... if unlabeled. Output only the .ce file contents.
