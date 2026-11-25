//! Administration › Server › Default blacklist: the list new accounts start
//! with and guests get until they change theirs. Existing accounts keep
//! their own. Each change is saved at once, as in Settings, and the list
//! then shows what the server kept.

use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::blacklist::{Entry, MAX_EXCLUDE};
use starbooru_core::{Rating, ServerSettings};

use super::server::save_part;
use crate::components::banner::ErrorBanner;
use crate::components::blacklist::{BlacklistEditor, entry_label};
use crate::components::icons::{Ico, Icon};
use crate::components::snackbar::notify_undo;
use crate::logic::blacklist::{Typed, put_back, replaced, toggled, with_typed, without};

#[component]
pub(super) fn DefaultBlacklistCard(settings: RwSignal<Option<ServerSettings>>) -> impl IntoView {
    let entries = Memo::new(move |_| {
        settings.with(|s| {
            s.as_ref()
                .and_then(|s| s.default_blacklist.clone())
                .unwrap_or_default()
        })
    });
    // The list a change is making, shown at once; after a failure it stays
    // for Try again.
    let pending = RwSignal::new(None::<Vec<String>>);
    let shown = Memo::new(move |_| pending.get().unwrap_or_else(|| entries.get()));
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let saved = RwSignal::new(false);

    let save = move |next: Vec<String>| {
        if busy.get_untracked() {
            return;
        }
        pending.set(Some(next.clone()));
        busy.set(true);
        error.set(None);
        saved.set(false);
        spawn_local(async move {
            let outcome = save_part(settings, move |s| s.default_blacklist = Some(next)).await;
            busy.try_set(false);
            match outcome {
                Ok(_) => {
                    pending.try_set(None);
                    saved.try_set(true);
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };
    let remove = move |entry: String| {
        let before = shown.get_untracked();
        save(without(&before, &entry));
        notify_undo(
            t!("blacklist-removed", entry = entry_label(&entry)),
            t!("press-undo"),
            move || {
                if let Some(now) = shown.try_get_untracked() {
                    save(put_back(&now, &before, &entry));
                }
            },
        );
    };
    let on_add = Callback::new(move |typed: Typed| {
        save(with_typed(&shown.get_untracked(), &typed));
    });
    let on_remove = Callback::new(remove);
    let on_toggle = Callback::new(move |rating: Rating| {
        let entry = Entry::Rating(rating).as_entry();
        if shown.with_untracked(|e| e.contains(&entry)) {
            remove(entry);
        } else {
            save(toggled(&shown.get_untracked(), rating));
        }
    });
    let on_replace = Callback::new(move |(from, to): (String, String)| {
        save(replaced(&shown.get_untracked(), &from, &to));
    });
    let retry = Callback::new(move |()| {
        if let Some(next) = pending.get_untracked() {
            save(next);
        }
    });

    view! {
        <section class="card blacklist-card" id="default-blacklist">
            <div class="card-head">
                <h2 class="card-title">{move || t!("admin-blacklist-title")}</h2>
                <Show when=move || busy.get()>
                    <span class="spinner small" aria-hidden="true"></span>
                </Show>
                <Show when=move || saved.get()>
                    <span class="saved-note" role="status">
                        <Ico icon=Icon::Check size=18 />
                        {move || t!("admin-saved")}
                    </span>
                </Show>
            </div>
            <p class="muted hint">{move || t!("admin-blacklist-lead")}</p>
            {move || error.get().map(|message| view! { <ErrorBanner message=message on_retry=retry /> })}
            <BlacklistEditor
                id="default-blacklist"
                entries=shown
                max=MAX_EXCLUDE
                busy=busy
                on_add=on_add
                on_remove=on_remove
                on_toggle=on_toggle
                on_replace=on_replace
            />
        </section>
    }
}
