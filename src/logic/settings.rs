//! Settings without a browser: the accent's presets and what a picked colour
//! comes to, what the lists of devices, sign-ins and API keys say of each,
//! when the password form can be sent, which cards an account sees, and the
//! app's version.

use chrono::{DateTime, Local, Utc};
use starbooru_core::color::Rgb;
use starbooru_core::keys::{ApiKeyError, MAX_API_KEY_DAYS, MAX_API_KEY_NAME_CHARS, clean_api_key};
use starbooru_core::{
    ApiKey, Identity, NewApiKey, Role, ServerInfo, SessionInfo, SignInMethods, Usage, User,
};

use super::library::ago;
use super::upload::human_size;
use crate::store::Prefs;
use crate::theme;

// ---------- the accent ----------

/// The accent's presets, as OKLCH hues, in the order they're offered.
pub const SWATCHES: [u16; 8] = [285, 330, 20, 65, 140, 190, 245, 0];

/// Whether the preset of `hue` is the accent in use.
pub fn picked(prefs: &Prefs, hue: u16) -> bool {
    prefs.accent_color.is_none() && prefs.accent_hue == hue
}

/// Whether the accent in use came from the picker rather than a preset. A
/// hue with no colour kept came from the slider older versions had.
pub fn custom_in_use(prefs: &Prefs) -> bool {
    prefs.accent_color.is_some() || !SWATCHES.contains(&prefs.accent_hue)
}

/// Makes the preset of `hue` the accent.
pub fn pick(prefs: &mut Prefs, hue: u16) {
    prefs.accent_hue = hue;
    prefs.accent_color = None;
}

/// Makes `rgb` the accent, keeping the colour to show in the picker again.
/// The palette is built from a hue, so a grey can't be one: that leaves the
/// accent as it was and answers false.
pub fn pick_color(prefs: &mut Prefs, rgb: Rgb) -> bool {
    let Some(hue) = theme::hue_of(rgb) else {
        return false;
    };
    prefs.accent_hue = hue;
    prefs.accent_color = Some(rgb);
    true
}

/// The colour the picker opens on: the one picked, else the preset's.
pub fn accent_rgb(prefs: &Prefs) -> Rgb {
    prefs
        .accent_color
        .unwrap_or_else(|| theme::swatch_rgb(prefs.accent_hue))
}

/// A preset's name, for a screen reader and the tooltip.
pub fn swatch_name(hue: u16) -> String {
    match hue {
        285 => t!("accent-violet"),
        330 => t!("accent-pink"),
        20 => t!("accent-coral"),
        65 => t!("accent-amber"),
        140 => t!("accent-green"),
        190 => t!("accent-teal"),
        245 => t!("accent-blue"),
        _ => t!("accent-red"),
    }
}

// ---------- the account ----------

pub fn role_label(role: Role) -> String {
    match role {
        Role::Admin => t!("role-admin"),
        Role::User => t!("role-user"),
    }
}

/// "12 posts · 1.2 GB of 5.0 GB", or without the limit when there is none.
pub fn usage_text(u: &Usage) -> String {
    let used = match u.quota_bytes {
        Some(quota) => t!(
            "usage-of",
            used = human_size(u.bytes),
            quota = human_size(quota)
        ),
        None => human_size(u.bytes),
    };
    format!("{} · {used}", t!("posts-count", count = u.posts))
}

/// The cards Settings shows besides the ones everyone has.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sections {
    /// The import of a whole library, unless the server never reaches out.
    pub library_import: bool,
    /// The way to the administration, for administrators.
    pub admin: bool,
    /// The ways to sign in, where there's a provider to link or an identity
    /// already linked.
    pub sign_in: bool,
}

pub fn sections(
    user: &User,
    info: Option<&ServerInfo>,
    methods: Option<&SignInMethods>,
) -> Sections {
    let provider = info.is_some_and(|i| i.oidc.is_some());
    Sections {
        library_import: info.is_some_and(|i| i.url_import),
        admin: user.role == Role::Admin,
        sign_in: provider || methods.is_some_and(|m| !m.identities.is_empty()),
    }
}

