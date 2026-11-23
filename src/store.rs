//! What the app keeps in the browser: the session and the preferences, as JSON
//! in `localStorage["starbooru"]`. The shape is the reference app's, so a
//! browser that used it keeps its settings:
//!
//! ```json
//! { "session": { "server": "…", "token": "…", "user": { … } } | null,
//!   "prefs": { "theme": "system", "accent_hue": 285, … },
//!   "memory": { "owner": "server|user id", "spots": [{ "q": "cat", … }] } }
//! ```
//!
//! Reading never fails: a field this version can't read takes its default, and
//! one it doesn't know is kept and written back as it came.

use std::cell::RefCell;

use leptos::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use starbooru_core::color::Rgb;
use starbooru_core::{AuthResponse, User};

use crate::i18n::Lang;
use crate::logic::address;

/// The signed-in account. Sign-in fills it in.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub server: String,
    pub token: String,
    pub user: User,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    /// Follows the system. Also what an unknown value (from a newer version)
    /// reads as. Last, as serde wants that one at the end.
    #[default]
    #[serde(other)]
    System,
}

impl Theme {
    /// In the order Settings offers them.
    pub const ALL: [Theme; 3] = [Theme::System, Theme::Light, Theme::Dark];

    /// The value of `data-theme` on `.app`, which `style/tokens.css` reads.
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::System => "system",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}

/// The hand the one-handed controls are for: the tablet navigation rail and
/// the editor's tools go on its side.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Hand {
    Left,
    Right,
    /// The usual layout: the rail on the left, the tools on the right. Also
    /// what an unknown value (from a newer version) reads as.
    #[default]
    #[serde(other)]
    Either,
}

impl Hand {
    pub const ALL: [Hand; 3] = [Hand::Either, Hand::Left, Hand::Right];

    /// The value of `data-hand` on `.app`, which the styles read.
    pub fn as_str(self) -> &'static str {
        match self {
            Hand::Either => "either",
            Hand::Left => "left",
            Hand::Right => "right",
        }
    }
}

/// The battery saver (`power.rs`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Saver {
    Always,
    Never,
    /// On while the battery is low or the device saves data. Also what an
    /// unknown value (from a newer version) reads as.
    #[default]
    #[serde(other)]
    Auto,
}

impl Saver {
    pub const ALL: [Saver; 3] = [Saver::Auto, Saver::Always, Saver::Never];
}

/// How much the app spends on looks (`pace.rs`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effects {
    Full,
    /// Plain surfaces for the glass, no entrance for new thumbnails and no
    /// glow behind a picture.
    Lite,
    /// Full until scrolling stutters on this device. Also what an unknown
    /// value (from a newer version) reads as.
    #[default]
    #[serde(other)]
    Auto,
}

impl Effects {
    pub const ALL: [Effects; 3] = [Effects::Auto, Effects::Full, Effects::Lite];
}

/// What swiping a gallery thumbnail sideways does. Off until chosen in
/// Settings, as a sideways swipe is easy to make by mistake.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SwipeAction {
    Like,
    Select,
    Share,
    Download,
    Delete,
    /// Also what an unknown value (from a newer version) reads as.
    #[default]
    #[serde(other)]
    Off,
}

impl SwipeAction {
    pub const ALL: [SwipeAction; 6] = [
        SwipeAction::Off,
        SwipeAction::Like,
        SwipeAction::Select,
        SwipeAction::Share,
        SwipeAction::Download,
        SwipeAction::Delete,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            SwipeAction::Off => "off",
            SwipeAction::Like => "like",
            SwipeAction::Select => "select",
            SwipeAction::Share => "share",
            SwipeAction::Download => "download",
            SwipeAction::Delete => "delete",
        }
    }

    pub fn parse(s: &str) -> Option<SwipeAction> {
        Self::ALL.into_iter().find(|a| a.as_str() == s)
    }
}

/// The shape of the gallery's tiles. Staggered tiles keep the picture's own
/// proportions, within the limits in `logic/layout.rs`: portrait ones are at
/// least square, landscape ones at most square, and `StaggeredAll` takes both.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GalleryView {
    Square,
    Landscape,
    Portrait,
    StaggeredLandscape,
    StaggeredAll,
    /// As on Pinterest. Also what an unknown value (from a newer version)
    /// reads as.
    #[default]
    #[serde(other)]
    StaggeredPortrait,
}

