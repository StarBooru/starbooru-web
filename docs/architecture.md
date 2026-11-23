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
  main.rs       loads the store, picks the language, mounts the app
  app.rs        theme and language on the root, and the routes
  route.rs      the navigation's tabs, which tab a path is under, search addresses
  api.rs        the shared Client, api::call (a 401 signs out), errors in words
  oidc.rs       sign-in through the identity provider, away and back to /oidc
  store.rs      session and preferences in localStorage
  theme.rs      the browser's bar colour (theme-color) from the seed hue
  i18n.rs       t!() over the Fluent catalogues
  pages/        one module per page; shell.rs is the frame, soon.rs the placeholder, post/ the viewer and
                  its media editor
  components/   icons, brand, snackbar, tiles, downloads, the tag box and its grip, a tag's menu
  logic/        pure Rust with unit tests: server addresses, the gallery's lists and layout, zoom, the viewer's
                  tags, details and file names, the word at the caret, the tag editor's text and who may edit,
                  the gallery's selection and the mass edit it sends, the booru browser's sites, addresses and
                  results, the library import's text, and the media editor's crop, turns and trim
locales/        en.ftl, it.ftl
style/          tokens.css, base.css, shell.css, pages/<page>.css
index.html      Trunk's entry, with the loading screen
```

Routes keep the reference's paths, so links people saved still open: `/`, `/explore`, `/post/:id`, `/tags`, `/tag`, `/upload`, `/boorus`, `/boorus/post`, `/settings`, `/trash`, `/duplicates`, `/help`, `/admin`, `/oidc`, `/s/:token`, `/privacy`, `/terms`. The first thirteen show inside the shell. A page that isn't built yet shows `Soon` with its title; it lands by replacing that view in `app.rs`.

## Styles

Plain CSS, split by role. `index.html` links each file with `<link data-trunk rel="css">` in this order:

| File | Holds |
|---|---|
| `style/tokens.css` | Colour roles for light, dark and the system's dark, all from `--seed-hue` in OKLCH; radii, easing, durations, shadows; `.glass` |
| `style/base.css` | Base elements and the shared components: buttons, switches, fields and the tag box, segmented choices, chips, cards, banners, spinners, the snackbar, the tag menu, keyframes and the reduced-motion fallback |
| `style/shell.css` | The top bar, the brand and search capsules, the navigation slider in its three places, the content area |
| `style/pages/<page>.css` | One per page, linked after the three above when the page lands |

Component CSS uses only the role variables, never a literal colour. The navigation sits at the top right from 1530 px, as a rail at the side from 761 to 1529 px, and at the bottom at 760 px and below.

## How it works

- **Media.** `<img>` and `<video>` load signed `/media` URLs directly, so the browser cache does the caching. The server rounds their expiry to 6-hour buckets, so a URL stays the same, and cached, that long.
- **Long lists.** Only the tiles within a screen and a half of the view are in the DOM, as in the reference gallery. A tile shows its post's main colour until the thumbnail arrives.
- **Motion.** CSS transitions and the Web Animations API. View Transitions where the browser has them, with a plain fallback. Everything heavy goes quiet under `prefers-reduced-motion` and the battery saver.
- **Uploads.** `Client::upload_file` sends files in chunks and resends a failed one. The queue lives in a signal at the root, so it keeps going while the user moves between pages.
- **Other boorus.** The server searches them (`/remote/search`), keeps the accounts and relays the pictures a site won't serve to other pages. The browser keeps the last results for a few hours while the tab is open, and saving a post queues an import on the server.

## Open questions

- starbooru-server's `web.rs` caches files for a year only when `dx` named them (`<name>-dxh<hex>.<ext>` under `/assets/`). Trunk names them `<name>-<hash>.<ext>`. Teach `web.rs` Trunk's names in starbooru-server, or the bundle is checked on every visit.
