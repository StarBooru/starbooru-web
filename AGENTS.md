# AGENTS.md: guide for AI agents and contributors

starbooru-web is the StarBooru web app: Rust compiled to WebAssembly with [Leptos](https://leptos.dev), rendering real DOM in the browser. It talks to [starbooru-server](https://github.com/StarBooru/starbooru-server) over its JSON API and nothing else.

There is no code yet, only these docs. Read this file first, then `docs/`.

## 1. Orientation

| Read | Why |
|---|---|
| `docs/architecture.md` | Stack, layout of the crate, how it reaches the server |
| `docs/development.md` | Toolchain, running against a local server, tests, visual checks |
| `docs/roadmap.md` | What to build, in order |

The API is in starbooru-server's `docs/api.md`. The owner's backlog is its `docs/checklist.md`.

## 2. Related repositories

| Repository | Role |
|---|---|
| [starbooru-server](https://github.com/StarBooru/starbooru-server) | The server, plus `starbooru-core` (API types, search parser, validators) and `starbooru-client` (typed HTTP client, wasm and native) |
| `starbooru-android`, `starbooru-windows`, `starbooru-linux` | The other apps, built on the same crates |

## 3. Golden rules

### Never crash

- Non-test code never panics: no `unwrap()`, `expect()`, `panic!`, `unreachable!`, `todo!` or `unimplemented!`. Copy the workspace lints and `clippy.toml` from starbooru-server.
- No `unsafe`: set `unsafe_code = "forbid"`.
- A failed request leaves the page usable and says what went wrong in words the user can act on.

### Project rules

1. **The API is the contract.** Use `starbooru-core` and `starbooru-client` from starbooru-server, pinned to a tag. Don't redefine API types here. A missing endpoint is added in starbooru-server first.
2. **The app talks only to the API.** It reaches media through the signed `/media` URLs the API returns.
3. **Design tokens, not literals.** Colours are CSS role variables in `style/tokens.css`, derived from one seed hue. Glass (blur) only on the bars, the viewer's floating controls and the sign-in card. Respect `prefers-reduced-motion`.
4. **Verify UI changes visually.** Light and dark, 1280 px and 390 px wide, then look at the PNGs (`docs/development.md` › Visual checks). Before committing anything that changes how the app looks, show the owner a preview and commit only once they confirm.
5. **Tests are the gate.** Logic worth a branch lives in plain functions with unit tests, outside components.
6. **Privacy.** Never log tokens, passwords or file contents. Test images are generated or public domain, never personal photos.
7. **Assets.** Third-party assets must be permissively licensed and get a row in `ATTRIBUTION.md`. Never commit font files.
8. **Text.** Every string the app shows is in a Fluent catalogue (`locales/en.ftl`, `it.ftl`), and a test keeps the catalogues in step.
9. **Time goes through `chrono`.**

## 4. Before you finish a task

```sh
cargo fmt --all
cargo test
cargo clippy --target wasm32-unknown-unknown --all-targets -- -D warnings
trunk build --release
```

Then append a terse entry to `log/devlog.md`, which stays local (`/log` goes in `.gitignore` with the first code): what landed, what you verified, what's still open. Update `docs/roadmap.md` when an item changes state.

## 5. Notes

- License: AGPL-3.0-or-later, like the server. Add `LICENSE` with the first code.
- The product name is written StarBooru in user-facing text. Machine names stay lowercase.
- "The reference" in code comments and test names is the first StarBooru app, which this one replaced. Its links and the settings it saved in the browser still work here.
