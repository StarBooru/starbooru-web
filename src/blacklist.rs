//! The reader's own blacklist: an account's, kept by the server, or a
//! guest's, kept on the device (`store::guest_blacklist`) and sent with
//! every read by the guest's client (`api.rs`). Settings changes it here,
//! so a change and its Undo work wherever they're pressed. Each change goes
//! out at once, one request at a time with the latest winning, and drops
//! what the app kept of posts the list may now hide or show.
//!
//! Also the posts uncovered with Show in the viewer: for the rest of the
//! visit, or until the list changes.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::mention;
use crate::components::snackbar::notify_undo;
use crate::logic::blacklist::{self, guest_owner};
use crate::pages::{gallery, home, post};
use crate::{api, instance, store};

/// The list as it is here: whose it is, and its entries or why they
/// couldn't be read.
#[derive(Clone, Debug, PartialEq)]
struct Kept {
    owner: String,
    list: Result<Vec<String>, String>,
}

/// How the last change went.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Saving {
    #[default]
    Idle,
    Busy,
    Saved,
    /// The list stays as it was changed here; Try again sends it again.
    Failed(String),
}

thread_local! {
    // Reference counted, as they outlive every page.
    static LIST: ArcRwSignal<Option<Kept>> = ArcRwSignal::new(None);
    static SAVING: ArcRwSignal<Saving> = ArcRwSignal::new(Saving::Idle);
    /// The list still to send, for the owner it was made for.
    static WANTED: RefCell<Option<(String, Vec<String>)>> = const { RefCell::new(None) };
    static SENDING: Cell<bool> = const { Cell::new(false) };
    static UNCOVERED: RefCell<HashSet<i64>> = RefCell::new(HashSet::new());
    /// Bumped when the uncovered posts change.
    static SHOWN: ArcRwSignal<u32> = ArcRwSignal::new(0);
}

/// Whose list the app uses now: the account's, or the guests' of a server,
/// with that server.
fn whose() -> Option<(String, Option<String>)> {
    if store::session_untracked().is_some() {
        return Some((store::owner(), None));
    }
    let server = instance::guest_server()?;
    Some((guest_owner(&server), Some(server)))
}

/// The list, once read, for whoever reads now. Read inside a view, it
/// follows each change.
pub fn entries() -> Option<Result<Vec<String>, String>> {
    let owner = whose()?.0;
    LIST.with(|l| {
        l.with(|l| {
            l.as_ref()
                .filter(|k| k.owner == owner)
                .map(|k| k.list.clone())
        })
    })
}

fn entries_untracked() -> Option<Vec<String>> {
    let owner = whose()?.0;
    LIST.with(|l| {
        l.with_untracked(|l| {
            l.as_ref()
                .filter(|k| k.owner == owner)
                .and_then(|k| k.list.clone().ok())
        })
    })
}

/// How the last change went. Read inside a view, it follows.
pub fn saving() -> Saving {
    SAVING.with(|s| s.get())
}

fn set_saving(now: Saving) {
    SAVING.with(|s| {
        if s.with_untracked(|s| *s != now) {
            s.set(now);
        }
    });
}

fn keep(owner: String, list: Result<Vec<String>, String>) {
    LIST.with(|l| l.set(Some(Kept { owner, list })));
}

/// Reads the list, unless it's here already for whoever reads now: an
/// account's from the server; a guest's from the device, or the server's
/// default until they change it.
pub fn load() {
    let Some((owner, guest)) = whose() else {
        return;
    };
    let here = LIST.with(|l| {
        l.with_untracked(|l| {
            l.as_ref()
                .is_some_and(|k| k.owner == owner && k.list.is_ok())
        })
    });
    if here {
        return;
    }
    if let Some(server) = &guest
        && let Some(list) = store::guest_blacklist(server)
    {
        keep(owner, Ok(list));
        return;
    }
    set_saving(Saving::Idle);
    spawn_local(async move {
        let answer = api::call(|c| async move { c.blacklist().await }).await;
        if whose().map(|(o, _)| o).as_deref() == Some(owner.as_str()) {
            keep(owner, answer.map(|b| b.entries));
        }
    });
}

