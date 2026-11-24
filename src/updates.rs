//! New StarBooru versions, for administrators: what the server last said
//! (`GET /admin/updates`), kept for the notice at the top of every page and
//! for Administration › Updates. The shell asks when the app starts and then
//! every few hours. Nobody else asks, and a server that doesn't know updates
//! (before 0.2.0), or fails to answer, shows nothing.

use std::cell::RefCell;

use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_location, use_query_map};
use starbooru_core::{Role, UpdateStatus};

use crate::components::icons::{Ico, Icon};
use crate::components::snackbar::notify_undo;
use crate::logic::admin::Tab;
use crate::logic::updates::{NOTICE_ROUND_MS, due, notice, quiet_on};
use crate::{api, dom, store};

thread_local! {
    // Reference counted, so no component's owner can dispose of it. The
    // status comes with the account it was for.
    static KNOWN: ArcRwSignal<Option<(String, UpdateStatus)>> = ArcRwSignal::new(None);
    // The account whose server answered that it doesn't know updates.
    static MISSING: ArcRwSignal<Option<String>> = ArcRwSignal::new(None);
    static ASKED: RefCell<Option<(String, DateTime<Utc>)>> = const { RefCell::new(None) };
}

fn admin_untracked() -> bool {
    store::session_untracked().is_some_and(|s| s.user.role == Role::Admin)
}

/// What the server last said to the account signed in. Read inside a view,
/// it redraws when that changes.
pub fn known() -> Option<UpdateStatus> {
    let owner = store::owner();
    KNOWN.with(|k| {
        k.with(|k| {
            k.as_ref()
                .filter(|(o, _)| *o == owner)
                .map(|(_, s)| s.clone())
        })
    })
}

/// Keeps what Administration › Updates heard, so the notice agrees with it.
pub fn put(status: UpdateStatus) {
    let owner = store::owner();
    ASKED.with_borrow_mut(|a| *a = Some((owner.clone(), Utc::now())));
    keep(owner, Some(status));
}

/// The server signed in to is from before updates: Administration leaves
/// them out. Read inside a view, it redraws when that changes.
pub fn missing() -> bool {
    let owner = store::owner();
    MISSING.with(|m| m.with(|m| m.as_deref() == Some(owner.as_str())))
}

/// Keeps an answer for `owner`: a status, or `None` from a server that
/// doesn't know updates.
fn keep(owner: String, answer: Option<UpdateStatus>) {
    let missing = answer.is_none();
    if MISSING.with(|m| m.with_untracked(|m| (m.as_deref() == Some(owner.as_str())) != missing)) {
        MISSING.with(|m| m.set(missing.then(|| owner.clone())));
    }
    KNOWN.with(|k| k.set(answer.map(|s| (owner, s))));
}

/// What Administration › Updates found: a server from before updates.
pub fn put_missing() {
    keep(store::owner(), None);
}

/// Asks the server, when it's time and an administrator is signed in.
async fn ask_when_due() {
    let owner = store::owner();
    if owner.is_empty() || !admin_untracked() {
        return;
    }
    let last = ASKED.with_borrow(|a| a.as_ref().filter(|(o, _)| *o == owner).map(|(_, at)| *at));
    if !due(last, Utc::now()) {
        return;
    }
    ASKED.with_borrow_mut(|a| *a = Some((owner.clone(), Utc::now())));
    let answer = api::attempt(|c| async move { c.update_status().await }).await;
    if store::owner() != owner {
        return;
    }
    // Any other failure keeps what was known: the next round asks again.
    match answer {
        Ok(status) => keep(owner, Some(status)),
        Err(r) if r.status == Some(404) => keep(owner, None),
        Err(_) => {}
    }
}

/// A new StarBooru is out: the version, a way to the update page, and a way
/// to hide it until the next one. On every page in the shell but the
/// viewers and the update page itself.
#[component]
pub fn UpdateNotice() -> impl IntoView {
    let alive = StoredValue::new(());
    spawn_local(async move {
        while alive.try_get_value().is_some() {
            ask_when_due().await;
            dom::sleep(NOTICE_ROUND_MS).await;
        }
    });
    let location = use_location();
    let query = use_query_map();
    let quiet = Memo::new(move |_| {
        location
            .pathname
            .with(|p| quiet_on(p, query.with(|q| q.get("tab")).as_deref()))
    });
    let version = Memo::new(move |_| {
        let admin = store::session().is_some_and(|s| s.user.role == Role::Admin);
        let status = known().filter(|_| admin)?;
        notice(&status, store::dismissed_update().as_deref())
    });
    // Folding away, before it goes.
    let leaving = RwSignal::new(false);
    let hide = move |_| {
        let Some(v) = version.get_untracked() else {
            return;
        };
        if leaving.get_untracked() {
            return;
        }
        leaving.set(true);
        spawn_local(async move {
            dom::sleep(260).await;
            let hidden = store::dismiss_update(&v);
            leaving.try_set(false);
            notify_undo(t!("updates-notice-hidden"), t!("press-undo"), move || {
                store::undo_dismiss_update(hidden.clone());
            });
        });
    };
    let shown = move || !quiet.get() && version.with(Option::is_some);
    let text = move || {
        version
            .get()
            .map(|v| t!("updates-notice", version = v))
            .unwrap_or_default()
    };
    view! {
        <Show when=shown>
            <div class="update-notice-slot" class:leaving=move || leaving.get()>
                <aside class="update-notice" aria-label=move || t!("updates-notice-label")>
                    <a class="update-notice-link" href=Tab::Updates.href()>
                        <span class="update-notice-icon">
                            <Ico icon=Icon::ArrowUp />
                        </span>
                        <span class="update-notice-text">
                            <strong>{text}</strong>
                            <span>{move || t!("updates-notice-more")}</span>
                        </span>
                        <Ico icon=Icon::ChevronRight />
                    </a>
                    <button
                        type="button"
                        class="icon-btn small"
                        aria-label=move || t!("updates-notice-hide")
                        title=move || t!("updates-notice-hide")
                        on:click=hide
                    >
                        <Ico icon=Icon::Close />
                    </button>
                </aside>
            </div>
        </Show>
    }
}
