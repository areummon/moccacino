# AGENTS.md

## Build & verify (non-obvious)

The app binary belongs to the **root** package: `Cargo.toml` maps `[[bin]] moccacino` to `moca-gui/src/main.rs`. Plain cargo commands default to the root package and work:

- Run app: `cargo run`
- Check/build: `cargo check` / `cargo build` (workspace-wide commands also work)
- Tests: `cargo test -p moca-data --bin moca-data`

## Known quirks

- Cargo warns "ignoring invalid dependency `moca-gui` which is missing a lib target": the root package lists `moca-gui` as a dependency, but `moca-gui` is bin-only, so the dep is ignored — the real linkage is the root `[[bin]]` compiling `moca-gui/src/main.rs` directly. Cosmetic; leave as-is.
- Dependencies used by code under `moca-gui/src/` must be declared in **both** `moca-gui/Cargo.toml` and the root `Cargo.toml`, because the same sources compile under two packages.
- Running the GUI needs a working Vulkan/EGL stack. The flake pins its own Mesa driver set (`VK_DRIVER_FILES`, hardware ICDs first, lavapipe software fallback) so driver ABIs always match the shell's libraries. If wgpu panics at instance creation (`wgpu-core ... instance.rs`), it's an environment/driver issue, not a code bug — check `VK_DRIVER_FILES` is exported (i.e., you entered via the flake) and use `VK_LOADER_DEBUG=warning cargo run` to see rejected drivers.

## Testing quirks

- Tests live only in `moca-data/src/tests/`, wired via `#[cfg(test)] mod tests;` in moca-data's **main.rs**, not lib.rs. So `-p moca-data --lib` runs zero tests; plain `cargo test` tests only the root package (also zero). Use the exact command above.
- Baseline on main: 16 pass / 0 fail. (An earlier baseline of 13 pass / 3 fail was fixed: NFA `check_input` dropped ε-paths when a longer label also matched, the λ-closure missed reachable states and fed wrong subsets to `to_dfa`, PDA transitions with ε input plus stack ops never fired, and the second PDA test fixture was unsatisfiable and got replaced with a correct aⁿbⁿ automaton.) Test names use snake_case (e.g. `check_input_nfa_test`). The Minimize/Hopcroft algorithm is still known-buggy (README admits this) even though its tests pass — don't rely on it or rewrite it casually.

## Architecture

- `moca-data`: dependency-free automata library. `FiniteAutomata` (+ subset construction NFA→DFA, Hopcroft minimization), `PushdownAutomata`, shared `StateMachine` trait (`state_machine.rs`), `State` (`state.rs`). Its bin target is vestigial (`fn main() {}`); its modules compile twice (lib + bin).
- `moca-gui/src`: Iced 0.13 GUI compiled by the root package. Directory modules:
  - `gui/`: `app.rs` = App state + slim `update()` dispatcher + `view()` assembly; `message.rs` = top-level `Message`; `tab.rs` = `Tab` model + automata sync/load + tree/grid layouts; `canvas_input.rs` = canvas/keyboard message handlers; `editing.rs` = label/dialog editing handlers; `operations.rs` = Check Input/DFA→NFA/Minimize/LaTeX handlers; `workspace.rs` = tab lifecycle handlers; `toolbars.rs`/`dialogs.rs` = view builders.
  - `state_machine/`: iced canvas program. `program.rs` = `Program` impl (mouse events + draw; trait impl kept whole); `hit_testing.rs` = click hit-testing math; `state.rs` = cache/controller (`State`); `node.rs`/`transition.rs`/`pending.rs` = drawn entities; `message.rs` = `CanvasMessage`; `util.rs` = `VectorExt`.
  - `tikz_export.rs` = TikZ LaTeX export.
- Cross-module visibility inside the GUI crate uses `pub(crate)` deliberately (sibling modules share field access); keep new items crate-private.

## Environment

- Nightly Rust is pinned only in `flake.nix` (no `rust-toolchain.toml`). Enter the devshell with **direnv** (`.envrc` → `use flake`) or plain `nix develop`. The shellHook sets `LD_LIBRARY_PATH` for X11/Wayland/Vulkan/Mesa/OpenSSL and exports `VK_DRIVER_FILES`/`LIBGL_DRIVERS_PATH` pointing at the flake-pinned Mesa — required by iced/winit to link and run.
- Background: devshell libraries are pinned by `flake.lock`. If the host OS is updated to newer nixpkgs while this lockfile stays old, `/run/opengl-driver` drivers stop loading inside the shell (glibc/wayland symbol clashes). The flake-pinned Mesa sidesteps this.
- `nix build` → `./result`; the packaged binary is wrapped (`wrapProgram`) with the same rendering env, so `./result/bin/moccacino` / `nix run` work outside the devshell too.
- Nix flakes only ingest **git-tracked** files: after adding source files/dirs they must be at least intent-to-add (`git add -N …`), or `nix build` fails with E0583 "file not found for module".
