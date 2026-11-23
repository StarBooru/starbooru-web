//! What the app keeps in the browser: the session and the preferences, as JSON
//! in `localStorage["starbooru"]`. The shape is the reference app's, so a
//! browser that used it keeps its settings:
//!
//! ```json
//! { "session": { "server": "…", "token": "…", "user": { … } } | null,
//!   "prefs": { "theme": "system", "accent_hue": 285, … } }
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
    /// The value of `data-theme` on `.app`, which `style/tokens.css` reads.
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::System => "system",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
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
            other: Map::new(),
        }
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
    /// What else the reference kept here (the gallery's places), written back
    /// untouched.
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
    static OTHER: RefCell<Map<String, Value>> = RefCell::new(Map::new());
    static SIGN_IN_NOTE: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Reads what the browser kept. Called once, before the app is mounted.
pub fn load() {
    let stored = backend::read().map(|json| parse(&json)).unwrap_or_default();
    SESSION.with(|s| s.set(stored.session));
    PREFS.with(|p| p.set(stored.prefs));
    OTHER.with_borrow_mut(|o| *o = stored.other);
}

/// Writes the current state back to the browser.
fn save() {
    let stored = Stored {
        session: SESSION.with(|s| s.get_untracked()),
        prefs: PREFS.with(|p| p.get_untracked()),
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
    OTHER.with_borrow_mut(|o| {
        o.remove("memory");
    });
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
#[allow(dead_code, reason = "the Settings page changes them")]
pub fn update_prefs(change: impl FnOnce(&mut Prefs)) {
    PREFS.with(|p| p.update(change));
    save();
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
                other: Map::new(),
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
            o.insert("memory".into(), serde_json::json!({"owner": "x|1"}));
            o.insert("kept".into(), serde_json::json!(1));
        });
        sign_out();
        OTHER.with_borrow(|o| {
            assert!(!o.contains_key("memory"));
            assert!(o.contains_key("kept"));
        });
    }

    #[test]
    fn what_this_version_doesnt_know_is_written_back() {
        // As the reference wrote it, with settings and the gallery's places
        // the web app doesn't have yet.
        let json = r#"{"session":null,
            "prefs":{"theme":"dark","accent_hue":200,"hand":"left","effects":"lite",
                     "boorus":[{"api":"e621","base":"https://e926.net"}]},
            "memory":{"owner":"x|1","spots":[{"q":"cat"}]}}"#;
        let stored = parse(json);
        assert_eq!(stored.prefs.theme, Theme::Dark);
        assert_eq!(stored.prefs.other["hand"], "left");
        let again: Value = serde_json::from_str(&to_json(&stored).unwrap()).unwrap();
        let before: Value = serde_json::from_str(json).unwrap();
        assert_eq!(again["prefs"]["hand"], before["prefs"]["hand"]);
        assert_eq!(again["prefs"]["effects"], before["prefs"]["effects"]);
        assert_eq!(again["prefs"]["boorus"], before["prefs"]["boorus"]);
        assert_eq!(again["memory"], before["memory"]);
    }
}
