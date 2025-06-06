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

- **Left Click**: Select states and transitions
- **Double Click**: Edit state or transition labels
- **Shift + Click**: Toggle final state
- **Alt + Click**: Set initial state
- **Tab**: Toggle deletion mode
- **Delete**: Enter deletion mode

### Operations

- **Check Input**: Test if a string is accepted by the automaton
- **DFA to NFA**: Convert a deterministic finite automaton to a non-deterministic one
- **Minimize**: Minimize a deterministic finite automaton

## Project Structure

- `moca-data/`: Core library for finite automata implementation
- `moca-gui/`: GUI application built with Iced framework

## Development

This project uses a workspace structure with two main crates:
- `moca-data`: Contains the core finite automata implementation
- `moca-gui`: Contains the GUI implementation using Iced

### Dependencies

- `iced`: GUI library for Rust


> [!NOTE]
>
> This is a project I created and worked on a few years ago. 
> I don't feel like developing it further right now, so 
> I'll leave it as is. I might update it in the future.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Iced](https://github.com/iced-rs/iced) GUI framework
- Inspired by various finite automata visualization tools like [JFLAP](https://www.jflap.org/) and [automatarium](https://github.com/automatarium/automatarium).
