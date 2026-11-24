//! The signed-in account's own profile on a public server: its picture and
//! display name, for the account button and its menu. Asked once a session
//! and again after an edit, which also puts the new one here at once, so
//! the button changes with the page that changed it.

use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::Profile;

use crate::{api, store};

thread_local! {
    // Reference counted, as it outlives every page. Whose it is, and it.
    static MINE: ArcRwSignal<Option<(String, Profile)>> = ArcRwSignal::new(None);
    static ASKED: ArcRwSignal<Option<String>> = ArcRwSignal::new(None);
}

/// The account's profile, once the server said. Read inside a view, it
/// follows each change.
pub fn profile() -> Option<Profile> {
    let owner = store::owner();
    MINE.with(|m| {
        m.with(|m| {
            m.as_ref()
                .filter(|(of, _)| *of == owner && !owner.is_empty())
                .map(|(_, p)| p.clone())
        })
    })
}

/// The same, without following it.
pub fn profile_untracked() -> Option<Profile> {
    untrack(profile)
}

/// Keeps the profile the server gave back after a change.
pub fn set(profile: Profile) {
    let owner = store::owner();
    if !owner.is_empty() {
        MINE.with(|m| m.set(Some((owner, profile))));
    }
}

/// Asks the server for the account's profile, once a session.
pub fn ask() {
    let owner = store::owner();
    if owner.is_empty()
        || ASKED.with(|a| a.with_untracked(|a| a.as_deref() == Some(owner.as_str())))
    {
        return;
    }
    ASKED.with(|a| a.set(Some(owner.clone())));
    spawn_local(async move {
        match api::call(|c| async move { c.my_profile().await }).await {
            Ok(found) if store::owner() == owner => set(found),
            // Asked again on the next page that wants it.
            Err(_) => ASKED.with(|a| a.set(None)),
            Ok(_) => {}
        }
    });
}
