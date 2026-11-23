# Roadmap

_Status as of 2026-11-23: sections 1 to 3 are done: the app does everything the first StarBooru app did. Section 4 (polish) is next. Scope comes from starbooru-server's `docs/checklist.md`, and sections 5 to 9 from the owner's list of 2026-11-23. Items marked (server) wait for starbooru-server, whose `docs/roadmap.md` has its side._

## 1. Foundation

- [x] Crate, lints, Trunk build, CI
- [x] Tokens and themes from one seed hue; the shell (brand, search, navigation)
- [x] Sign-in with a password and with an identity provider
- [x] Gallery: grid, infinite scroll, sorts
- [x] Viewer: image, GIF and video, zoom, previous and next

## 2. Daily use

- [x] Search with tag autocomplete, typo correction and the cheat sheet (`/help`)
- [x] Tag editor, the Tags page and each tag's page
- [x] Upload: drag and drop, folders, paste, links, chunks with progress
- [x] Explore, saved searches, votes, related posts
- [x] Recycle bin, mass edit

## 3. The rest of the first app

- [x] Other boorus and library imports
- [x] People in a post (figures)
- [x] Image and video editor
- [x] Duplicate finder
- [x] Share links and the public share page (`/s/:token`)
- [x] Settings: theme, accent, language, sessions, password, API keys
- [x] Administration: overview, accounts, server, AI, backups
- [x] Privacy and terms pages

## 4. Private and public booru

An instance runs in one of two modes: private, as today, or public, open to other people with their own accounts.

Switching:

- [ ] A switch between the two modes in Administration (server)
- [ ] Switching is destructive, so it first offers a backup made for the switch, on by default (server)

Private mode:

- [ ] Scores as likes and dislikes by default, or 0 to 5 stars as an option (server)
- [x] Explore without Trending and Popular tags

Public mode:

- [ ] Accounts with a profile and a profile picture (server)
- [ ] An account button with the profile picture beside the navigation; Upload and Settings move under it
- [ ] A forum, in Upload's place in the navigation (server)
- [ ] Votes on tags (server)
- [ ] No Boorus side in Explore, and no way to turn it on (server)

Both modes:

- [ ] Share links set up to suit each mode (server)
- [ ] A tag blacklist in Settings (server)
- [ ] An optional home page that can be turned on or off and customised. The default, in StarBooru's look, has the logo above a search bar, over a moving background of the most popular posts rated safe (server)

## 5. People

- [ ] A setting to turn People on and off
- [ ] AI that finds faces on upload, anime ones included: it keeps a crop of each face and fills in the name, the franchise, whether it's a cosplay, and the clothing and looks (server)
- [ ] A franchise (copyright) on each person, to group people by franchise (server)
- [ ] People in Explore (server)
- [ ] Popular characters in Explore, in public mode (server)

## 6. Browsing and sharing

- [ ] Explore has no search field of its own: the top bar's search narrows Explore while it's open
- [ ] The last posts opened (history) in Explore
- [ ] The staggered view fills left to right before top to bottom, so the next post in the viewer is the tile beside it
- [ ] Click and drag across tiles to select several
- [ ] A saved search reopens where it was left. Today the gallery remembers only the last 30 searches, so a saved one can drop out
- [ ] Share links: a custom expiry date besides 1, 7 and 30 days
- [ ] Reverse search an image from a post on google images, iqdb, saucenao, trace.moe 

## 7. Design

- [ ] Any accent colour, greys and white included
- [ ] OLED mode, with a black background
- [ ] A video player of our own design
- [ ] Redesign Settings, the Tags page, the tag editor in the viewer and saved searches

## 8. Polish

- [ ] Smooth on a Raspberry Pi 4 in Chromium and Firefox
- [ ] Effects (automatic, full, reduced) and battery saver
- [ ] English and Italian, complete

## 9. Install

- [ ] Installable as a progressive web app in Chrome, a web app in Firefox and a Safari web app

## Ideas

- Waifu wars, looked at only once the rest is done
