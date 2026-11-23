//! Saved searches in the gallery: a star beside a search's title saves it or
//! forgets it, and the plain gallery lists them as chips above the grid, each
//! running its search, with a way to remove it.

use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{SavedSearch, clean_saved_search};

use crate::api;
use crate::components::icons::{Ico, Icon};
use crate::components::snackbar::{notify, notify_undo};
use crate::logic::saved::{add, find, name_for, remove, tidy};
use crate::route;
use crate::store;

thread_local! {
    // The account's saved searches and whose they are; `None` until loaded.
    // Reference counted, as it outlives any one gallery.
    static SAVED: ArcRwSignal<Option<(String, Vec<SavedSearch>)>> = ArcRwSignal::new(None);
}

/// This account's saved searches, `None` until they're known. Read inside a
/// view, it redraws when they change.
fn current() -> Option<Vec<SavedSearch>> {
    let owner = store::owner();
    SAVED.with(|s| {
        s.with(|s| {
            s.as_ref()
                .filter(|(whose, _)| *whose == owner)
                .map(|(_, list)| list.clone())
        })
    })
}

/// Changes this account's list, once it's known.
fn update(change: impl FnOnce(&mut Vec<SavedSearch>)) {
    let owner = store::owner();
    SAVED.with(|s| {
        s.update(|s| {
            if let Some((whose, list)) = s.as_mut()
                && *whose == owner
            {
                change(list);
            }
        });
    });
}

/// Asks the server for the account's saved searches. The gallery shows the
/// ones kept from before in the meantime.
pub fn load() {
    let owner = store::owner();
    spawn_local(async move {
        if let Ok(list) = api::call(|c| async move { c.saved_searches().await }).await
            && store::owner() == owner
        {
            SAVED.with(|s| s.set(Some((owner, list))));
        }
    });
}

/// Saves `query` under `name` and puts it in the list. `None` when the
/// server took it, or the words to show.
async fn save(name: String, query: String) -> Option<String> {
    let (name, query) = match clean_saved_search(&name, &query) {
        Ok(clean) => clean,
        Err(e) => return Some(e.to_string()),
    };
    match api::call(move |c| async move { c.save_search(&name, &query).await }).await {
        Ok(saved) => {
            update(|list| add(list, saved));
            None
        }
        Err(message) => Some(message),
    }
}

/// The star beside a search's title: saves the search, or forgets it.
#[component]
pub fn SaveToggle(#[prop(into)] q: Signal<String>) -> impl IntoView {
    let busy = RwSignal::new(false);
    let list = Memo::new(|_| current());
    let existing = Memo::new(move |_| list.with(|l| l.as_ref().and_then(|l| find(l, &q.get()))));
    let label = move || {
        if existing.get().is_some() {
            t!("search-forget")
        } else {
            t!("search-save")
        }
    };
    let toggle = move |_| {
        if busy.get_untracked() {
            return;
        }
        let query = tidy(&q.get_untracked());
        busy.set(true);
        let existing = existing.get_untracked();
        spawn_local(async move {
            let failed = match existing {
                Some(id) => {
                    match api::call(move |c| async move { c.delete_search(id).await }).await {
                        Ok(()) => {
                            update(|list| {
                                remove(list, id);
                            });
                            None
                        }
                        Err(message) => Some(t!("search-forget-failed", reason = message)),
                    }
                }
                None => save(name_for(&query), query)
                    .await
                    .map(|message| t!("search-save-failed", reason = message)),
            };
            busy.try_set(false);
            if let Some(message) = failed {
                notify(message);
            }
        });
    };
    view! {
        <button
            type="button"
            class="icon-btn save-search"
            class:on=move || existing.get().is_some()
            title=label
            aria-label=label
            aria-pressed=move || existing.get().is_some().to_string()
            disabled=move || busy.get() || list.with(Option::is_none)
            on:click=toggle
        >
            {move || {
                let icon = if existing.get().is_some() { Icon::Star } else { Icon::StarOutline };
                view! { <Ico icon=icon /> }
            }}
        </button>
    }
}

/// Takes `saved` off the list, with a way to put it back.
fn forget(saved: SavedSearch) {
    let id = saved.id;
    update(|list| {
        remove(list, id);
    });
    spawn_local(async move {
        match api::call(move |c| async move { c.delete_search(id).await }).await {
            Ok(()) => {
                let (name, query) = (saved.name.clone(), saved.query.clone());
                notify_undo(
                    t!("saved-removed", name = saved.name),
                    t!("press-undo"),
                    move || {
                        let (name, query) = (name.clone(), query.clone());
                        spawn_local(async move {
                            if let Some(message) = save(name, query).await {
                                notify(t!("search-save-failed", reason = message));
                            }
                        });
                    },
                );
            }
            Err(message) => {
                update(|list| add(list, saved));
                notify(t!("search-forget-failed", reason = message));
            }
        }
    });
}

/// The saved searches as chips, on the plain gallery.
#[component]
pub fn SavedList() -> impl IntoView {
    move || {
        let list = current().filter(|l| !l.is_empty())?;
        let chips = list
            .into_iter()
            .map(|s| {
                let name = s.name.clone();
                let remove_label = t!("saved-remove", name = name.clone());
                view! {
                    <span class="chip removable">
                        <a href=route::gallery(&s.query) title=s.query.clone()>
                            <Ico icon=Icon::Star size=16 />
                            <span class="saved-name">{name}</span>
                        </a>
                        <button
                            type="button"
                            class="chip-x"
                            title=remove_label.clone()
                            aria-label=remove_label
                            on:click=move |_| forget(s.clone())
                        >
                            <Ico icon=Icon::Close />
                        </button>
                    </span>
                }
            })
            .collect_view();
        Some(view! {
            <nav class="saved-searches" aria-label=move || t!("saved-searches")>
                {chips}
            </nav>
        })
    }
}