/// Makes `next` the list: here at once, then on the server or the device.
pub fn set(next: Vec<String>) {
    let Some((owner, guest)) = whose() else {
        return;
    };
    keep(owner.clone(), Ok(next.clone()));
    changed();
    match guest {
        Some(server) => {
            store::set_guest_blacklist(&server, Some(next));
            set_saving(Saving::Saved);
        }
        None => {
            WANTED.with_borrow_mut(|w| *w = Some((owner, next)));
            set_saving(Saving::Busy);
            if !SENDING.get() {
                send();
            }
        }
    }
}

/// Sends the list wanted last, and again while it changes on the way.
fn send() {
    SENDING.set(true);
    spawn_local(async move {
        while let Some((owner, list)) = WANTED.with_borrow_mut(Option::take) {
            let sent = list.clone();
            let answer = api::call(move |c| async move { c.set_blacklist(&sent).await }).await;
            let newer = WANTED.with_borrow(Option::is_some);
            let mine = store::owner() == owner;
            match answer {
                // As the server kept it: an alias turns into its tag.
                Ok(kept) if mine && !newer => {
                    if kept.entries != list {
                        keep(owner, Ok(kept.entries));
                        changed();
                    }
                    set_saving(Saving::Saved);
                }
                Ok(_) => {}
                Err(message) if mine && !newer => set_saving(Saving::Failed(message)),
                Err(_) => {}
            }
        }
        SENDING.set(false);
    });
}

/// Sends the list as it is here again, after a failure.
pub fn retry() {
    if let Some(list) = entries_untracked() {
        set(list);
    }
}

/// Whether a guest has a list of their own on this device. Read inside a
/// view, it follows.
pub fn guest_changed() -> bool {
    let Some(server) = instance::guest_server() else {
        return false;
    };
    let owner = guest_owner(&server);
    store::prefs()
        .guest_blacklists
        .iter()
        .any(|l| l.owner == owner)
}

/// A guest goes back to the server's default list.
pub fn use_default() {
    let Some(server) = instance::guest_server() else {
        return;
    };
    store::set_guest_blacklist(&server, None);
    LIST.with(|l| l.set(None));
    changed();
    load();
}

/// Takes `entry` off, with Undo in the notice that says so. `label` is how
/// the notice names it.
pub fn remove(entry: String, label: String) {
    let Some(before) = entries_untracked() else {
        return;
    };
    let owner = whose().map(|(o, _)| o);
    set(blacklist::without(&before, &entry));
    notify_undo(
        t!("blacklist-removed", entry = label),
        t!("press-undo"),
        move || {
            if whose().map(|(o, _)| o) != owner {
                return;
            }
            if let Some(now) = entries_untracked() {
                set(blacklist::put_back(&now, &before, &entry));
            }
        },
    );
}

/// What the app kept of posts can't be trusted after a change: the
/// gallery's lists and the previews, the viewer's copies (with what they
/// matched), the posts named in the forum, the home page and the posts
/// uncovered.
fn changed() {
    gallery::forget_lists();
    post::forget_all_kept();
    mention::forget_seen();
    home::forget();
    UNCOVERED.with_borrow_mut(HashSet::clear);
    SHOWN.with(|s| s.update(|n| *n = n.wrapping_add(1)));
}

/// Shows post `id` for the rest of the visit, whatever the list says.
pub fn uncover(id: i64) {
    UNCOVERED.with_borrow_mut(|u| u.insert(id));
    SHOWN.with(|s| s.update(|n| *n = n.wrapping_add(1)));
}

/// Whether post `id` was uncovered, outside a view.
pub fn uncovered(id: i64) -> bool {
    UNCOVERED.with_borrow(|u| u.contains(&id))
}

/// Makes the view calling it follow the posts uncovered.
pub fn track_uncovered() {
    SHOWN.with(|s| s.track());
}

/// Forgets the list at sign-out or sign-in, so the next reader's is read.
pub fn forget() {
    LIST.with(|l| l.set(None));
    WANTED.with_borrow_mut(|w| *w = None);
    set_saving(Saving::Idle);
    UNCOVERED.with_borrow_mut(HashSet::clear);
}
