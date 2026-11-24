//! Private and public servers, without a browser: the changes a switch of
//! mode or of score kind lists, in the app's own words, why a switch was
//! refused, which screen someone without a session gets, and the page a
//! sign-in comes back to.

use starbooru_core::mode::changes;
use starbooru_core::{Change, Mode, ScoreKind};

/// What a switch deletes or converts, one kind of thing at a time
/// (`starbooru_core::mode::changes`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    ShareLinks,
    BooruAccounts,
    RemoteSearches,
    StarVotesConverted,
    StarVotesDropped,
    LikeVotesConverted,
}

impl Kind {
    pub const ALL: [Kind; 6] = [
        Kind::ShareLinks,
        Kind::BooruAccounts,
        Kind::RemoteSearches,
        Kind::StarVotesConverted,
        Kind::StarVotesDropped,
        Kind::LikeVotesConverted,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Kind::ShareLinks => changes::SHARE_LINKS,
            Kind::BooruAccounts => changes::BOORU_ACCOUNTS,
            Kind::RemoteSearches => changes::REMOTE_SEARCHES,
            Kind::StarVotesConverted => changes::STAR_VOTES_CONVERTED,
            Kind::StarVotesDropped => changes::STAR_VOTES_DROPPED,
            Kind::LikeVotesConverted => changes::LIKE_VOTES_CONVERTED,
        }
    }

    pub fn parse(s: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.as_str() == s)
    }

    /// Whether these go for good, rather than turning into something else.
    pub fn deletes(self) -> bool {
        !matches!(self, Kind::StarVotesConverted | Kind::LikeVotesConverted)
    }

    /// How many of them, as a noun: "12 share links".
    pub fn noun(self, count: u64) -> String {
        match self {
            Kind::ShareLinks => t!("change-share-links", count = count),
            Kind::BooruAccounts => t!("change-booru-accounts", count = count),
            Kind::RemoteSearches => t!("change-remote-searches", count = count),
            Kind::StarVotesConverted => t!("change-star-votes", count = count),
            Kind::StarVotesDropped => t!("change-three-stars", count = count),
            Kind::LikeVotesConverted => t!("change-like-votes", count = count),
        }
    }

    /// What happens to them, "will be deleted", or once `done`, what did:
    /// "deleted".
    pub fn fate(self, done: bool) -> String {
        match (self, done) {
            (Kind::ShareLinks | Kind::BooruAccounts | Kind::RemoteSearches, false) => {
                t!("change-fate-deleted")
            }
            (Kind::ShareLinks | Kind::BooruAccounts | Kind::RemoteSearches, true) => {
                t!("change-done-deleted")
            }
            (Kind::StarVotesConverted, false) => t!("change-fate-likes"),
            (Kind::StarVotesConverted, true) => t!("change-done-likes"),
            (Kind::StarVotesDropped, false) => t!("change-fate-neither"),
            (Kind::StarVotesDropped, true) => t!("change-done-neither"),
            (Kind::LikeVotesConverted, false) => t!("change-fate-stars"),
            (Kind::LikeVotesConverted, true) => t!("change-done-stars"),
        }
    }
}

/// One row of a preview, or of what a switch did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// `None` for a kind from a newer server, which keeps its own words.
    pub kind: Option<Kind>,
    pub count: u64,
    /// "12 share links", or the server's sentence.
    pub what: String,
    /// "will be deleted", or nothing for the server's sentence.
    pub fate: String,
    pub deletes: bool,
}

/// The rows for `changes`, in the server's order, leaving out those with
/// nothing to do. `done` words them as what a switch did.
pub fn rows(changes: &[Change], done: bool) -> Vec<Row> {
    changes
        .iter()
        .filter(|c| c.count > 0)
        .map(|c| match Kind::parse(&c.kind) {
            Some(kind) => Row {
                kind: Some(kind),
                count: c.count,
                what: kind.noun(c.count),
                fate: kind.fate(done),
                deletes: kind.deletes(),
            },
            None => Row {
                kind: None,
                count: c.count,
                what: c.message.clone(),
                fate: String::new(),
                deletes: false,
            },
        })
        .collect()
}

/// "a, b and c", in the language's way.
pub fn and_list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => t!("list-and", rest = rest.join(", "), last = last.clone()),
    }
}

