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
  app.rs        theme, OLED, accent and language on the root, and the routes
  route.rs      the navigation's tabs, which tab a path is under, search addresses
  api.rs        the shared Client, a session's or a guest's without a token, api::call (a 401 signs out),
                  errors in words
  instance.rs   what the server is (private or public, likes or stars) from /info, and whether it takes guests
  oidc.rs       sign-in through the identity provider, and linking it to the account, away and back to /oidc
  store.rs      session, preferences, the gallery's place in each search and the posts opened, in localStorage
  theme.rs      the accent as hue, chroma and lightness (--seed-*), the browser's bar colour (theme-color)
  power.rs      the battery saver: data-saver on the root, from the setting, the battery and data saver
  pace.rs       the effects: data-effects on the root, from the setting or from frame times while scrolling
  install.rs    registers the service worker, and keeps the browser's offer to install for Settings › Help and about
  updates.rs    what the server says of new versions, for administrators, and the notice at the top of the shell
  i18n.rs       t!() over the Fluent catalogues
  pages/        one module per page; shell.rs is the frame, post/ the viewer and its media editor, share/
                  the share sheet, the list of links and the public /s/ page, settings/ the Settings groups,
                  admin/ the administration's tabs, legal.rs the privacy policy and the terms
  components/   icons, brand, snackbar, tiles and their swipes, downloads, the tag box and its grip, a tag's menu,
                  the button that asks "Sure?", the links to the legal pages, the menu that looks a picture up on
                  other sites, the video player, the Back button of the pages under a tab, Markdown as elements,
                  the five stars of a vote, and what stands in for a page a guest or a public server can't use
  logic/        pure Rust with unit tests: server addresses, the gallery's lists and layout, zoom, the viewer's
                  tags, people, details and file names, the video player's clock, seek bar, keys and resting
                  controls, the word at the caret, the tag editor's text and who may edit,
                  the gallery's selection, picking by dragging and the mass edit it sends, which places in the gallery it remembers and
                  when two searches are the same, the posts opened lately, the booru browser's sites, addresses and
                  results, the library import's text, the media editor's crop, turns and trim, share links'
                  lifetimes, the days a picked end date takes, addresses and expiry text, reverse search's copy to
                  send and how each site gets it, Settings' groups and old anchors, accent, lists and password checks, swipes, the
                  administration's limits, chart, form checks and AI requests, the pages a tab went through,
                  a small Markdown reader, updates: the notice's rule, an update's steps, the reasons a
                  server can't update itself and the hour of automatic updates, stars: a press, the keys, the
                  average and the tallies shown before the server answers, and modes: what a switch lists, why it
                  was refused, which screen someone without a session gets and where a sign-in comes back to
locales/        en.ftl, it.ftl
style/          tokens.css, base.css, shell.css, player.css, pages/<page>.css
assets/         the icons, the web app manifest and the service worker (sw.js), copied to the root as they are
index.html      Trunk's entry, with the loading screen
```

Routes keep the first StarBooru app's paths, so links people saved still open: `/`, `/explore`, `/post/:id`, `/tags`, `/tag`, `/upload`, `/boorus`, `/boorus/post`, `/settings` and `/settings/<group>`, `/trash`, `/duplicates`, `/help`, `/admin`, `/oidc`, `/s/:token`, `/privacy`, `/terms`. The first thirteen show inside the shell; the rest open without signing in. A guest on a public server gets the shell too.

Settings is a list of groups (`logic::settings::Group`): Account and security, Appearance, Viewing and gestures, Battery and effects, Library, Sharing and API keys, Help and about, with the administration at the foot for administrators. From 1000 px the list stays beside the open group, and `/settings` shows Appearance; narrower, `/settings` is the list and each group is a page with a Back. An address from before the groups, `/settings#keys` or `/settings/keys`, is replaced by the group's with the card's anchor (`/settings/sharing#keys`). Choices that change looks are drawn as small pictures in the app's own roles: `tokens.css` gives `.scheme-light` and `.scheme-dark` the two themes' roles, so the theme pictures show the other theme too.

## Styles

Plain CSS, split by role. `index.html` links each file with `<link data-trunk rel="css">` in this order:

