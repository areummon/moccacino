# Web build

The same app compiled to wasm runs in a browser tab (GitHub Pages, base path `/moccacino/`). It renders through WebGL2 and replaces the desktop's file dialogs, clipboard and settings file with browser equivalents. Everything that is not listed here behaves as on the desktop.

## Sub-features

- `web-boot`: the page replaces its `Loading Moccacino…` placeholder with the canvas and shows the startup picker without a `Quit` button. A panic reaches the browser console (`console.error: panicked at …`).
- `web-open`: **Open .ce** (menu, `ctrl+o`, picker link) opens the browser file picker directly; cancelling does nothing.
- `web-save`: **Save tab as .ce** downloads `<tab name>.ce`, marks the tab saved and toasts `Downloaded <name>.ce`.
- `web-clipboard`: LaTeX, regex and LLM-prompt copies write the system clipboard through `navigator.clipboard`.
- `web-theme`: the theme toggle persists in `localStorage["moccacino.settings"]` (`theme=dark`) across reloads.
- `web-shortcuts`: Ctrl chords work as on the desktop (`ctrl+shift+l` theme, `ctrl+o`, `ctrl+s`, `ctrl+a` in fields).
- `web-paste`: Ctrl+V pastes the system clipboard at the cursor in every text field and in the grammar editor.

## How to get to it (user POV)

- Open the deployed page, or serve a local build. Source: `index.html` (trunk entry, hides `navigator.gpu` to force WebGL2) and `moca-gui/src/platform.rs` (the only wasm seam).

## Driving it with web

The desktop harness (`moca`) does not apply. `.claude/skills/verify/scripts/web` serves a `dist/` directory at `http://127.0.0.1:8765/moccacino/`, drives headless Chromium (SwiftShader WebGL2) over CDP, prints console output and evaluated values, and exits 1 if the page logged an exception or a panic.

```bash
W=.claude/skills/verify/scripts/web
env -u LD_LIBRARY_PATH nix develop --command trunk build --release --public-url /moccacino/
$W dist "$EVID/web" '[{"click":[507,326],"shot":"fa"}]'
```

It needs `node` and `chromium` on `PATH` and free ports 8765 and 9333. It always shoots `00-loaded` after an 8 s boot wait. Actions run in order, each optionally followed by `wait` (ms, default 600) and `shot` (name): `click: [x, y]`, `dbl: [x, y]`, `key: "Escape"`, `type: "aa"`, `eval: "<js>"` (logged with its value), `cdp: [method, params]`, `reload: true`. Coordinates match the desktop's 1280×820 layout.

Preconditions:

- `dist/` was built from the current tree.

- **Boot.** Run with no actions. `00-loaded` shows the picker with `Open a .ce file…` and no `Quit`; the output is `(no console output)`.
- **Edit and run.** `[{"click":[507,326]},{"key":"2"},{"click":[400,350]},{"click":[750,350]},{"key":"3"},{"click":[400,350]},{"click":[750,350],"wait":800},{"type":"a"},{"key":"Enter"},{"click":[285,725]},{"type":"aa"},{"click":[428,725]},{"click":[556,704],"wait":4000,"shot":"played"}]`: two states, an `a` edge, and the run auto-plays to `Rejected` at `state q1 steps 1`.
- **Save.** Hook the download first, then save: `{"eval":"window.__dl=[];HTMLAnchorElement.prototype.click=function(){window.__dl.push(this.download);fetch(this.href).then(r=>r.text()).then(t=>window.__dl.push(t))};1"}`, then `{"click":[357,25]},{"click":[428,112],"wait":1500}` and `{"eval":"JSON.stringify(window.__dl)"}`: `["Automaton.ce","entity: dfa\n…"]`.
- **Clipboard.** `{"cdp":["Browser.grantPermissions",{"origin":"http://127.0.0.1:8765","permissions":["clipboardReadWrite","clipboardSanitizedWrite"]}]}` first; after Export → Copy LLM prompt (`{"click":[430,25]},{"click":[495,160],"wait":1000}`), read it back with `{"eval":"window.__c='';navigator.clipboard.readText().then(t=>window.__c=t.length);1","wait":1000},{"eval":"window.__c"}` (2463 characters).
- **Theme.** `{"click":[1251,25],"wait":800},{"eval":"localStorage.getItem(\"moccacino.settings\")"},{"reload":true,"shot":"reloaded"}`: `"theme=dark\n"`, and the reloaded page is dark.
- **Paste.** After the clipboard grant, `{"eval":"navigator.clipboard.writeText(\"abba\").then(()=>1)","wait":300},{"click":[285,731]},{"type":"x"}`, then the three CDP events `["Input.dispatchKeyEvent",{"type":"rawKeyDown","key":"Control","code":"ControlLeft","windowsVirtualKeyCode":17,"modifiers":2}]`, `["Input.dispatchKeyEvent",{"type":"rawKeyDown","key":"v","code":"KeyV","windowsVirtualKeyCode":86,"modifiers":2,"commands":["paste"]}]`, `["Input.dispatchKeyEvent",{"type":"keyUp","key":"Control","code":"ControlLeft","windowsVirtualKeyCode":17}]`: the run input reads `xabba`.

## Gotchas

- The dock grows after Load, so Play moves from (546, 725) to (556, 704).
- A blank page with a 300×150 canvas means startup failed: rerun and read the printed `console.error`. The known causes (missing `instant/wasm-bindgen`, WebGPU without an adapter, MSAA on WebGL) are listed in AGENTS.md.
- Ctrl shortcuts and paste depend on the vendored `iced_winit` (modifier routing, clipboard) and the paste shim in `index.html`. Drive Ctrl chords with trusted CDP key events (`rawKeyDown` for `Control`, then the key with `"modifiers": 2`, then `keyUp`); a paste needs `"commands": ["paste"]` on the `v` event, because CDP does not run the browser's editing command by itself.
- `ctrl+w` closes the browser tab in a real browser; do not use it to test tab closing on the web.
