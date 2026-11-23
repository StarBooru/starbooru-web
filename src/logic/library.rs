//! Importing a whole library from another booru, without a browser: what
//! each system asks for, what the form's mistakes and an import's state say,
//! and whether another import may start.

use chrono::{DateTime, Local, Utc};
use starbooru_core::{
    LibraryError, LibraryImport, LibraryPhase, LibrarySource, LibraryState,
    MAX_ACTIVE_LIBRARY_IMPORTS,
};

/// How often a running import's numbers are read again, in milliseconds.
pub const POLL_MS: i32 = 3000;
/// While nothing runs, the list is read one time in this many, for imports
/// started elsewhere.
pub const IDLE_ROUNDS: u32 = 10;

/// An empty field means "not given".
pub fn given(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

/// Where this system keeps what the form asks for, and what doesn't come
/// across.
pub fn source_hint(source: LibrarySource) -> String {
    match source {
        LibrarySource::Szurubooru => t!("library-hint-szurubooru"),
        LibrarySource::Shimmie2 => t!("library-hint-shimmie2"),
        LibrarySource::Danbooru => t!("library-hint-danbooru"),
        LibrarySource::Stash => t!("library-hint-stash"),
        LibrarySource::Blombooru => t!("library-hint-blombooru"),
        LibrarySource::Monbooru => t!("library-hint-monbooru"),
        LibrarySource::Lanbooru => t!("library-hint-lanbooru"),
    }
}

/// What is wrong with the form, in words.
pub fn form_error(error: &LibraryError) -> String {
    match error {
        LibraryError::Base(_) => t!("library-bad-address"),
        LibraryError::MissingLogin(_) => t!("library-need-login"),
        LibraryError::MissingSecret(_) => t!("library-need-key"),
    }
}

/// Where an import is: waiting, which part it is reading, or how it ended.
pub fn state_label(import: &LibraryImport) -> String {
    match import.state {
        LibraryState::Queued => t!("library-state-queued"),
        LibraryState::Running => match import.phase {
            LibraryPhase::Rules => t!("library-state-rules"),
            LibraryPhase::Posts => t!("library-state-posts"),
            LibraryPhase::Groups => t!("library-state-groups"),
        },
        LibraryState::Done => t!("library-state-done"),
        LibraryState::Stopped => t!("library-state-stopped"),
        LibraryState::Failed => t!("library-state-failed"),
    }
}

/// The class of an import's state pill.
pub fn state_class(state: LibraryState) -> &'static str {
    match state {
        LibraryState::Failed => "pill bad",
        LibraryState::Done => "pill ok",
        LibraryState::Queued | LibraryState::Running | LibraryState::Stopped => "pill",
    }
}

/// "12 of 300 · 2 already here · 1 failed · 40 aliases and implications".
pub fn counts(import: &LibraryImport) -> String {
    let mut parts = vec![match import.total {
        Some(total) => t!("library-count-of", done = import.handled(), total = total),
        None => t!("library-count", done = import.handled()),
    }];
    if import.merged > 0 {
        parts.push(t!("library-count-merged", count = import.merged));
    }
    if import.failed > 0 {
        parts.push(t!("library-count-failed", count = import.failed));
    }
    if import.rules > 0 {
        parts.push(t!("library-count-rules", count = import.rules));
    }
    parts.join(" · ")
}

/// Whether as many imports wait or run as one account may have.
pub fn full(imports: &[LibraryImport]) -> bool {
    let active = imports.iter().filter(|i| i.state.is_active()).count();
    i64::try_from(active).unwrap_or(i64::MAX) >= MAX_ACTIVE_LIBRARY_IMPORTS
}

/// Whether the list is worth reading again soon: something is moving.
pub fn moving(imports: &[LibraryImport]) -> bool {
    imports.iter().any(|i| i.state.is_active())
}

