# The `.ce` file format (computational entities)

A `.ce` file is a small, key-based text document that declares any number of
computational models — finite automata, Turing machines, pushdown automata,
regular expressions and grammars — in one place. Load it in moccacino via
**File ▾ → Load .ce…** in the top bar: every healthy entity opens as its own
tab. Files with the `.cm` extension are accepted as well.

The syntax is intentionally regular so that both humans and LLMs can write it
reliably (see `docs/vision-llm-prompt.md` for a paste-ready prompt that turns
a state-diagram image into a `.ce` file).

## Structure

- The document is a sequence of **entity blocks**. Each block starts with an
  `entity: <kind>` header; the following `key: value` lines belong to that
  block until the next `entity:` header.
- Lines starting with `#` and blank lines are ignored.
- Keys are case-insensitive. `transitions:`, `productions:` and `states:`
  lines may repeat; their contents accumulate.
- Entities are parsed **independently**: one broken block is skipped (and
  reported) without affecting the others.

### Entity kinds

| Alias(es)                                   | Builds            |
|---------------------------------------------|-------------------|
| `dfa`, `nfa`, `fa`, `automata`, `automaton`, `finite` | Finite automaton tab |
| `tm`, `turing`                              | Turing machine tab |
| `pda`, `pushdown`                           | Pushdown automaton tab |
| `regex`, `re`                               | Finite automaton tab (Thompson construction) |
| `grammar`, `cfg`                            | Grammar tab |

### Common keys

| Key | Applies to | Meaning |
|-----|------------|---------|
| `name:` | all | Tab name (optional; family defaults + ordinals otherwise) |
| `states:` | machines | Comma-separated state labels; labels referenced elsewhere are auto-registered |
| `initial:` | machines | The initial state (single label) |
| `final:` / `finals:` / `halt:` / `final/halt:` | machines | Comma-separated accepting/halting states |
| `blank:` | `tm` only | Tape blank symbol, single character (default `_`) |
| `stack:` | `pda` only | Initial stack symbol (default `Z`) |
| `regex:` | `regex` only | The regular expression |
| `productions:` | `grammar` only | One grammar line, e.g. `S -> a S b | ε` |

Keys from another family (e.g. `blank:` in a PDA) are reported as errors.

## Transition syntax — different per family

Each family has its own tuple shape so the three can never be confused.

### Finite automata (`dfa`, `nfa`, …) — 2 parts

```
transitions: (from, symbol) -> to
```

```text
entity: dfa
states: s0, s1
transitions: (s0, a) -> s1, (s0, b) -> s0
transitions: (s1, a) -> s1, (s1, b) -> s0
initial: s0
final: s0
```

An empty symbol position means ε.

### Pushdown automata (`pda`) — at least 4 parts

```
transitions: (from, input, pop, push...) -> to
```

The three fixed parts are the input symbol, the popped stack symbol and the
pushed stack entries; everything after the pop part re-joins into the push
string (comma-separated entries are pushed in order, last on top), so
`(q0, a, Z, A, Z) -> q1` means *read a, pop Z, push A then Z*. Use ε for
no-ops.

```text
entity: pda
states: q0, q1, q2
transitions: (q0, a, Z, A, Z) -> q0, (q0, a, A, A, A) -> q0
transitions: (q0, ε, Z, Z) -> q1, (q1, b, A, ε) -> q1
transitions: (q1, ε, Z, Z) -> q2
initial: q0
final: q2
```

### Turing machines (`tm`) — one (read, write, dir) group per tape

```
transitions: (from, read, write, dir[, read, write, dir...]) -> to
```

`dir` is `L`, `R` or `S` (case-insensitive). The first transition fixes the
tape count for the entity; a later transition with a different group count is
an error. All tapes read simultaneously.

```text
entity: tm
states: scan, accept
transitions: (scan, 1, 1, R) -> scan, (scan, 0, 1, S) -> accept
initial: scan
final: accept
```

## Saving

**File ▾ → Save .ce…** writes the active tab as one entity block in this
format. Saving is refused for empty entities (no states / no productions)
and for machines that fail validation (e.g. no initial state), so every
saved file reloads without errors. Saved files pick the `dfa`/`nfa` alias
from the determinism flag, sort states and transitions deterministically,
omit `blank:`/`stack:` when they carry the defaults, and emit the start
symbol's productions first (the first line fixes the start symbol on
reload). Labels containing `,`, `(` or `)` are rejected — those characters
structure the transition tuples.

## Regex and grammar blocks

```text
entity: regex
name: abb
regex: (a|b)*abb
```

```text
entity: grammar
productions: S -> a S b | ε
productions: A -> a A | a
```

`productions:` lines use the same syntax as the grammar editor: alternatives
separated by `|`, the first left-hand side is the start symbol, and an empty
body (or literal `ε`) denotes an ε-production. Symbol names may not contain
`,`, `;`, `/` or be `ε`.

## Errors

A block that fails to parse or build is skipped and summarized in the error
popup (`'name' (line N): message`); every other entity still opens. Referenced
-but-undeclared state labels are auto-registered instead of erroring, which
keeps LLM-generated files forgiving.

## Full example

See `examples/demo.ce` — one entity of every kind in a single loadable file.
