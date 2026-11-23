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

## Installing and offline

The service worker (`assets/sw.js`) runs wherever the app does on `http://127.0.0.1` or `localhost`, which browsers count as secure: under `trunk serve` and on any server that serves `dist/`. On any other address it needs HTTPS. It fetches pages from the network first, so a rebuild shows on the next reload; DevTools › Application › Service workers can unregister it.

To check installing in Chromium, open DevTools › Application › Manifest (it lists any problem) and use Install in the address bar or Settings › About › Install StarBooru. Over CDP, `Page.getAppManifest` and `Page.getInstallabilityErrors` say the same. Headless Chromium fires `beforeinstallprompt` but never settles the prompt, so tests dispatch their own event with `prompt()` and `userChoice`.

To check offline, open the app once, then tick Offline in DevTools › Network (or `context.setOffline(true)` in Playwright) and reload: the shell starts from the cache, and each page says it can't reach the server. Back online, Try again brings it back. For an update, rebuild, reload once online, and the new wasm loads; offline after that, the new version starts.

## Tests

- `cargo test` runs the logic on the host: `src/logic/`, the stores, and the check that every catalogue has every message with the same variables.
- `wasm-bindgen-test` covers what needs a browser (storage, file reading) and runs with `wasm-pack test --headless --chrome`.
- Behaviour against a real server is checked by hand and written in the dev log, until there are browser tests for it.

## Visual checks

1. `trunk build --release` and serve it from a local server with generated test pictures.
2. Drive a headless Chromium or Playwright through sign-in, the gallery, a post, upload and settings.
3. Screenshot each in light and dark, at 1280 px and 390 px.
4. Look at the PNGs, then show the owner before committing.

Headless Chromium draws in software, so blur and glows cost far more there than on a real card. Judge smoothness on a real browser.

## Releasing

`.github/workflows/release.yml` builds the bundle on a push of a `v*` tag and puts it in a draft release: `starbooru-web-<version>.tar.gz`, with `LICENSE` and `ATTRIBUTION.md` beside the files, and a `SHA256SUMS`. A hyphen in the tag (`v0.2.0-beta.1`) makes it a prerelease. Running the workflow by hand builds without releasing.

The build sets `STARBOORU_WEB_VERSION` to the tag. Write the notes, then publish with `gh release edit <tag> --draft=false`.

To serve a release, unpack it and point the server at the folder:

```sh
tar -xzf starbooru-web-<version>.tar.gz
STARBOORU_WEB_DIR=$PWD/starbooru-web-<version> starbooru-server
```
