//! Settings › Viewing › Blacklist: the tags and ratings the reader never
//! wants to see. Each change is saved at once (`crate::blacklist`) and the
//! list then shows what the server kept, so an alias turns into its tag in
//! place. A guest's list stays on this device, and can go back to the
//! server's default.

use leptos::prelude::*;
use starbooru_core::Rating;
use starbooru_core::blacklist::{Entry, MAX_BLACKLIST, MAX_EXCLUDE};

use crate::blacklist::{self, Saving};
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::blacklist::{BlacklistEditor, entry_label};
use crate::components::icons::{Ico, Icon};
use crate::instance;
use crate::logic::blacklist::{Typed, replaced, toggled, with_typed};

#[component]
pub(super) fn BlacklistCard() -> impl IntoView {
    blacklist::load();
    let list = Memo::new(|_| blacklist::entries());
    let entries = Signal::derive(move || list.get().and_then(Result::ok).unwrap_or_default());
    let saving = Memo::new(|_| blacklist::saving());
    let guest = Memo::new(|_| instance::guest());
    // A guest's list goes with every read, and the server takes up to 100.
    let max = if instance::guest() {
        MAX_EXCLUDE
    } else {
        MAX_BLACKLIST
    };

    let on_add = Callback::new(move |typed: Typed| {
        blacklist::set(with_typed(&entries.get_untracked(), &typed));
    });
    let on_remove = Callback::new(move |tag: String| {
        let label = entry_label(&tag);
        blacklist::remove(tag, label);
    });
    let on_toggle = Callback::new(move |rating: Rating| {
        let entry = Entry::Rating(rating).as_entry();
        if entries.with_untracked(|e| e.contains(&entry)) {
            blacklist::remove(entry, entry_label(&Entry::Rating(rating).as_entry()));
        } else {
            blacklist::set(toggled(&entries.get_untracked(), rating));
        }
    });
    let on_replace = Callback::new(move |(from, to): (String, String)| {
        blacklist::set(replaced(&entries.get_untracked(), &from, &to));
    });

    // How the last change went, beside the title where the eye goes after
    // a press.
    let status = move || match saving.get() {
        Saving::Busy => Some(
            view! {
                <span class="muted saving-note" role="status">
                    <span class="spinner small" aria-hidden="true"></span>
                    {move || t!("blacklist-saving")}
                </span>
            }
            .into_any(),
        ),
        Saving::Saved => Some(
            view! {
                <span class="saved-note" role="status">
                    <Ico icon=Icon::Check size=18 />
                    {move || t!("admin-saved")}
                </span>
            }
            .into_any(),
        ),
        Saving::Idle | Saving::Failed(_) => None,
    };
    // What was typed stays; Try again sends it.
    let failure = move || match saving.get() {
        Saving::Failed(message) => Some(view! {
            <ErrorBanner message=message on_retry=Callback::new(move |()| blacklist::retry()) />
        }),
        _ => None,
    };
    let body = move || match list.get() {
        None => view! {
            <div class="admin-wait">
                <Spinner />
            </div>
        }
        .into_any(),
        Some(Err(message)) => view! {
            <ErrorBanner message=message on_retry=Callback::new(move |()| blacklist::load()) />
        }
        .into_any(),
        Some(Ok(_)) => view! {
            <BlacklistEditor
                id="blacklist"
                entries=entries
                max=max
                on_add=on_add
                on_remove=on_remove
                on_toggle=on_toggle
                on_replace=on_replace
            />
        }
        .into_any(),
    };
    let guest_row = move || {
        guest.get().then(|| {
            view! {
                <div class="blacklist-foot">
                    <span class="muted">{move || t!("blacklist-guest-note")}</span>
                    <button
                        type="button"
                        class="btn text small"
                        disabled=move || !blacklist::guest_changed()
                        on:click=move |_| blacklist::use_default()
                    >
                        <Ico icon=Icon::Restore />
                        {move || t!("blacklist-default")}
                    </button>
                </div>
            }
        })
    };

    view! {
        <section class="card blacklist-card" id="blacklist">
            <div class="card-head">
                <h3 class="card-title">{move || t!("blacklist-title")}</h3>
                {status}
            </div>
            <p class="muted hint">{move || t!("blacklist-lead")}</p>
            {failure}
            {body}
            {guest_row}
        </section>
    }
}