/// What the confirmation says goes for good, or `None` when nothing does.
pub fn deletion_warning(rows: &[Row]) -> Option<String> {
    let gone: Vec<String> = rows
        .iter()
        .filter(|r| r.deletes)
        .map(|r| r.what.clone())
        .collect();
    (!gone.is_empty()).then(|| t!("mode-deletes", what = and_list(&gone)))
}

pub fn mode_name(mode: Mode) -> String {
    match mode {
        Mode::Private => t!("mode-private"),
        Mode::Public => t!("mode-public"),
    }
}

pub fn score_name(kind: ScoreKind) -> String {
    match kind {
        ScoreKind::Likes => t!("scores-likes"),
        ScoreKind::Stars => t!("scores-stars"),
    }
}

/// Why the server turned a switch down, from its error code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// No `pg_dump` here: switching is still possible without a backup.
    BackupUnavailable,
    BackupRunning,
    /// Nothing changed.
    BackupFailed,
    SwitchRunning,
    /// Switched already, maybe from another tab.
    SameMode,
    PublicCountsLikes,
    /// Anything else, in the words the request came back with.
    Other,
}

pub fn refusal(code: Option<&str>) -> Refusal {
    match code {
        Some("backup_unavailable") => Refusal::BackupUnavailable,
        Some("backup_running") => Refusal::BackupRunning,
        Some("backup_failed") => Refusal::BackupFailed,
        Some("mode_switch_running") => Refusal::SwitchRunning,
        Some("same_mode") => Refusal::SameMode,
        Some("public_counts_likes") => Refusal::PublicCountsLikes,
        _ => Refusal::Other,
    }
}

/// A refusal in words; `other` is what the request said otherwise.
pub fn refusal_text(why: Refusal, other: &str) -> String {
    match why {
        Refusal::BackupUnavailable => t!("mode-backup-unavailable"),
        Refusal::BackupRunning => t!("mode-backup-running"),
        Refusal::BackupFailed => t!("mode-backup-failed"),
        Refusal::SwitchRunning => t!("mode-switch-running"),
        Refusal::SameMode => t!("mode-same"),
        Refusal::PublicCountsLikes => t!("scores-public-likes"),
        Refusal::Other => other.to_owned(),
    }
}

/// Whether the server lets people without an account browse, as far as the
/// app knows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Guests {
    /// Still asking the server.
    #[default]
    Asking,
    Welcome,
    No,
}

/// What the shell shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    /// Nothing yet, for the moment it takes to ask the server.
    Waiting,
    Member,
    Guest,
    SignIn,
}

/// The shell's screen: the pages for someone signed in, the pages a guest
/// may browse on a server that welcomes them unless they asked to sign in,
/// and the sign-in screen otherwise.
pub fn screen(signed_in: bool, guests: Guests, asked: bool) -> Screen {
    match (signed_in, guests, asked) {
        (true, ..) => Screen::Member,
        (false, Guests::Asking, _) => Screen::Waiting,
        (false, Guests::Welcome, false) => Screen::Guest,
        _ => Screen::SignIn,
    }
}