impl GalleryView {
    pub const FIXED: [GalleryView; 3] = [
        GalleryView::Square,
        GalleryView::Landscape,
        GalleryView::Portrait,
    ];
    pub const STAGGERED: [GalleryView; 3] = [
        GalleryView::StaggeredPortrait,
        GalleryView::StaggeredLandscape,
        GalleryView::StaggeredAll,
    ];

    pub fn is_staggered(self) -> bool {
        Self::STAGGERED.contains(&self)
    }
}

/// A booru the user added to the browser by its address. `login` and
/// `api_key` are only read: an older version of the reference kept the
/// account here, and the browser moves it to the server, which keeps the key
/// sealed (`pages/boorus/accounts.rs`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SavedBooru {
    /// `BooruApi::as_str`. Plain text, so a kind from a newer version
    /// doesn't stop the settings from loading.
    pub api: String,
    pub base: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub login: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub api_key: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    #[serde(deserialize_with = "lenient")]
    pub theme: Theme,
    /// OKLCH hue (0–360) the whole palette is derived from.
    #[serde(deserialize_with = "lenient_hue")]
    pub accent_hue: u16,
    /// The exact colour `accent_hue` was taken from, to show it again in the
    /// picker; `None` for a preset.
    #[serde(deserialize_with = "lenient")]
    pub accent_color: Option<Rgb>,
    /// `None` follows the browser's languages.
    #[serde(deserialize_with = "lenient")]
    pub language: Option<Lang>,
    /// The server signed in to last, to prefill the sign-in screen.
    #[serde(deserialize_with = "lenient")]
    pub last_server: String,
    #[serde(deserialize_with = "lenient")]
    pub gallery_view: GalleryView,
    /// Columns in the gallery; `None` aims for a set tile width.
    #[serde(deserialize_with = "lenient")]
    pub gallery_columns: Option<u8>,
    /// No space between thumbnails and no rounded corners.
    #[serde(deserialize_with = "lenient")]
    pub flush_tiles: bool,
    /// The viewer shows originals even where the server has a smaller copy.
    #[serde(deserialize_with = "lenient")]
    pub originals: bool,
    /// The boorus the user added by their address, in the order of their
    /// chips: the latest last.
    #[serde(deserialize_with = "lenient")]
    pub boorus: Vec<SavedBooru>,
    /// The sites the booru browser searched last, as their keys
    /// (`logic::boorus::site_key`). Empty until one is chosen.
    #[serde(deserialize_with = "lenient")]
    pub booru_sites: Vec<String>,
    /// Whether the booru browser leaves out a file another site already
    /// showed.
    #[serde(deserialize_with = "lenient_yes")]
    pub booru_dedupe: bool,
    #[serde(deserialize_with = "lenient")]
    pub hand: Hand,
    /// The editor keeps the original post; otherwise it goes to the recycle
    /// bin once the edited copy is saved.
    #[serde(deserialize_with = "lenient")]
    pub keep_original: bool,
    #[serde(deserialize_with = "lenient")]
    pub battery_saver: Saver,
    #[serde(deserialize_with = "lenient")]
    pub effects: Effects,
    /// Automatic effects found this device too slow for the full ones. Kept,
    /// so the next visit doesn't stutter until it finds out again.
    #[serde(deserialize_with = "lenient")]
    pub effects_reduced: bool,
    /// What swiping a gallery thumbnail to the left does.
    #[serde(deserialize_with = "lenient")]
    pub swipe_left: SwipeAction,
    /// What swiping a gallery thumbnail to the right does.
    #[serde(deserialize_with = "lenient")]
    pub swipe_right: SwipeAction,
    /// Settings this version doesn't have yet (the reference's, or a newer
    /// version's), written back untouched.
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

/// The reference's default accent, a violet.
pub const DEFAULT_HUE: u16 = 285;

impl Default for Prefs {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            accent_hue: DEFAULT_HUE,
            accent_color: None,
            language: None,
            last_server: String::new(),
            gallery_view: GalleryView::default(),
            gallery_columns: None,
            flush_tiles: false,
            originals: false,
            boorus: Vec::new(),
            booru_sites: Vec::new(),
            booru_dedupe: true,
            hand: Hand::Either,
            keep_original: false,
            battery_saver: Saver::Auto,
            effects: Effects::Auto,
            effects_reduced: false,
            swipe_left: SwipeAction::Off,
            swipe_right: SwipeAction::Off,
            other: Map::new(),
        }
    }
}