/// How long ago `then` was: "just now", "3 hours ago", "yesterday", and past
/// a month the day itself.
pub fn ago(then: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let secs = (now - then).num_seconds();
    match secs {
        ..60 => t!("ago-now"),
        60..3_600 => t!("ago-minutes", count = secs / 60),
        3_600..86_400 => t!("ago-hours", count = secs / 3_600),
        86_400..172_800 => t!("ago-yesterday"),
        172_800..2_592_000 => t!("ago-days", count = secs / 86_400),
        _ => then.with_timezone(&Local).format("%Y-%m-%d").to_string(),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use starbooru_core::RemoteError;

    use super::*;

    fn import(state: LibraryState, phase: LibraryPhase) -> LibraryImport {
        LibraryImport {
            id: 1,
            source: LibrarySource::Szurubooru,
            base: "http://192.168.1.4:8080".into(),
            state,
            phase,
            total: None,
            imported: 0,
            merged: 0,
            failed: 0,
            rules: 0,
            note: None,
            error: None,
            created_at: Utc::now(),
            finished_at: None,
        }
    }

    #[test]
    fn an_import_says_where_it_is() {
        let label = |state, phase| state_label(&import(state, phase));
        assert_eq!(label(LibraryState::Queued, LibraryPhase::Rules), "Waiting");
        assert_eq!(
            label(LibraryState::Running, LibraryPhase::Rules),
            "Reading aliases and implications"
        );
        assert_eq!(
            label(LibraryState::Running, LibraryPhase::Posts),
            "Importing posts"
        );
        assert_eq!(
            label(LibraryState::Running, LibraryPhase::Groups),
            "Reading pools and likes"
        );
        assert_eq!(label(LibraryState::Done, LibraryPhase::Groups), "Finished");
        assert_eq!(label(LibraryState::Stopped, LibraryPhase::Posts), "Stopped");
        assert_eq!(label(LibraryState::Failed, LibraryPhase::Posts), "Failed");
        assert_eq!(state_class(LibraryState::Failed), "pill bad");
        assert_eq!(state_class(LibraryState::Done), "pill ok");
        assert_eq!(state_class(LibraryState::Running), "pill");
    }

    #[test]
    fn an_import_counts_what_it_did() {
        let mut i = import(LibraryState::Running, LibraryPhase::Posts);
        assert_eq!(counts(&i), "0 done");
        i.imported = 9;
        i.merged = 2;
        i.failed = 1;
        assert_eq!(counts(&i), "12 done · 2 already here · 1 failed");
        i.total = Some(300);
        i.rules = 40;
        assert_eq!(
            counts(&i),
            "12 of 300 · 2 already here · 1 failed · 40 aliases and implications"
        );
    }

    #[test]
    fn the_form_says_what_is_missing() {
        assert_eq!(
            form_error(&LibraryError::Base(RemoteError::BadBase)),
            "That address isn’t valid. Write it like http://192.168.1.4:8080."
        );
        assert_eq!(
            form_error(&LibraryError::MissingLogin("Danbooru")),
            "This site needs the username along with the key."
        );
        assert_eq!(
            form_error(&LibraryError::MissingSecret("Danbooru")),
            "This site needs the key along with the username."
        );
        for source in LibrarySource::ALL {
            assert!(!source_hint(source).starts_with("library-"), "{source:?}");
        }
        assert_eq!(given("  "), None);
        assert_eq!(given(" me "), Some("me".into()));
    }

    #[test]
    fn five_imports_at_once_at_most() {
        let running = import(LibraryState::Running, LibraryPhase::Posts);
        let done = import(LibraryState::Done, LibraryPhase::Groups);
        let mut list = vec![done.clone(); 7];
        assert!(!full(&list));
        assert!(!moving(&list));
        list.extend(vec![running; 4]);
        assert!(!full(&list));
        assert!(moving(&list));
        list.push(import(LibraryState::Queued, LibraryPhase::Rules));
        assert!(full(&list));
    }

    #[test]
    fn a_time_reads_as_how_long_ago() {
        let now = Utc::now();
        let at = |secs: i64| ago(now - Duration::seconds(secs), now);
        assert_eq!(at(-30), "just now");
        assert_eq!(at(59), "just now");
        assert_eq!(at(60), "1 minute ago");
        assert_eq!(at(7_199), "1 hour ago");
        assert_eq!(at(7_200), "2 hours ago");
        assert_eq!(at(90_000), "yesterday");
        assert_eq!(at(3 * 86_400), "3 days ago");
        let old = now - Duration::days(40);
        assert_eq!(
            ago(old, now),
            old.with_timezone(&Local).format("%Y-%m-%d").to_string()
        );
    }
}
