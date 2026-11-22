# Architecture

```text
 browser ── starbooru-web (Leptos, wasm) ── HTTPS: /api/v1 and signed /media URLs ──▶ starbooru-server
```

A single-page app compiled to WebAssembly. Leptos renders it into the DOM with fine-grained signals, so a change updates only the nodes that read it, with no virtual DOM to diff. There is no server-side rendering. starbooru-server serves the built files from `STARBOORU_WEB_DIR`, at the same address as the API.

The same address matters twice. The app calls the API at its own origin by default, and provider sign-in only returns to a page under `STARBOORU_PUBLIC_URL`. The API allows any origin (bearer tokens, no cookies), so `trunk serve` on another port works in development.

## Crates

| Crate | From | Role |
|---|---|---|
| `leptos` 0.8 (`csr`), `leptos_router` | crates.io | Components, signals, routes |
| `starbooru-core` | starbooru-server, by tag | API models, search query parser, tag rules, `password_is_weak` |
| `starbooru-client` | starbooru-server, by tag | One async method per endpoint, chunked uploads, waits out `429` |
| `fluent-bundle` | crates.io | UI text |
| `web-sys`, `gloo-*` | crates.io | Storage, clipboard, drag and drop, observers |

```toml
starbooru-core = { git = "https://github.com/StarBooru/starbooru-server", tag = "<release>" }
starbooru-client = { git = "https://github.com/StarBooru/starbooru-server", tag = "<release>" }
```

## Layout

```text
src/
  main.rs       mounts the app
  api.rs        the shared Client and the session signal
  store.rs      session and preferences in localStorage
  i18n.rs       t!() over the Fluent catalogues
  pages/        one module per page
  components/   tiles, chips, menus, the tag box, the viewer stage
  logic/        pure Rust with unit tests: tile layout, zoom, the word at the caret, file names
locales/        en.ftl, it.ftl
style/          tokens and components
index.html      Trunk's entry, with the loading screen
```

Routes keep the reference's paths, so links people saved still open: `/`, `/explore`, `/post/:id`, `/tags`, `/tag`, `/upload`, `/boorus`, `/boorus/post`, `/settings`, `/trash`, `/duplicates`, `/help`, `/admin`, `/oidc`, `/s/:token`, `/privacy`, `/terms`.

## How it works

- **Media.** `<img>` and `<video>` load signed `/media` URLs directly, so the browser cache does the caching. The server rounds their expiry to 6-hour buckets, so a URL stays the same, and cached, that long.
- **Long lists.** Only the tiles within a screen and a half of the view are in the DOM, as in the reference gallery. A tile shows its post's main colour until the thumbnail arrives.
- **Motion.** CSS transitions and the Web Animations API. View Transitions where the browser has them, with a plain fallback. Everything heavy goes quiet under `prefers-reduced-motion` and the battery saver.
- **Uploads.** `Client::upload_file` sends files in chunks and resends a failed one. The queue lives in a signal at the root, so it keeps going while the user moves between pages.

## Open questions

- starbooru-server's `web.rs` caches files for a year only when `dx` named them (`<name>-dxh<hex>.<ext>` under `/assets/`). Trunk names them `<name>-<hash>.<ext>`. Teach `web.rs` Trunk's names in starbooru-server, or the bundle is checked on every visit.