| File | Holds |
|---|---|
| `style/tokens.css` | Colour roles for light, dark and the system's dark, all in OKLCH from the accent's hue (`--seed-hue`) and, for a picked colour, its chroma and lightness (`--seed-chroma`, `--seed-lightness`): a grey makes a palette without a tint. `[data-oled]` blacks out the dark roles. Radii, easing, durations, shadows; `.glass` |
| `style/base.css` | Base elements and the shared components: buttons, switches, fields and the tag box, segmented choices, chips, cards, banners, spinners, Markdown, the snackbar, the tag menu, keyframes and the reduced-motion fallback |
| `style/shell.css` | The top bar, the brand and search capsules, the navigation slider in its three places, the content area and the update notice |
| `style/player.css` | The video player, shared by the viewer and the share page; its narrow layouts are container queries on the player's own width |
| `style/pages/<page>.css` | One per page, linked after the four above when the page lands |

Component CSS uses only the role variables, never a literal colour. The navigation sits at the top right from 1530 px, as a rail at the side from 761 to 1529 px, and at the bottom at 760 px and below.

## How it works

- **Media.** `<img>` and `<video>` load signed `/media` URLs directly, so the browser cache does the caching. The server rounds their expiry to 6-hour buckets, so a URL stays the same, and cached, that long.
- **Video.** The viewer and the share page play videos in `components/player.rs`, with the app's controls in place of the browser's. The page around it owns fullscreen, so the viewer's arrows and keys keep working there. The player hears keys on the document in the capture phase, before the viewer's on the window, and the viewer skips the ones it took: the arrows step between posts, Shift with them (or J and L) seeks. Videos up to a minute long loop unless told otherwise. Volume and mute are kept in the store; the speed is kept for the tab.
- **Long lists.** Only the tiles within a screen and a half of the view are in the DOM. A tile shows its post's main colour until the thumbnail arrives. A drag across tiles to pick them finds the tile under the pointer from the layout, by its place in the list, so tiles that scroll out of the DOM stay in the range. The tile the drag started on stays in the DOM until it ends, as a finger's events go to it.
- **Motion.** CSS transitions and the Web Animations API. View Transitions where the browser has them, with a plain fallback. Everything heavy goes quiet under `prefers-reduced-motion` and the battery saver.
- **Uploads.** `Client::upload_file` sends files in chunks and resends a failed one. The queue lives in a signal at the root, so it keeps going while the user moves between pages.
- **Private and public.** `instance.rs` keeps what `/info` said last: the mode, the score kind and whether guests may browse. Until it answers, a server counts as private with likes. The shell asks when it starts, `api::server_info` again every two minutes, and a switch in Administration › Server puts the new mode in at once and asks again, so every view that reads it follows. A public server counts likes whatever is stored, as the server does.
- **Guests.** Without a session, the shell asks the server the sign-in screen would start with whether it takes guests. If it does, the pages open without signing in, and requests go out with a client that has no token; nothing is kept per account (no history, no saved searches, no uploads). The navigation has Gallery, Explore, Tags and Settings (Appearance and Help only), then Sign in. Upload, the recycle bin, the duplicates and the administration show a card that leads to sign-in instead. A vote, from the viewer or a tile's menu, asks to sign in. The sign-in screen then opens over the page with a way back, and signing in, with a password or through the identity provider (which keeps the page in its handle), stays on that page. A guest's 401 asks the server again whether it still takes guests; a 429 the client doesn't wait out says to slow down and when to try again. On a private server nothing changes: the sign-in screen comes first.
- **Stars.** On a private server set to stars, the viewer's vote is five stars (`components/stars.rs`) with the post's average and count; the menu's like gives five, and a sort by score reads as by rating. The tallies change at once and go back if the server turns the vote down, as likes do.
- **Sharing on a public server.** The share sheet gives the post's own address (`/post/{id}`), or several, one a line, to copy or hand to the browser's share sheet. Settings › Sharing has no list of links there.
- **Explore.** A private server has no Trending or Popular tags, so Explore shows the posts added on this day in earlier years and random picks, and asks for a one-day window, as only the two missing sections read it. A public one leads with Trending, over the window in `?days=` (today, a week, a month or a year, a week by default), and Popular tags, and has no boorus side: the `/boorus` pages say so. While it's open, the top bar's search box narrows it (`?q=`) instead of searching the gallery.
- **The gallery's place.** The store keeps, per account, the tile at the top of the screen in each search, with its offset, the scroll and how many posts were loaded. A saved search keeps its place however many searches come after it, up to the server's 200; the others keep the latest 30 among themselves, and a search that is no longer saved ages out with them. Until the account's saved searches arrive from the server, it keeps up to 230 places, so none of theirs is lost. Searches are matched by what they find (`logic::saved::search_key`, from the parsed query), so a chip's tidied search and an address with other spacing or term order share one place.
- **Recently viewed.** Each post the viewer loads, previous and next included, goes first in a list of up to 60 ids kept in the store under the account's owner key; booru posts and share pages don't count, and signing out empties it. Explore shows it first, 12 at a time. The API has no way to fetch several posts by id (the search syntax has no `id:` term), so each tile is a `GET /posts/{id}`, all sent at once. Posts that come back missing or in the recycle bin leave the list quietly. A search can't narrow a list of ids, so the section steps aside while `?q=` is set. Clear empties it, with Undo in the snackbar.
- **Other boorus.** The server searches them (`/remote/search`), keeps the accounts and relays the pictures a site won't serve to other pages. The browser keeps the last results for a few hours while the tab is open, and saving a post queues an import on the server.
- **Reverse search.** Find source, in the viewer's bar and a tile's menu, looks the post's thumbnail up on Google Lens, iqdb, SauceNAO or trace.moe. The thumbnail is read from the server as the menu opens, and nothing goes to a site until one is picked. iqdb and SauceNAO get it from a form posted into a new tab. Lens and trace.moe get its signed address when the server's host is on the internet; for a server at home, a private network or a VPN, the app copies it as PNG and opens the site to paste it (on a touch screen, or without a clipboard for pictures, it saves it instead). Lens doesn't take an upload: in the EU, Google's consent page swallows a form posted from another site.
- **Installing.** `manifest.webmanifest` makes the app installable: standalone, starting at `/`, in the default light canvas, with a maskable icon for Android. Chromium offers installing with `beforeinstallprompt`; the page catches it even before the app starts, and Settings › Help and about shows Install StarBooru until the offer is taken, turned down or the app runs in its own window. Safari and Firefox make no offer, so nothing shows there. An installed app has no browser Back button, so every page has its own way back: the viewer and the booru post their Back, Trash, Duplicates, Help, the administration and a tag's page an arrow before the title (back through history when the page before is in the app, else up to Settings' group they belong to or Tags), a Settings group on a phone the same arrow back to the list, the legal pages theirs, and a share page's brand leads home.
- **Service worker.** `assets/sw.js` lets the app start without a network, and nothing more. The app registers it after the page has loaded. It answers two kinds of request:
  - A page (any navigation) comes from the network, and the copy of `index.html` it keeps only when the network fails or the server answers 5xx. Online, a new version is always the one that starts.
  - A file of the bundle (`/<name>.js`, `.wasm`, `.css`, `.png`, `.svg`, `.webmanifest`) comes from the cache, else from the network. The cache holds the files the last `index.html` names: each new page drops the files it no longer names, fetches the ones it names, and fetches the unhashed icons and manifest again.
  - Everything else goes to the network untouched, with no `respondWith`: `/api/`, `/media/`, `/remote/`, `/backups/`, `/healthz`, other origins (a session on another server included), and every request that isn't a GET. Nothing the API or `/media` returns is ever stored, so the cache holds no private data.
  - A new `sw.js` installs and takes over at once, and its activation deletes the app's other caches; bump `CACHE` when the worker starts keeping things another way. Offline, the shell starts and each page says it can't reach the server, as for any failed request.
