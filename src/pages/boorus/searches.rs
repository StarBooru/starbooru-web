//! Searches on other boorus kept by name on the server: a star in the head
//! keeps the search with its sites or forgets it, and while nothing is
//! searched they show as chips, each going to its own sites.

use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{RemoteSite, SaveRemoteSearch, SavedRemoteSearch};

use crate::api;
use crate::components::icons::{Ico, Icon};
use crate::components::snackbar::notify;
use crate::logic::boorus::{same_sites, search_name, sites_param};
use crate::route;
use crate::store;

thread_local! {
    // The saved searches last seen and whose they are, so their chips don't
    // push the grid down when they load. Reference counted, as it outlives
    // any one page.
    static SAVED: ArcRwSignal<Option<(String, Vec<SavedRemoteSearch>)>> = ArcRwSignal::new(None);
}

/// This account's saved searches, `None` until they're known.
fn current() -> Option<Vec<SavedRemoteSearch>> {
    let owner = store::owner();
    SAVED.with(|s| {
        s.with(|s| {
            s.as_ref()
                .filter(|(whose, _)| *whose == owner)
                .map(|(_, list)| list.clone())
        })
    })
}

fn update(change: impl FnOnce(&mut Vec<SavedRemoteSearch>)) {
    let owner = store::owner();
    SAVED.with(|s| {
        s.update(|s| {
            if let Some((whose, list)) = s.as_mut()
                && *whose == owner
            {
                change(list);
                list.sort_by_key(|s| s.name.to_lowercase());
            }
        });
    });
}

/// Asks the server for them; the ones kept from before show meanwhile.
pub(super) fn load() {
    let owner = store::owner();
    spawn_local(async move {
        if let Ok(list) = api::call(|c| async move { c.remote_searches().await }).await
            && store::owner() == owner
        {
            SAVED.with(|s| s.set(Some((owner, list))));
        }
    });
}

/// The star in the head: keeps this search and its sites, or forgets it.
#[component]
pub(super) fn SaveSearch(q: Memo<String>, sites: Memo<Vec<RemoteSite>>) -> impl IntoView {
    let busy = RwSignal::new(false);
    let list = Memo::new(|_| current());
    let existing = Memo::new(move |_| {
        list.with(|l| {
            l.as_ref().and_then(|l| {
                q.with(|q| {
                    sites.with(|sites| {
                        l.iter()
                            .find(|s| s.query == *q && same_sites(&s.sites, sites))
                            .map(|s| s.id)
                    })
                })
            })
        })
    });
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
        let (q, sites) = (q.get_untracked(), sites.get_untracked());
        let search = SaveRemoteSearch {
            name: search_name(&q, &sites),
            query: q,
            sites,
        };
        let existing = existing.get_untracked();
        busy.set(true);
        spawn_local(async move {
            let failed = match existing {
                Some(id) => {
                    match api::call(move |c| async move { c.delete_remote_search(id).await }).await
                    {
                        Ok(()) => {
                            update(|list| list.retain(|s| s.id != id));
                            None
                        }
                        Err(message) => {
                            Some(untrack(|| t!("search-forget-failed", reason = message)))
                        }
                    }
                }
                None => {
                    match api::call(move |c| async move { c.save_remote_search(&search).await })
                        .await
                    {
                        Ok(saved) => {
                            update(|list| list.push(saved));
                            None
                        }
                        Err(message) => {
                            Some(untrack(|| t!("search-save-failed", reason = message)))
                        }
                    }
                }
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

/// The saved searches as chips, while nothing is searched.
#[component]
pub(super) fn SavedList() -> impl IntoView {
    move || {
        let list = current().filter(|l| !l.is_empty())?;
        let chips = list
            .into_iter()
            .map(|s| {
                let to = route::boorus(&s.query, &sites_param(&s.sites));
                view! {
                    <a class="chip" href=to title=search_name(&s.query, &s.sites)>
                        <Ico icon=Icon::Star size=16 />
                        <span class="saved-name">{s.name}</span>
                    </a>
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