// ---------- ways to sign in ----------

/// "alice@example.com", or the provider's name when it gave no label.
pub fn identity_name(identity: &Identity, provider: Option<&str>) -> String {
    match (identity.label.as_deref(), provider) {
        (Some(label), _) if !label.trim().is_empty() => label.to_owned(),
        (_, Some(provider)) => provider.to_owned(),
        _ => t!("signin-provider"),
    }
}

/// "Linked 3 days ago · last used just now".
pub fn identity_times(identity: &Identity, now: DateTime<Utc>) -> String {
    let linked = ago(identity.created_at, now);
    match identity.last_used_at {
        Some(used) => t!("signin-times", linked = linked, used = ago(used, now)),
        None => t!("signin-linked", linked = linked),
    }
}

/// The one way left in, which can't be taken away: no password and a
/// single identity.
pub fn last_way_in(m: &SignInMethods) -> bool {
    !m.password && m.identities.len() == 1
}

// ---------- the password ----------

/// The server's shortest password.
pub const MIN_PASSWORD_CHARS: usize = 8;

/// What the form says under the new password and its repeat.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PasswordCheck {
    pub too_short: bool,
    pub mismatch: bool,
    /// Everything the server needs is there.
    pub ready: bool,
}

/// `has_password` is false for an account made by a provider sign-in, which
/// sets its first password without a current one.
pub fn check_password(has_password: bool, current: &str, new: &str, repeat: &str) -> PasswordCheck {
    let long_enough = new.chars().count() >= MIN_PASSWORD_CHARS;
    PasswordCheck {
        too_short: !new.is_empty() && !long_enough,
        mismatch: !repeat.is_empty() && repeat != new,
        ready: (!has_password || !current.is_empty()) && long_enough && repeat == new,
    }
}

/// Whether the server refused a new password because the current one was
/// wrong. Only an account with a password is asked for it; any other 400
/// (a password the server finds too short, say) keeps the server's words.
pub fn wrong_current(
    has_password: bool,
    status: Option<u16>,
    code: Option<&str>,
    message: &str,
) -> bool {
    let about_current = code.is_some_and(|c| c.contains("current"))
        || message.to_lowercase().contains("current password");
    has_password && status == Some(400) && about_current
}

// ---------- signed-in devices ----------

/// A device's name, as the server worked it out from its browser.
pub fn session_name(s: &SessionInfo) -> String {
    let client = s.client.trim();
    if client.is_empty() {
        t!("sessions-unknown")
    } else {
        client.to_owned()
    }
}

/// "Active 3 minutes ago · signed in yesterday".
pub fn session_times(s: &SessionInfo, now: DateTime<Utc>) -> String {
    t!(
        "sessions-times",
        active = ago(s.last_used_at, now),
        created = ago(s.created_at, now)
    )
}

/// Whether there's another device to sign out.
pub fn others(list: &[SessionInfo]) -> bool {
    list.iter().any(|s| !s.current)
}

// ---------- API keys ----------

/// Lifetimes offered for a new key, in days; `None` never expires.
pub const KEY_LIFETIMES: [Option<u32>; 4] = [None, Some(30), Some(90), Some(365)];

/// A lifetime as the drop-down holds it.
pub fn key_lifetime_value(days: Option<u32>) -> String {
    days.map(|d| d.to_string()).unwrap_or_default()
}

/// The drop-down's value as the request's `expires_days`. Anything the
/// server wouldn't take means never.
pub fn key_lifetime_days(value: &str) -> Option<u32> {
    value
        .trim()
        .parse()
        .ok()
        .filter(|d| (1..=MAX_API_KEY_DAYS).contains(d))
}

/// "Never", "30 days".
pub fn key_lifetime_label(days: Option<u32>) -> String {
    match days {
        None => t!("keys-never-expires"),
        Some(d) => t!("keys-days", count = d),
    }
}

/// The request for a new key, checked as the server checks it.
pub fn new_key(name: &str, read_only: bool, days: Option<u32>) -> Result<NewApiKey, ApiKeyError> {
    Ok(NewApiKey {
        name: clean_api_key(name, days)?,
        read_only,
        expires_days: days,
    })
}