- **Updates.** For administrators only, the shell asks `GET /admin/updates` when the app starts and every 3 hours while it stays open, and keeps the answer in `updates.rs`. When a newer release is out, a pill under the top bar names it and leads to Administration › Updates; it stays off the viewers and that page. Hiding it is kept in the store per account and version, so it comes back for a newer one. A server older than 0.2.0 answers 404: the notice and the Updates tab stay away. The page shows the notes through `logic::markdown`, which builds elements and never HTML; only `http:`, `https:` and `mailto:` links survive, and pictures become their description. After Update now it asks every 1.5 s; once the run reaches `restarting` it knocks on `/info` every 2 s until the new version answers, then reloads the page, so the new version's bundle loads (navigations are network-first in the service worker).
- **Share link end dates.** The server takes a link's lifetime in whole days from when it makes it. A day picked on the calendar becomes the days from now to the end of that day on the browser's clock, rounded up, so the link ends on the day after, at the time of day it was made.

The server decides which booru pictures it relays: the previews and samples of sites that serve them only to their own pages come as its signed `/api/v1/remote/media` addresses. Everything else loads in the browser straight from the booru, as the privacy page says, so the app shows a picture only from one of those addresses or over `https:`. Anything else (`http:`, `data:`, `//host`) gets the tile's colour instead.

## Open questions

- starbooru-server's `web.rs` caches files for a year only when `dx` named them (`<name>-dxh<hex>.<ext>` under `/assets/`). Trunk names them `<name>-<hash>.<ext>`. Teach `web.rs` Trunk's names in starbooru-server, or the bundle is checked on every visit.