/// The page a sign-in through the identity provider comes back to, when it
/// is one of this app's: a path, never another site or `/oidc` itself.
pub fn back_to(path: &str) -> Option<&str> {
    let local = path.starts_with('/')
        && !path.starts_with("//")
        && !path.contains('\\')
        && !path.chars().any(char::is_control);
    let looping = path == "/oidc" || path.starts_with("/oidc?") || path.starts_with("/oidc/");
    (local && !looping).then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(kind: &str, count: u64) -> Change {
        Change {
            kind: kind.into(),
            count,
            message: format!("{count} {kind} (server)"),
        }
    }

    #[test]
    fn every_kind_has_its_name_and_words() {
        for kind in Kind::ALL {
            assert_eq!(Kind::parse(kind.as_str()), Some(kind));
            assert!(!kind.noun(2).is_empty());
            assert_ne!(kind.fate(false), kind.fate(true));
        }
        assert_eq!(Kind::parse("comments"), None);
        assert_eq!(Kind::ShareLinks.noun(12), "12 share links");
        assert_eq!(Kind::ShareLinks.noun(1), "1 share link");
        assert_eq!(Kind::StarVotesDropped.noun(6), "6 votes of 3 stars");
        let deleted: Vec<Kind> = Kind::ALL.into_iter().filter(|k| k.deletes()).collect();
        assert_eq!(
            deleted,
            [
                Kind::ShareLinks,
                Kind::BooruAccounts,
                Kind::RemoteSearches,
                Kind::StarVotesDropped
            ]
        );
    }

    #[test]
    fn a_preview_lists_what_has_something_to_do_in_our_words() {
        let rows = rows(
            &[
                change("share_links", 12),
                change("booru_accounts", 0),
                change("star_votes_converted", 40),
                change("comments", 3),
            ],
            false,
        );
        assert_eq!(rows.len(), 3);
        let first = rows.first().unwrap();
        assert_eq!(first.kind, Some(Kind::ShareLinks));
        assert_eq!(first.what, "12 share links");
        assert_eq!(first.fate, "will be deleted");
        assert!(first.deletes);
        let votes = rows.get(1).unwrap();
        assert!(!votes.deletes);
        assert!(votes.fate.starts_with("will become likes and dislikes"));
        // A kind from a newer server keeps the server's sentence.
        let unknown = rows.get(2).unwrap();
        assert_eq!(unknown.kind, None);
        assert_eq!(unknown.what, "3 comments (server)");
        assert!(unknown.fate.is_empty() && !unknown.deletes);
        // What a switch did, in the past.
        let done = super::rows(&[change("share_links", 2)], true);
        assert_eq!(done.first().map(|r| r.fate.as_str()), Some("deleted"));
    }

    #[test]
    fn the_confirmation_names_what_goes_for_good() {
        let all = rows(
            &[
                change("share_links", 12),
                change("booru_accounts", 3),
                change("remote_searches", 1),
                change("star_votes_converted", 40),
                change("star_votes_dropped", 6),
            ],
            false,
        );
        assert_eq!(
            deletion_warning(&all).as_deref(),
            Some(
                "Switching deletes 12 share links, 3 saved booru accounts, 1 saved booru search and 6 votes of 3 stars for good."
            )
        );
        let one = rows(&[change("share_links", 1)], false);
        assert_eq!(
            deletion_warning(&one).as_deref(),
            Some("Switching deletes 1 share link for good.")
        );
        assert_eq!(
            deletion_warning(&rows(&[change("like_votes_converted", 9)], false)),
            None
        );
        assert_eq!(deletion_warning(&[]), None);
        assert_eq!(and_list(&["a".into(), "b".into()]), "a and b");
        assert_eq!(and_list(&[]), "");
    }

    #[test]
    fn each_refusal_says_what_to_do() {
        for (code, why) in [
            ("backup_unavailable", Refusal::BackupUnavailable),
            ("backup_running", Refusal::BackupRunning),
            ("backup_failed", Refusal::BackupFailed),
            ("mode_switch_running", Refusal::SwitchRunning),
            ("same_mode", Refusal::SameMode),
            ("public_counts_likes", Refusal::PublicCountsLikes),
        ] {
            assert_eq!(refusal(Some(code)), why);
            assert!(!refusal_text(why, "x").is_empty());
            assert_ne!(refusal_text(why, "x"), "x");
        }
        assert_eq!(refusal(Some("forbidden")), Refusal::Other);
        assert_eq!(refusal(None), Refusal::Other);
        assert_eq!(refusal_text(Refusal::Other, "server words"), "server words");
        assert_ne!(mode_name(Mode::Private), mode_name(Mode::Public));
        assert_ne!(score_name(ScoreKind::Likes), score_name(ScoreKind::Stars));
    }

    #[test]
    fn a_guest_browses_until_asking_to_sign_in() {
        use Screen::*;
        assert_eq!(screen(true, Guests::No, false), Member);
        assert_eq!(screen(true, Guests::Welcome, true), Member);
        assert_eq!(screen(true, Guests::Asking, false), Member);
        assert_eq!(screen(false, Guests::Asking, false), Waiting);
        assert_eq!(screen(false, Guests::Welcome, false), Guest);
        assert_eq!(screen(false, Guests::Welcome, true), SignIn);
        // A private server: the sign-in screen first, as before.
        assert_eq!(screen(false, Guests::No, false), SignIn);
        assert_eq!(screen(false, Guests::No, true), SignIn);
    }

    #[test]
    fn a_sign_in_comes_back_only_to_a_page_of_the_app() {
        for path in ["/", "/post/5?q=cat", "/explore?days=30", "/tag?name=a%26b"] {
            assert_eq!(back_to(path), Some(path), "{path}");
        }
        for path in [
            "",
            "post/5",
            "//evil.example",
            "/\\evil.example",
            "https://evil.example/",
            "/oidc",
            "/oidc?code=1",
            "/a\nb",
        ] {
            assert_eq!(back_to(path), None, "{path:?}");
        }
    }
}