/// Why a key can't be made, in words.
pub fn key_error_text(e: &ApiKeyError) -> String {
    match e {
        ApiKeyError::EmptyName => t!("keys-error-name"),
        ApiKeyError::NameTooLong => t!("keys-error-long", max = MAX_API_KEY_NAME_CHARS),
        ApiKeyError::ControlChar => t!("keys-error-control"),
        ApiKeyError::BadExpiry => t!("keys-error-days", max = MAX_API_KEY_DAYS),
    }
}

/// Whether the key has stopped working by its date.
pub fn key_expired(key: &ApiKey, now: DateTime<Utc>) -> bool {
    key.expires_at.is_some_and(|t| t <= now)
}

/// "Used 3 minutes ago · Expires 2027-11-23", for one key.
pub fn key_status(key: &ApiKey, now: DateTime<Utc>) -> String {
    let date = |t: DateTime<Utc>| t.with_timezone(&Local).format("%Y-%m-%d").to_string();
    let used = match key.last_used_at {
        Some(t) => t!("keys-used", when = ago(t, now)),
        None => t!("keys-never-used"),
    };
    let expiry = match key.expires_at {
        None => t!("keys-no-expiry"),
        Some(t) if t <= now => t!("keys-expired-on", date = date(t)),
        Some(t) => t!("keys-expires-on", date = date(t)),
    };
    format!("{used} · {expiry}")
}

// ---------- about ----------

