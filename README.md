# moccacino - State Machine Visualizer

moccacino is a software program to experiment with concepts in automata theory and formal languages. Built with Rust and the Iced GUI library.

## Installation

### Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

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

> [!WARNING]
>
> There are several errors and bugs in the program, including some 
> functions that are not working correctly, such as the Minimize algorithm.
> Some situations are not caught or are simply not implemented.
> Additionally, the project's structure is not well modularized or organized.

## Usage

### Basic Controls

- **Click**: Create state
- **Click on state**: Create or complete a transition
- **Ctrl + Click**: Drag and drop states
- **Click on Labels**: Edit transition labels
- **Shift + Click**: Toggle final state
- **Alt + Click**: Set initial state
- **Tab**: Toggle deletion mode

### Operations

- **Check Input**: Test if a string is accepted by the automaton
- **DFA to NFA**: Convert a deterministic finite automaton to a non-deterministic one
- **Minimize**: Minimize a deterministic finite automaton

### LaTeX

You can get the LaTeX code for the state machine you have drawn—just click the button and you will get the code. It uses the tikz package and the automata, arrows.meta, and positioning libraries from TikZ.

You can also change the settings by modifying the `moca-gui/tikz_export.rs` file with your desired preferences.

> [!NOTE]
>
> Currently, you cannot change the position of loops in the GUI. If you want to change the position 
> of a loop in the resulting TikZ code, simply change `edge[loop above]` to `edge[loop below]`.

## Development

This project uses a [rust workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) structure with two main crates:

- `moca-data`: Contains the core state machine implementation.
- `moca-gui`: Contains the GUI implementation using the [Iced](https://github.com/iced-rs/iced) library.

The project uses [Nix flakes](https://nixos.wiki/wiki/Flakes) to provide a reproducible development environment and build process, leveraging [Crane](https://github.com/ipetkov/crane) for Rust builds.

> [!NOTE]
>
> This is a project I created and worked on a long ago. 
> I don't feel like developing it further right now, so 
> I'll leave it as is. I might update it in the future.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Iced](https://github.com/iced-rs/iced) GUI framework
- Inspired by various finite automata visualization tools like [JFLAP](https://www.jflap.org/) and [automatarium](https://github.com/automatarium/automatarium).