/// Where the gallery was for one search. It remembers the tile at the top of
/// the screen and how far down it was, as a scroll position alone means
/// nothing once new posts arrive or the layout changes.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Spot {
    pub q: String,
    pub post: Option<i64>,
    /// The tile's distance from the top of the screen in pixels.
    pub offset: f64,
    /// The scroll position, for when that tile has gone.
    pub y: f64,
    /// How many posts were loaded.
    pub count: u32,
}

const MAX_SPOTS: usize = 30;

/// The searches the gallery has shown this account, the latest first.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GalleryMemory {
    owner: String,
    spots: Vec<Spot>,
}

impl GalleryMemory {
    fn visit(&mut self, q: &str) {
        let spot = self
            .spots
            .iter()
            .position(|s| s.q == q)
            .map(|i| self.spots.remove(i))
            .unwrap_or_else(|| Spot {
                q: q.to_owned(),
                ..Spot::default()
            });
        self.spots.insert(0, spot);
        self.spots.truncate(MAX_SPOTS);
    }

    fn remember(&mut self, spot: Spot) {
        self.visit(&spot.q);
        if let Some(first) = self.spots.first_mut() {
            *first = spot;
        }
    }

    /// This account's memory, or a fresh one for an account it isn't.
    fn for_owner(&mut self, owner: &str) -> &mut Self {
        if self.owner != owner {
            *self = Self {
                owner: owner.to_owned(),
                spots: Vec::new(),
            };
        }
        self
    }

    fn spot(&self, owner: &str, q: &str) -> Option<Spot> {
        (self.owner == owner)
            .then(|| self.spots.iter().find(|s| s.q == q).cloned())
            .flatten()
    }

    fn last_search(&self, owner: &str) -> String {
        (self.owner == owner)
            .then(|| self.spots.first().map(|s| s.q.clone()))
            .flatten()
            .unwrap_or_default()
    }
}

/// Everything under the storage key.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct Stored {
    /// Lenient on its own: a session this version can't read means signing
    /// in again, not losing the preferences with it.
    #[serde(deserialize_with = "lenient")]
    session: Option<Session>,
    #[serde(deserialize_with = "lenient")]
    prefs: Prefs,
    #[serde(deserialize_with = "lenient")]
    memory: GalleryMemory,
    /// What else the reference kept here, written back untouched.
    #[serde(flatten)]
    other: Map<String, Value>,
}

/// A field's value, or its default when it holds something this version
/// can't read, so one odd field doesn't cost the rest.
fn lenient<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + Default,
{
    let value = Value::deserialize(d)?;
    Ok(T::deserialize(value).unwrap_or_default())
}

/// A switch that is on unless it says otherwise.
fn lenient_yes<'de, D>(d: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(d)?;
    Ok(bool::deserialize(value).unwrap_or(true))
}

/// A hue, or the default one when the value is no hue at all.
fn lenient_hue<'de, D>(d: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(d)?;
    Ok(u16::deserialize(value)
        .ok()
        .filter(|h| *h <= 360)
        .unwrap_or(DEFAULT_HUE))
}

/// The stored state, from what the browser kept. Anything unreadable is the
/// first start's state.
fn parse(json: &str) -> Stored {
    serde_json::from_str(json).unwrap_or_default()
}

fn to_json(stored: &Stored) -> Option<String> {
    serde_json::to_string(stored).ok()
}