/// This build's version: the release tag it was built from (`v0.2.0` reads
/// `0.2.0`), else the crate's.
pub fn app_version(tag: Option<&'static str>) -> &'static str {
    match tag.map(str::trim).filter(|t| !t.is_empty()) {
        Some(tag) => tag.strip_prefix('v').unwrap_or(tag),
        None => env!("CARGO_PKG_VERSION"),
    }
}

/// The version shown in Settings.
pub fn version() -> &'static str {
    app_version(option_env!("STARBOORU_WEB_VERSION"))
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::from_timestamp(1_800_000_000, 0).unwrap()
    }

    fn user(role: &str) -> User {
        serde_json::from_value(serde_json::json!({
            "id": 7, "username": "ada", "role": role,
            "created_at": "2026-01-02T03:04:05Z",
        }))
        .unwrap()
    }

    fn info(url_import: bool, oidc: Option<&str>) -> ServerInfo {
        ServerInfo {
            name: "StarBooru".into(),
            version: "0.1.0".into(),
            has_users: true,
            registration_open: false,
            max_upload_bytes: 1,
            url_import,
            oidc: oidc.map(str::to_owned),
            video_editing: false,
        }
    }

    fn identity(label: Option<&str>, used: Option<DateTime<Utc>>) -> Identity {
        Identity {
            id: 3,
            label: label.map(str::to_owned),
            created_at: now() - TimeDelta::days(3),
            last_used_at: used,
        }
    }

    #[test]
    fn the_accent_keeps_the_exact_colour_it_came_from() {
        let mut prefs = Prefs::default();
        assert!(pick_color(&mut prefs, [0xff, 0x00, 0x00]));
        assert_eq!(prefs.accent_hue, 29);
        assert_eq!(prefs.accent_color, Some([0xff, 0x00, 0x00]));
        assert_eq!(accent_rgb(&prefs), [0xff, 0x00, 0x00]);
        // A preset forgets it.
        pick(&mut prefs, 140);
        assert_eq!((prefs.accent_hue, prefs.accent_color), (140, None));
        assert_eq!(accent_rgb(&prefs), theme::swatch_rgb(140));
    }

    #[test]
    fn a_grey_cant_be_the_accent() {
        let mut prefs = Prefs::default();
        for grey in [[0x80, 0x80, 0x80], [0, 0, 0], [0xff, 0xff, 0xff]] {
            assert!(!pick_color(&mut prefs, grey));
        }
        assert_eq!(prefs, Prefs::default());
    }

    #[test]
    fn one_accent_choice_shows_as_in_use() {
        let mut prefs = Prefs {
            accent_hue: 140,
            ..Prefs::default()
        };
        assert!(picked(&prefs, 140));
        assert!(!SWATCHES.iter().any(|h| *h != 140 && picked(&prefs, *h)));
        assert!(!custom_in_use(&prefs));
        // An exact colour, even of a preset's hue, belongs to the picker.
        prefs.accent_color = Some([0x20, 0xb0, 0x20]);
        assert!(!picked(&prefs, 140));
        assert!(custom_in_use(&prefs));
        // A hue from the slider of older versions.
        prefs.accent_color = None;
        prefs.accent_hue = 100;
        assert!(custom_in_use(&prefs));
        assert!(!SWATCHES.iter().any(|h| picked(&prefs, *h)));
        // Every preset has a name of its own.
        let names: std::collections::BTreeSet<String> =
            SWATCHES.into_iter().map(swatch_name).collect();
        assert_eq!(names.len(), SWATCHES.len());
    }

    #[test]
    fn an_admin_sees_more_than_a_user() {
        let plain = info(false, None);
        let user_view = sections(&user("user"), Some(&plain), None);
        assert_eq!(user_view, Sections::default());
        let admin_view = sections(&user("admin"), Some(&plain), None);
        assert!(admin_view.admin);
        assert!(!admin_view.library_import && !admin_view.sign_in);
        // The import, where the server reaches other sites.
        assert!(sections(&user("user"), Some(&info(true, None)), None).library_import);
        // Ways to sign in, with a provider to link, or one linked already.
        assert!(sections(&user("user"), Some(&info(false, Some("Authentik"))), None).sign_in);
        let linked = SignInMethods {
            password: true,
            identities: vec![identity(Some("ada@example.com"), None)],
        };
        assert!(sections(&user("user"), None, Some(&linked)).sign_in);
        // Before the server answers, nothing that depends on it.
        let waiting = sections(&user("admin"), None, None);
        assert!(waiting.admin && !waiting.library_import && !waiting.sign_in);
    }

    #[test]
    fn the_account_card_says_how_much_is_used() {
        let mut usage = Usage {
            posts: 12,
            bytes: 1536,
            quota_bytes: None,
        };
        assert_eq!(usage_text(&usage), "12 posts · 1.5 KB");
        usage.quota_bytes = Some(5 * 1024 * 1024 * 1024);
        assert_eq!(usage_text(&usage), "12 posts · 1.5 KB of 5.0 GB");
        assert_eq!(role_label(Role::Admin), "admin");
    }

    #[test]
    fn identities_and_devices_say_when() {
        let now = now();
        let linked = identity(Some("ada@example.com"), None);
        assert_eq!(identity_name(&linked, Some("Authentik")), "ada@example.com");
        assert_eq!(identity_times(&linked, now), "Linked 3 days ago");
        let used = identity(Some(" "), Some(now - TimeDelta::seconds(20)));
        assert_eq!(identity_name(&used, Some("Authentik")), "Authentik");
        assert_eq!(identity_name(&used, None), "Single sign-on");
        assert_eq!(
            identity_times(&used, now),
            "Linked 3 days ago · last used just now"
        );
        let mut methods = SignInMethods {
            password: false,
            identities: vec![linked],
        };
        assert!(last_way_in(&methods));
        methods.password = true;
        assert!(!last_way_in(&methods));

        let device = |id, current, client: &str| SessionInfo {
            id,
            client: client.into(),
            created_at: now - TimeDelta::days(1) - TimeDelta::hours(2),
            last_used_at: now - TimeDelta::minutes(3),
            expires_at: now + TimeDelta::days(29),
            current,
        };
        let here = device(1, true, "Firefox on Linux");
        assert_eq!(session_name(&here), "Firefox on Linux");
        assert_eq!(
            session_times(&here, now),
            "Active 3 minutes ago · signed in yesterday"
        );
        assert_eq!(session_name(&device(2, false, "  ")), "Unknown client");
        assert!(!others(std::slice::from_ref(&here)));
        assert!(others(&[here, device(2, false, "curl")]));
    }

    #[test]
    fn the_password_form_waits_for_what_the_server_needs() {
        let check = check_password(true, "", "", "");
        assert_eq!(check, PasswordCheck::default());
        let short = check_password(true, "old", "abc", "");
        assert!(short.too_short && !short.ready);
        let differs = check_password(true, "old", "Velvet-Comet-1", "Velvet-Comet-2");
        assert!(differs.mismatch && !differs.ready);
        assert!(check_password(true, "old", "Velvet-Comet-1", "Velvet-Comet-1").ready);
        // The current one is needed only when there is one.
        assert!(!check_password(true, "", "Velvet-Comet-1", "Velvet-Comet-1").ready);
        assert!(check_password(false, "", "Velvet-Comet-1", "Velvet-Comet-1").ready);
        // Characters, not bytes.
        assert!(!check_password(false, "", "éééééééé", "éééééééé").too_short);
    }

    #[test]
    fn only_a_refusal_about_the_current_password_says_it_is_wrong() {
        let wrong = "the current password is wrong";
        assert!(wrong_current(true, Some(400), Some("bad_request"), wrong));
        assert!(wrong_current(
            true,
            Some(400),
            Some("wrong_current_password"),
            "no"
        ));
        // No password to be wrong about: the account sets its first one.
        assert!(!wrong_current(false, Some(400), Some("bad_request"), wrong));
        // Other refusals keep the server's words.
        let short = "password must be at least 8 characters";
        assert!(!wrong_current(true, Some(400), Some("bad_request"), short));
        assert!(!wrong_current(
            true,
            Some(400),
            Some("weak_password"),
            "too weak"
        ));
        assert!(!wrong_current(true, Some(429), Some("rate_limited"), wrong));
        assert!(!wrong_current(true, None, None, wrong));
    }

    #[test]
    fn each_key_lifetime_becomes_the_requests_days() {
        for days in KEY_LIFETIMES {
            assert_eq!(key_lifetime_days(&key_lifetime_value(days)), days);
        }
        assert_eq!(key_lifetime_label(None), "Never");
        assert_eq!(key_lifetime_label(Some(30)), "30 days");
        assert_eq!(key_lifetime_label(Some(1)), "1 day");
        // Nothing the server would refuse.
        assert_eq!(key_lifetime_days("0"), None);
        assert_eq!(key_lifetime_days("99999"), None);
        assert_eq!(key_lifetime_days("soon"), None);
        let key = new_key("  backup   script ", true, Some(90)).unwrap();
        assert_eq!(key.name, "backup script");
        assert!(key.read_only);
        assert_eq!(key.expires_days, Some(90));
        let empty = new_key("   ", false, None).unwrap_err();
        assert_eq!(key_error_text(&empty), "Give the key a name.");
        let long = new_key(&"n".repeat(65), false, None).unwrap_err();
        assert!(key_error_text(&long).contains("64"));
    }

    #[test]
    fn describes_keys() {
        let now = now();
        let mut key = ApiKey {
            id: 1,
            name: "backup".into(),
            prefix: "sbk_12345678".into(),
            read_only: false,
            created_at: now,
            last_used_at: None,
            expires_at: None,
        };
        assert_eq!(key_status(&key, now), "Never used · Never expires");
        assert!(!key_expired(&key, now));
        key.last_used_at = Some(now - TimeDelta::minutes(3));
        let gone = now - TimeDelta::days(1);
        key.expires_at = Some(gone);
        let day = gone.with_timezone(&Local).format("%Y-%m-%d").to_string();
        assert_eq!(
            key_status(&key, now),
            format!("Used 3 minutes ago · Expired {day}")
        );
        assert!(key_expired(&key, now));
    }

    #[test]
    fn the_version_is_the_release_tags() {
        assert_eq!(app_version(Some("v0.2.0-beta.1")), "0.2.0-beta.1");
        assert_eq!(app_version(Some("0.3.0")), "0.3.0");
        assert_eq!(app_version(Some(" ")), env!("CARGO_PKG_VERSION"));
        assert_eq!(app_version(None), env!("CARGO_PKG_VERSION"));
    }
}
