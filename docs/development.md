# Development

## Toolchain

- Rust 1.88 or newer, with `rustup target add wasm32-unknown-unknown`.
- [Trunk](https://trunkrs.dev) 0.22: `cargo install trunk --locked`.
- A local starbooru-server (its `docs/development.md` › Run the server).
- Chromium or Playwright for visual checks.

## Run it

```sh
trunk serve --port 8081
```

Open http://127.0.0.1:8081 and sign in to `http://127.0.0.1:8080`. `STARBOORU_SERVER`, read at compile time, prefills the address.

To try it the way production runs it, from the server's own address:

```sh
trunk build --release
STARBOORU_WEB_DIR=$PWD/dist cargo run -p starbooru-server   # in starbooru-server
```

## Tests

- `cargo test` runs the logic on the host: `src/logic/`, the stores, and the check that every catalogue has every message with the same variables.
- `wasm-bindgen-test` covers what needs a browser (storage, file reading) and runs with `wasm-pack test --headless --chrome`.
- Behaviour against a real server is checked by hand and written in the dev log, until there are browser tests for it.

## Visual checks

1. `trunk build --release` and serve it from a local server with generated test pictures.
2. Drive a headless Chromium or Playwright through sign-in, the gallery, a post, upload and settings.
3. Screenshot each in light and dark, at 1280 px and 390 px.
4. Look at the PNGs next to the reference's (`docs/design.md`).
5. Show the owner before committing.

Headless Chromium draws in software, so blur and glows cost far more there than on a real card. Judge smoothness on a real browser.

## Releasing

Not set up yet. The plan: a tagged build attaches the `dist/` bundle to a release, and the server's deployment docs say how to mount it at `STARBOORU_WEB_DIR`.
