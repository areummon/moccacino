# moccacino

moccacino is a desktop workbench for experimenting with concepts in automata theory and formal languages. Draw machines on a canvas, run them step by step, and apply the classic conversions — built with Rust and the [Iced](https://github.com/iced-rs/iced) GUI library.

## Features

- **Finite automata (DFA/NFA)** — canvas editing with ε-transitions, deterministic and nondeterministic execution, NFA→DFA (subset construction), and Hopcroft minimization.
- **Pushdown automata** — transitions of the form `input;pop/push`, a run panel with a live stack lane, deterministic and nondeterministic execution.
- **Turing machines** — multi-tape support over a shared blank symbol, arrival-based acceptance, run panel with tape strips, breadth-first nondeterministic exploration.
- **Context-free grammars** — text format (`S -> a S b | ε`), CYK membership, leftmost/rightmost derivations, conversion to Chomsky Normal Form, right-linear grammar ↔ automaton conversion, and a CFG→PDA recognizer.
- **Regular expressions** — parser supporting `|`, juxtaposition, `*`, `+`, `?`, grouping, escapes, and literal `ε`; compiles straight into an automaton (Thompson construction).
- **`.ce` files** — plain-text documents that hold any mix of the above in one place; load them and every entity opens as its own tab (see [`docs/ce-format.md`](docs/ce-format.md)).
- **TikZ export** — generates LaTeX code for the automaton or grammar you drew.

## Installation

### Prerequisites

- Rust and Cargo (the [Nix flake](https://nixos.wiki/wiki/Flakes) pins nightly; stable should also build the workspace)
- On Linux, the usual windowing/graphics development libraries (X11, Wayland, Vulkan) — the flake provides all of them

### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/yourusername/moccacino.git
cd moccacino
```

2. Build the project:
```bash
cargo build --release
```

3. Run the application:
```bash
cargo run --release
```

Or, with Nix installed:

```bash
nix run .          # build and run (binary is wrapped with the right GL/Vulkan env)
nix build          # produce ./result/bin/moccacino
nix develop        # reproducible dev shell (use direnv for it to be automatic)
```

> [!WARNING]
>
> There may be some bugs I do not currently know about.

## Usage

### Canvas editing

Editing is tool-based (JFLAP style). Pick a tool from the toolbar or with the keyboard — `1`–`4` select tools, `Tab` toggles between Select and Delete, holding `Delete` temporarily engages Delete, and `Esc` returns to Select.

- **Select** (`1`) — drag states to move them, drag empty space to pan, double-click a state to rename it, `Shift`+click toggles final, `Alt`+click toggles initial, and clicking a transition opens its label editor.
- **State** (`2`) — click empty canvas to create a state.
- **Transition** (`3`) — click the source state, then the target state, to add a transition.
- **Delete** (`4`) — click a state or a transition to remove it.

Transition labels mean different things per machine family:

| Family | Label format |
|---|---|
| Finite | one symbol per label; `ε` (or empty) is an ε-transition |
| Pushdown | `input;pop/push` — `ε` as input consumes nothing, as pop/push it's a no-op |
| Turing | `read;write/dir` per tape with dir ∈ {`L`, `R`, `S`}; multitape labels join one segment per tape, e.g. `a;X/R,_;*/L` |

### Run panels

Every machine tab has a run panel: type an input, press **Load**, then **Step** or **Play** to watch the execution — a tape strip for Turing machines, a stack lane plus input ribbon for pushdown, an input ribbon for finite automata. Nondeterministic machines display their branching frontier as lanes you can step level by level.

### Operations

From the **Operations** menu:

- **Check Input** — test whether a string is accepted (works for every family; grammars answer via CYK)
- **DFA to NFA** — convert a deterministic finite automaton to a non-deterministic one
- **Minimize** — minimize a deterministic finite automaton
- **Build from Regex** — compile a regular expression into an automaton and open it as a new tab
- **To Regex** — inspect the expression behind a regex-built tab

### Grammars

Grammar tabs are text-driven: write one production group per line (`S -> a S b | ε`) in the panel, check words via CYK, view leftmost derivations, or convert the grammar to Chomsky Normal Form (opened as a new tab).

### Files

Use **File ▾ → Load .ce…** to open a `.ce` (or `.cm`) file — each entity in it becomes its own tab — and **Save .ce…** to write the active tab back out. Ready-made demos live in [`examples/`](examples/), and the full syntax is documented in [`docs/ce-format.md`](docs/ce-format.md):

```text
entity: pda
name: anbn
states: p0, p1, p2
transitions: (p0, a, Z, A, Z) -> p0, (p0, a, A, A, A) -> p0
transitions: (p0, ε, A, A) -> p1, (p0, ε, Z, Z) -> p1
transitions: (p1, b, A, ε) -> p1, (p1, ε, Z, Z) -> p2
initial: p0
final: p2
```

### LaTeX

You can get the LaTeX code for the automaton or grammar you have drawn — just click the button and you will get the code. It uses the tikz package and the automata, arrows.meta, and positioning libraries from TikZ.

You can also change the settings by modifying the `moca-gui/tikz_export.rs` file with your desired preferences.

> [!NOTE]
>
> Currently, you cannot change the position of loops in the GUI. If you want to change the position 
> of a loop in the resulting TikZ code, simply change `edge[loop above]` to `edge[loop below]`.

## Development

This project uses a [rust workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) structure with two main crates:

- `moca-data`: dependency-free automata library — the engines for every family (stepping, validation, conversions), grammars, and the regex compiler.
- `moca-gui`: the GUI implementation using the [Iced](https://github.com/iced-rs/iced) library (the packaged binary is built from its sources).

Tests live in `moca-data` and run with:

```bash
cargo test -p moca-data --bin moca-data
```

The project uses [Nix flakes](https://nixos.wiki/wiki/Flakes) to provide a reproducible development environment and build process, leveraging [Crane](https://github.com/ipetkov/crane) for Rust builds. Enter the dev shell with [direnv](https://direnv.net/) (the committed `.envrc` handles it) or plain `nix develop`.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Iced](https://github.com/iced-rs/iced) GUI framework
- Inspired by various finite automata visualization tools like [JFLAP](https://www.jflap.org/) and [automatarium](https://github.com/automatarium/automatarium).
