# Design

The archived [StarBooru](https://github.com/StarBooru/StarBooru) app is the visual reference: how each page looks, what it holds and how it moves. It is read-only. Clone it to look, to run it, or to take files from it.

| In the reference | For |
|---|---|
| `docs/showcase/` | Screenshots of the main pages, light and dark, desktop and phone |
| `apps/app/assets/main.css` | The tokens (colour roles from `--seed-hue` in OKLCH, radii, easing, durations) and every component's styles |
| `apps/app/src/pages/` | What each page holds and does |
| `apps/app/src/motion.rs`, `gesture.rs` | How things move: a post grows out of its tile, swipes follow the finger 1:1 |
| `apps/app/locales/` | The UI text in English and Italian |
| `docs/design-ui-ux/` | The design guide the reference was built with |
| `docs/branding.md`, `docs/brand/` | The logo, its two cuts and its colours |
| `docs/roadmap.md` | What every feature did when the apps split |

To run it, follow its `docs/development.md` (`dx serve --web -p starbooru-app`) against a local server.

## What carries over

The web app is the closest to the reference, as both are HTML and CSS. Keep:

- every colour derived from one seed hue, with light, dark and system themes and the user's accent;
- the floating capsules: the brand, the search centred on the window, and a navigation slider of at most five items, at the top right from 1530 px, at the side down to 761 px and at the bottom on phones;
- glass only on the bars, the viewer's controls and the sign-in card;
- spring easing, and the Effects and Battery saver settings, which take the blur and the entrances away;
- the icons and their attribution rows (MingCute and the others in the reference's `ATTRIBUTION.md`);
- system fonts ("Roboto Flex", Roboto, Segoe UI, system-ui), never font files.

Port the styles page by page with the markup they belong to, instead of copying `main.css` whole.

## What can change

Some of the reference's motion is script because WebKitGTK aborted in View Transitions and the webview was slow to paint. A browser has no such limit, so use CSS, the Web Animations API and View Transitions directly. Anything that looks different from the reference on purpose goes past the owner first.
