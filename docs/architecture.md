# Architecture

```text
 browser ── starbooru-web (Leptos, wasm) ── HTTPS: /api/v1 and signed /media URLs ──▶ starbooru-server
```

A single-page app compiled to WebAssembly. Leptos renders it into the DOM with fine-grained signals, so a change updates only the nodes that read it, with no virtual DOM to diff. There is no server-side rendering. starbooru-server serves the built files from `STARBOORU_WEB_DIR`, at the same address as the API.

The same address matters three times. The app calls the API at its own origin by default, provider sign-in only returns to a page under `STARBOORU_PUBLIC_URL`, and a share link (`/s/:token`) is opened by someone with no session at all, so that page asks the origin it was served from, with a client that has no token. The API allows any origin (bearer tokens, no cookies), so `trunk serve` on another port works in development.

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
  oidc.rs       sign-in through the identity provider, and linking it to the account, away and back to /oidc
  store.rs      session, preferences, the gallery's place in each search and the posts opened, in localStorage
  theme.rs      the browser's bar colour (theme-color) from the seed hue, and the hue of a picked accent
  power.rs      the battery saver: data-saver on the root, from the setting, the battery and data saver
  pace.rs       the effects: data-effects on the root, from the setting or from frame times while scrolling
  i18n.rs       t!() over the Fluent catalogues
  pages/        one module per page; shell.rs is the frame, post/ the viewer and its media editor, share/
                  the share sheet, the list of links and the public /s/ page, settings/ the Settings cards,
                  admin/ the administration's tabs, legal.rs the privacy policy and the terms
  components/   icons, brand, snackbar, tiles and their swipes, downloads, the tag box and its grip, a tag's menu,
                  the button that asks "Sure?", the links to the legal pages, the menu that looks a picture up on
                  other sites
  logic/        pure Rust with unit tests: server addresses, the gallery's lists and layout, zoom, the viewer's
                  tags, people, details and file names, the word at the caret, the tag editor's text and who may edit,
                  the gallery's selection and the mass edit it sends, the posts opened lately, the booru browser's sites, addresses and
                  results, the library import's text, the media editor's crop, turns and trim, share links'
                  lifetimes, the days a picked end date takes, addresses and expiry text, reverse search's copy to
                  send and how each site gets it, Settings' accent, lists and password checks, swipes, and the
                  administration's limits, chart, form checks and AI requests
locales/        en.ftl, it.ftl
style/          tokens.css, base.css, shell.css, pages/<page>.css
index.html      Trunk's entry, with the loading screen
```

Routes keep the first StarBooru app's paths, so links people saved still open: `/`, `/explore`, `/post/:id`, `/tags`, `/tag`, `/upload`, `/boorus`, `/boorus/post`, `/settings`, `/trash`, `/duplicates`, `/help`, `/admin`, `/oidc`, `/s/:token`, `/privacy`, `/terms`. The first thirteen show inside the shell; the rest open without signing in.

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
- **Long lists.** Only the tiles within a screen and a half of the view are in the DOM. A tile shows its post's main colour until the thumbnail arrives.
- **Motion.** CSS transitions and the Web Animations API. View Transitions where the browser has them, with a plain fallback. Everything heavy goes quiet under `prefers-reduced-motion` and the battery saver.
- **Uploads.** `Client::upload_file` sends files in chunks and resends a failed one. The queue lives in a signal at the root, so it keeps going while the user moves between pages.
- **Explore.** A private booru has no Trending or Popular tags, and every instance counts as private until the server reports a mode, so Explore shows the posts added on this day in earlier years and random picks. It asks for a one-day window, as only the two missing sections read it. While it's open, the top bar's search box narrows it (`?q=`) instead of searching the gallery.
- **Recently viewed.** Each post the viewer loads, previous and next included, goes first in a list of up to 60 ids kept in the store under the account's owner key; booru posts and share pages don't count, and signing out empties it. Explore shows it first, 12 at a time. The API has no way to fetch several posts by id (the search syntax has no `id:` term), so each tile is a `GET /posts/{id}`, all sent at once. Posts that come back missing or in the recycle bin leave the list quietly. A search can't narrow a list of ids, so the section steps aside while `?q=` is set. Clear empties it, with Undo in the snackbar.
- **Other boorus.** The server searches them (`/remote/search`), keeps the accounts and relays the pictures a site won't serve to other pages. The browser keeps the last results for a few hours while the tab is open, and saving a post queues an import on the server.
- **Reverse search.** Find source, in the viewer's bar and a tile's menu, looks the post's thumbnail up on Google Lens, iqdb, SauceNAO or trace.moe. The thumbnail is read from the server as the menu opens, and nothing goes to a site until one is picked. iqdb and SauceNAO get it from a form posted into a new tab. Lens and trace.moe get its signed address when the server's host is on the internet; for a server at home, a private network or a VPN, the app copies it as PNG and opens the site to paste it (on a touch screen, or without a clipboard for pictures, it saves it instead). Lens doesn't take an upload: in the EU, Google's consent page swallows a form posted from another site.
- **Share link end dates.** The server takes a link's lifetime in whole days from when it makes it. A day picked on the calendar becomes the days from now to the end of that day on the browser's clock, rounded up, so the link ends on the day after, at the time of day it was made.

The server decides which booru pictures it relays: the previews and samples of sites that serve them only to their own pages come as its signed `/api/v1/remote/media` addresses. Everything else loads in the browser straight from the booru, as the privacy page says, so the app shows a picture only from one of those addresses or over `https:`. Anything else (`http:`, `data:`, `//host`) gets the tile's colour instead.

## Open questions

- starbooru-server's `web.rs` caches files for a year only when `dx` named them (`<name>-dxh<hex>.<ext>` under `/assets/`). Trunk names them `<name>-<hash>.<ext>`. Teach `web.rs` Trunk's names in starbooru-server, or the bundle is checked on every visit.