thread_local! {
    // Reference counted, so no component's owner can dispose of them.
    static SESSION: ArcRwSignal<Option<Session>> = ArcRwSignal::new(None);
    static PREFS: ArcRwSignal<Prefs> = ArcRwSignal::new(Prefs::default());
    static MEMORY: ArcRwSignal<GalleryMemory> = ArcRwSignal::new(GalleryMemory::default());
    static OTHER: RefCell<Map<String, Value>> = RefCell::new(Map::new());
    static SIGN_IN_NOTE: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Reads what the browser kept. Called once, before the app is mounted.
pub fn load() {
    let stored = backend::read().map(|json| parse(&json)).unwrap_or_default();
    SESSION.with(|s| s.set(stored.session));
    PREFS.with(|p| p.set(stored.prefs));
    MEMORY.with(|m| m.set(stored.memory));
    OTHER.with_borrow_mut(|o| *o = stored.other);
}

/// Writes the current state back to the browser.
fn save() {
    let stored = Stored {
        session: SESSION.with(|s| s.get_untracked()),
        prefs: PREFS.with(|p| p.get_untracked()),
        memory: MEMORY.with(|m| m.get_untracked()),
        other: OTHER.with_borrow(Clone::clone),
    };
    if let Some(json) = to_json(&stored) {
        backend::write(&json);
    }
}

/// The signed-in session, if any. Read inside a view, it redraws on sign-in
/// and sign-out.
pub fn session() -> Option<Session> {
    SESSION.with(|s| s.get())
}

/// The session, outside a view.
pub fn session_untracked() -> Option<Session> {
    SESSION.with(|s| s.get_untracked())
}

/// Whether someone is signed in. Read inside a view, it redraws only when
/// that changes.
pub fn signed_in() -> bool {
    SESSION.with(|s| s.with(Option::is_some))
}

/// Keeps the session from a sign-in, and its server for the next one.
pub fn sign_in(server: String, auth: AuthResponse) {
    SIGN_IN_NOTE.with_borrow_mut(|note| *note = None);
    PREFS.with(|p| p.update(|p| p.last_server.clone_from(&server)));
    SESSION.with(|s| {
        s.set(Some(Session {
            server,
            token: auth.token,
            user: auth.user,
        }));
    });
    save();
}

/// The account as the server has it now, after a rename or a new role.
pub fn update_user(user: User) {
    let changed =
        SESSION.with(|s| s.with_untracked(|s| s.as_ref().is_some_and(|s| s.user != user)));
    if changed {
        SESSION.with(|s| {
            s.update(|s| {
                if let Some(s) = s {
                    s.user = user;
                }
            });
        });
        save();
    }
}

/// Signs out here and on the server. The server's answer doesn't matter: the
/// token is forgotten either way.
pub fn sign_out() {
    if let Some(client) = crate::api::client() {
        leptos::task::spawn_local(async move {
            let _ = client.logout().await;
        });
    }
    // Where the gallery was in each search belongs to this account.
    MEMORY.with(|m| m.set(GalleryMemory::default()));
    forget_session();
}

/// Drops the session here only, as after the server said it's no good.
pub fn forget_session() {
    crate::api::forget();
    SESSION.with(|s| s.set(None));
    save();
}

/// Says on the sign-in screen why it's showing, such as a session that
/// expired. Kept until that screen shows it or someone signs in.
pub fn set_sign_in_note(text: String) {
    SIGN_IN_NOTE.with_borrow_mut(|note| *note = Some(text));
}

/// The note for the sign-in screen, once.
pub fn take_sign_in_note() -> Option<String> {
    SIGN_IN_NOTE.with_borrow_mut(Option::take)
}

/// The address the sign-in screen starts with (`logic::address::prefill`).
pub fn default_server() -> String {
    let origin = backend::origin().unwrap_or_default();
    address::prefill(
        &prefs_untracked().last_server,
        option_env!("STARBOORU_SERVER"),
        &origin,
    )
}

/// The preferences. Read inside a view, it redraws when they change.
pub fn prefs() -> Prefs {
    PREFS.with(|p| p.get())
}

/// The preferences, outside a view: reading them here redraws nothing.
pub fn prefs_untracked() -> Prefs {
    PREFS.with(|p| p.get_untracked())
}

/// Changes the preferences and saves them.
pub fn update_prefs(change: impl FnOnce(&mut Prefs)) {
    PREFS.with(|p| p.update(change));
    save();
}

/// Who the app is signed in as, so one account never sees what another
/// searched: `server|user id`, or empty when no one is.
pub fn owner() -> String {
    session_untracked().map_or_else(String::new, |s| format!("{}|{}", s.server, s.user.id))
}

fn update_memory(change: impl FnOnce(&mut GalleryMemory)) {
    let owner = owner();
    MEMORY.with(|m| m.update(|m| change(m.for_owner(&owner))));
    save();
}

/// Makes `q` the search the gallery goes back to.
pub fn visit_search(q: &str) {
    update_memory(|memory| memory.visit(q));
}

/// Where the gallery is in its search, as the screen stops moving.
pub fn remember_spot(spot: Spot) {
    update_memory(|memory| memory.remember(spot));
}

/// Where the gallery was for `q`.
pub fn spot(q: &str) -> Option<Spot> {
    let owner = owner();
    MEMORY.with(|m| m.with_untracked(|m| m.spot(&owner, q)))
}

/// The search the gallery showed last. Read inside a view, it redraws when
/// the gallery remembers a spot, so read it through a `Memo`.
pub fn last_search() -> String {
    let owner = owner();
    MEMORY.with(|m| m.with(|m| m.last_search(&owner)))
}

/// The search the gallery showed last, outside a view.
pub fn last_search_untracked() -> String {
    let owner = owner();
    MEMORY.with(|m| m.with_untracked(|m| m.last_search(&owner)))
}

#[cfg(target_arch = "wasm32")]
mod backend {
    const KEY: &str = "starbooru";

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    pub fn read() -> Option<String> {
        storage()?.get_item(KEY).ok()?
    }

    /// Best effort: a browser that keeps nothing (private mode, full storage)
    /// still runs, with the settings for this visit only.
    pub fn write(json: &str) {
        if let Some(storage) = storage() {
            let _ = storage.set_item(KEY, json);
        }
    }

    /// The page's own address, `https://host[:port]`.
    pub fn origin() -> Option<String> {
        web_sys::window()?.location().origin().ok()
    }
}

/// The host, where the tests run, has no browser storage.
#[cfg(not(target_arch = "wasm32"))]
mod backend {
    pub fn read() -> Option<String> {
        None
    }

    pub fn write(_json: &str) {}

    pub fn origin() -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user() -> User {
        serde_json::from_value(serde_json::json!({
            "id": 7, "username": "ada", "role": "admin",
            "created_at": "2026-01-02T03:04:05Z",
        }))
        .unwrap()
    }

    #[test]
    fn state_round_trips() {
        let stored = Stored {
            session: Some(Session {
                server: "https://booru.example".into(),
                token: "t0k3n".into(),
                user: user(),
            }),
            prefs: Prefs {
                theme: Theme::Dark,
                accent_hue: 120,
                accent_color: Some([1, 2, 3]),
                language: Some(Lang::Italian),
                last_server: "https://booru.example".into(),
                gallery_view: GalleryView::StaggeredAll,
                gallery_columns: Some(4),
                flush_tiles: true,
                originals: true,
                boorus: vec![SavedBooru {
                    api: "danbooru".into(),
                    base: "http://nas:8000".into(),
                    ..SavedBooru::default()
                }],
                booru_sites: vec!["danbooru".into(), "e926".into()],
                booru_dedupe: false,
                hand: Hand::Left,
                keep_original: true,
                battery_saver: Saver::Always,
                effects: Effects::Lite,
                effects_reduced: true,
                swipe_left: SwipeAction::Delete,
                swipe_right: SwipeAction::Like,
                other: Map::new(),
            },
            memory: GalleryMemory {
                owner: "https://booru.example|7".into(),
                spots: vec![Spot {
                    q: "cat".into(),
                    post: Some(12),
                    offset: 80.5,
                    y: 2400.0,
                    count: 120,
                }],
            },
            other: Map::new(),
        };
        let json = to_json(&stored).unwrap();
        assert_eq!(parse(&json), stored);
        // The shape the reference wrote, and index.html's loading screen reads.
        let value: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["prefs"]["theme"], "dark");
        assert_eq!(value["prefs"]["accent_hue"], 120);
        assert_eq!(value["prefs"]["language"], "it");
        assert_eq!(value["session"]["user"]["username"], "ada");
        assert_eq!(value["prefs"]["gallery_view"], "staggered-all");
        // The reference's name, so its setting carries over.
        assert_eq!(value["prefs"]["originals"], true);
        assert_eq!(value["memory"]["spots"][0]["post"], 12);
        // The reference's names, and no empty account written back.
        assert_eq!(value["prefs"]["booru_sites"][1], "e926");
        assert_eq!(value["prefs"]["booru_dedupe"], false);
        assert_eq!(value["prefs"]["hand"], "left");
        assert_eq!(value["prefs"]["keep_original"], true);
        // The reference's names for the looks and the gestures.
        assert_eq!(value["prefs"]["battery_saver"], "always");
        assert_eq!(value["prefs"]["effects"], "lite");
        assert_eq!(value["prefs"]["effects_reduced"], true);
        assert_eq!(value["prefs"]["swipe_left"], "delete");
        assert_eq!(value["prefs"]["swipe_right"], "like");
        assert_eq!(
            value["prefs"]["boorus"][0],
            serde_json::json!({"api": "danbooru", "base": "http://nas:8000"})
        );
    }

    #[test]
    fn the_booru_browser_starts_from_prefs_without_its_fields() {
        let prefs = parse(r#"{"prefs":{"theme":"dark"}}"#).prefs;
        assert!(prefs.boorus.is_empty());
        assert!(prefs.booru_sites.is_empty());
        assert!(prefs.booru_dedupe, "duplicates are hidden unless asked for");
        // As an older reference wrote them, with the account still on the device.
        let prefs = parse(
            r#"{"prefs":{"boorus":[{"api":"e621","base":"https://e926.net","login":"me","api_key":"k"}],
                "booru_sites":["danbooru","danbooru|http://nas:8000"],"booru_dedupe":false}}"#,
        )
        .prefs;
        assert_eq!(prefs.boorus.len(), 1);
        assert_eq!(prefs.boorus[0].login, "me");
        assert_eq!(prefs.booru_sites, ["danbooru", "danbooru|http://nas:8000"]);
        assert!(!prefs.booru_dedupe);
        // Odd values cost only themselves.
        let prefs =
            parse(r#"{"prefs":{"boorus":"x","booru_sites":[1],"booru_dedupe":"no"}}"#).prefs;
        assert!(prefs.boorus.is_empty());
        assert!(prefs.booru_sites.is_empty());
        assert!(prefs.booru_dedupe);
    }

    #[test]
    fn older_and_partial_state_still_loads() {
        let stored = parse(r#"{"prefs":{"theme":"dark","accent_hue":120}}"#);
        assert_eq!(stored.session, None);
        assert_eq!(stored.prefs.theme, Theme::Dark);
        assert_eq!(stored.prefs.accent_hue, 120);
        assert_eq!(stored.prefs.accent_color, None);
        assert_eq!(stored.prefs.language, None);
        assert_eq!(parse("{}"), Stored::default());
        assert_eq!(parse(r#"{"prefs":{}}"#).prefs, Prefs::default());
        // Not JSON at all: the first start's state.
        assert_eq!(parse("not json"), Stored::default());
        assert_eq!(parse(""), Stored::default());
    }

    #[test]
    fn an_unknown_theme_falls_back_to_the_system() {
        let stored = parse(r#"{"prefs":{"theme":"sepia","accent_hue":40}}"#);
        assert_eq!(stored.prefs.theme, Theme::System);
        assert_eq!(stored.prefs.accent_hue, 40);
        for theme in [Theme::System, Theme::Light, Theme::Dark] {
            let json = format!(r#"{{"prefs":{{"theme":"{}"}}}}"#, theme.as_str());
            assert_eq!(parse(&json).prefs.theme, theme);
        }
    }

    #[test]
    fn an_odd_field_costs_only_itself() {
        let stored = parse(
            r#"{"session":{"server":"s","token":"t"},
                "prefs":{"theme":"light","language":"de","accent_hue":"blue",
                         "accent_color":"red","last_server":7}}"#,
        );
        // A session missing its user can't be used: sign in again.
        assert_eq!(stored.session, None);
        assert_eq!(stored.prefs.theme, Theme::Light);
        assert_eq!(stored.prefs.language, None);
        assert_eq!(stored.prefs.accent_hue, DEFAULT_HUE);
        assert_eq!(stored.prefs.accent_color, None);
        assert_eq!(stored.prefs.last_server, "");
        // A hue past the circle is no hue.
        assert_eq!(
            parse(r#"{"prefs":{"accent_hue":400}}"#).prefs.accent_hue,
            DEFAULT_HUE
        );
        // Prefs that aren't an object.
        assert_eq!(parse(r#"{"prefs":[1,2]}"#).prefs, Prefs::default());
    }

    #[test]
    fn signing_in_keeps_the_session_and_its_server() {
        set_sign_in_note("expired".into());
        sign_in(
            "https://booru.example".into(),
            AuthResponse {
                token: "t0k3n".into(),
                user: user(),
            },
        );
        assert!(signed_in());
        let session = session_untracked().unwrap();
        assert_eq!(session.server, "https://booru.example");
        assert_eq!(session.user.username, "ada");
        assert_eq!(prefs_untracked().last_server, "https://booru.example");
        // A note from before is no longer true.
        assert_eq!(take_sign_in_note(), None);

        let mut renamed = user();
        renamed.username = "ada2".into();
        update_user(renamed);
        assert_eq!(session_untracked().unwrap().user.username, "ada2");

        forget_session();
        assert!(!signed_in());
        // The next sign-in starts from the same server.
        assert_eq!(prefs_untracked().last_server, "https://booru.example");
    }

    #[test]
    fn the_sign_in_note_shows_once() {
        assert_eq!(take_sign_in_note(), None);
        set_sign_in_note("Your session has expired.".into());
        assert_eq!(
            take_sign_in_note().as_deref(),
            Some("Your session has expired.")
        );
        assert_eq!(take_sign_in_note(), None);
    }

    #[test]
    fn signing_out_forgets_where_the_gallery_was() {
        OTHER.with_borrow_mut(|o| {
            o.insert("kept".into(), serde_json::json!(1));
        });
        visit_search("cat");
        assert_eq!(last_search_untracked(), "cat");
        sign_out();
        assert_eq!(MEMORY.with(|m| m.get_untracked()), GalleryMemory::default());
        OTHER.with_borrow(|o| assert!(o.contains_key("kept")));
    }

    #[test]
    fn the_latest_search_comes_first_and_keeps_its_spot() {
        let mut memory = GalleryMemory::default();
        memory.remember(Spot {
            q: "cat".into(),
            post: Some(7),
            offset: 12.0,
            y: 900.0,
            count: 120,
        });
        memory.visit("dog");
        assert_eq!(memory.last_search(""), "dog");
        assert_eq!(memory.spot("", "dog").and_then(|s| s.post), None);
        memory.visit("cat");
        assert_eq!(memory.last_search(""), "cat");
        assert_eq!(memory.spot("", "cat").and_then(|s| s.post), Some(7));
        assert_eq!(memory.spots.len(), 2);
    }

    #[test]
    fn old_searches_drop_off() {
        let mut memory = GalleryMemory::default();
        for i in 0..MAX_SPOTS + 5 {
            memory.visit(&format!("tag{i}"));
        }
        assert_eq!(memory.spots.len(), MAX_SPOTS);
        assert!(memory.spots.iter().all(|s| s.q != "tag0"));
    }

    #[test]
    fn one_account_never_sees_where_another_was() {
        let mut memory = GalleryMemory::default();
        memory.for_owner("a|1").visit("cat");
        assert_eq!(memory.last_search("a|1"), "cat");
        assert_eq!(memory.last_search("a|2"), "");
        assert_eq!(memory.spot("a|2", "cat"), None);
        // Another account starts again.
        memory.for_owner("a|2").visit("dog");
        assert_eq!(memory.last_search("a|1"), "");
        assert_eq!(memory.spots.len(), 1);
    }

    #[test]
    fn gallery_prefs_default_and_round_trip() {
        let prefs = parse(r#"{"prefs":{"theme":"dark"}}"#).prefs;
        assert_eq!(prefs.gallery_view, GalleryView::StaggeredPortrait);
        assert_eq!(prefs.gallery_columns, None);
        assert!(!prefs.flush_tiles);
        for view in GalleryView::FIXED.into_iter().chain(GalleryView::STAGGERED) {
            let json = serde_json::to_string(&view).unwrap();
            let stored = parse(&format!(r#"{{"prefs":{{"gallery_view":{json}}}}}"#));
            assert_eq!(stored.prefs.gallery_view, view);
        }
        // An unknown view from a newer version falls back rather than failing,
        // and an odd column count costs only itself.
        let prefs = parse(
            r#"{"prefs":{"gallery_view":"hexagons","gallery_columns":4,"flush_tiles":true}}"#,
        )
        .prefs;
        assert_eq!(prefs.gallery_view, GalleryView::StaggeredPortrait);
        assert_eq!(prefs.gallery_columns, Some(4));
        assert!(prefs.flush_tiles);
        let prefs = parse(r#"{"prefs":{"gallery_view":"square","gallery_columns":900}}"#).prefs;
        assert_eq!(prefs.gallery_view, GalleryView::Square);
        assert_eq!(prefs.gallery_columns, None);
    }

    #[test]
    fn the_hand_and_the_editors_original_default_and_fall_back() {
        let prefs = parse(r#"{"prefs":{"theme":"dark"}}"#).prefs;
        assert_eq!(prefs.hand, Hand::Either);
        assert!(
            !prefs.keep_original,
            "the original goes to the bin unless kept"
        );
        let prefs = parse(r#"{"prefs":{"hand":"right","keep_original":true}}"#).prefs;
        assert_eq!(prefs.hand, Hand::Right);
        assert!(prefs.keep_original);
        // An unknown hand from a newer version, or odd values, cost only themselves.
        let prefs =
            parse(r#"{"prefs":{"hand":"both","keep_original":"yes","theme":"light"}}"#).prefs;
        assert_eq!(prefs.hand, Hand::Either);
        assert!(!prefs.keep_original);
        assert_eq!(prefs.theme, Theme::Light);
        let prefs = parse(r#"{"prefs":{"hand":7}}"#).prefs;
        assert_eq!(prefs.hand, Hand::Either);
        for hand in [Hand::Either, Hand::Left, Hand::Right] {
            let json = format!(r#"{{"prefs":{{"hand":"{}"}}}}"#, hand.as_str());
            assert_eq!(parse(&json).prefs.hand, hand);
        }
    }

    #[test]
    fn looks_and_gestures_default_and_fall_back() {
        let prefs = parse(r#"{"prefs":{"theme":"dark"}}"#).prefs;
        assert_eq!(prefs.battery_saver, Saver::Auto);
        assert_eq!(
            (prefs.effects, prefs.effects_reduced),
            (Effects::Auto, false)
        );
        // Swiping a thumbnail does nothing until chosen.
        assert_eq!(
            (prefs.swipe_left, prefs.swipe_right),
            (SwipeAction::Off, SwipeAction::Off)
        );
        for effects in Effects::ALL {
            let json = format!(
                r#"{{"prefs":{{"effects":{},"effects_reduced":true}}}}"#,
                serde_json::to_string(&effects).unwrap()
            );
            let prefs = parse(&json).prefs;
            assert_eq!((prefs.effects, prefs.effects_reduced), (effects, true));
        }
        for saver in Saver::ALL {
            let json = format!(
                r#"{{"prefs":{{"battery_saver":{}}}}}"#,
                serde_json::to_string(&saver).unwrap()
            );
            assert_eq!(parse(&json).prefs.battery_saver, saver);
        }
        for action in SwipeAction::ALL {
            assert_eq!(SwipeAction::parse(action.as_str()), Some(action));
            let json = format!(r#"{{"prefs":{{"swipe_left":"{}"}}}}"#, action.as_str());
            assert_eq!(parse(&json).prefs.swipe_left, action);
        }
        assert_eq!(SwipeAction::parse("nope"), None);
        // From a newer version, or odd: the default, and the rest still reads.
        let prefs = parse(
            r#"{"prefs":{"effects":"cinematic","battery_saver":3,"swipe_left":"archive",
                "swipe_right":["x"],"effects_reduced":"yes","theme":"light"}}"#,
        )
        .prefs;
        assert_eq!(prefs.effects, Effects::Auto);
        assert_eq!(prefs.battery_saver, Saver::Auto);
        assert_eq!(prefs.swipe_left, SwipeAction::Off);
        assert_eq!(prefs.swipe_right, SwipeAction::Off);
        assert!(!prefs.effects_reduced);
        assert_eq!(prefs.theme, Theme::Light);
    }

    #[test]
    fn state_without_memory_still_loads() {
        assert_eq!(
            parse(r#"{"prefs":{"theme":"dark"}}"#).memory,
            GalleryMemory::default()
        );
        assert_eq!(
            parse(r#"{"memory":"odd"}"#).memory,
            GalleryMemory::default()
        );
    }

    #[test]
    fn what_this_version_doesnt_know_is_written_back() {
        // As the reference wrote it, with settings and the gallery's places
        // the web app doesn't have yet.
        let json = r#"{"session":null,
            "prefs":{"theme":"dark","accent_hue":200,"hand":"left","cache_mb":500,
                     "system_accent":false,"effects":"lite",
                     "boorus":[{"api":"e621","base":"https://e926.net"}]},
            "memory":{"owner":"x|1","spots":[{"q":"cat"}]}}"#;
        let stored = parse(json);
        assert_eq!(stored.prefs.theme, Theme::Dark);
        assert_eq!(stored.prefs.hand, Hand::Left);
        assert_eq!(stored.prefs.effects, Effects::Lite);
        assert_eq!(stored.prefs.other["cache_mb"], 500);
        let again: Value = serde_json::from_str(&to_json(&stored).unwrap()).unwrap();
        let before: Value = serde_json::from_str(json).unwrap();
        assert_eq!(again["prefs"]["hand"], before["prefs"]["hand"]);
        assert_eq!(again["prefs"]["effects"], before["prefs"]["effects"]);
        // The desktop's cache and Android's wallpaper colour, kept for them.
        assert_eq!(again["prefs"]["cache_mb"], before["prefs"]["cache_mb"]);
        assert_eq!(
            again["prefs"]["system_accent"],
            before["prefs"]["system_accent"]
        );
        assert_eq!(again["prefs"]["boorus"], before["prefs"]["boorus"]);
        // The gallery's places, in the reference's shape, read as they were.
        assert_eq!(stored.memory.last_search("x|1"), "cat");
        assert_eq!(again["memory"]["owner"], before["memory"]["owner"]);
        assert_eq!(again["memory"]["spots"][0]["q"], "cat");
    }
}
