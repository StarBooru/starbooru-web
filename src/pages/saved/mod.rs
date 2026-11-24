//! Saved searches, the gallery's and the booru browser's, in one design: a
//! button beside a search saves it or forgets it, a row over the grid shows
//! the ones used last with the way to all of them, and a sheet filters,
//! sorts, renames, moves and removes them (`row.rs`). The search box offers
//! them while it's empty (`shell.rs`). Every change can be undone.

mod row;

use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{RemoteSite, SaveRemoteSearch, clean_saved_search};

pub use self::row::{SavedLine, SavedRow, SavedSheet};
use super::saved_search_error_text;
use crate::api;
use crate::components::icons::{Ico, Icon};
use crate::components::snackbar::{notify, notify_undo};
use crate::logic::boorus::{remote_error_text, search_name, sites_param};
use crate::logic::saved::{
    Item, Kind, Label, OFFERED, Rename, add, find, labels, name_for, remove, rename, sorted, tidy,
};
use crate::route;
use crate::store;

thread_local! {
    // Each kind's saved searches and whose they are, `None` until loaded,
    // and why they couldn't be. Reference counted, as they outlive any page.
    static LISTS: ArcRwSignal<Lists> = ArcRwSignal::new(Lists::default());
    // The saved search just moved, lit up where it landed.
    static MOVED: ArcRwSignal<Option<(Kind, i64)>> = ArcRwSignal::new(None);
    // The kind whose sheet is open.
    static SHEET: ArcRwSignal<Option<Kind>> = ArcRwSignal::new(None);
}

/// How long a moved search stays lit up, in milliseconds.
const MOVED_MS: u64 = 700;

#[derive(Clone, Debug, Default)]
struct Lists {
    gallery: Option<(String, Vec<Item>)>,
    boorus: Option<(String, Vec<Item>)>,
    failed: Vec<(String, Kind, String)>,
}

impl Lists {
    fn of(&self, kind: Kind) -> &Option<(String, Vec<Item>)> {
        match kind {
            Kind::Gallery => &self.gallery,
            Kind::Boorus => &self.boorus,
        }
    }

    fn of_mut(&mut self, kind: Kind) -> &mut Option<(String, Vec<Item>)> {
        match kind {
            Kind::Gallery => &mut self.gallery,
            Kind::Boorus => &mut self.boorus,
        }
    }
}

/// This account's saved searches of `kind`, `None` until they're known.
/// Read inside a view, it redraws when they change.
fn current(kind: Kind) -> Option<Vec<Item>> {
    let owner = store::owner();
    LISTS.with(|l| {
        l.with(|l| {
            l.of(kind)
                .as_ref()
                .filter(|(whose, _)| *whose == owner)
                .map(|(_, list)| list.clone())
        })
    })
}

/// Why this account's saved searches of `kind` couldn't be read.
fn failure(kind: Kind) -> Option<String> {
    let owner = store::owner();
    LISTS.with(|l| {
        l.with(|l| {
            l.failed
                .iter()
                .find(|(whose, k, _)| *whose == owner && *k == kind)
                .map(|(.., why)| why.clone())
        })
    })
}

/// The saved searches of `kind` in the order the user chose, with their
/// labels; `None` until they're known.
fn arranged(kind: Kind) -> Option<Vec<(Item, Label)>> {
    let saved = store::prefs().saved;
    let list = sorted(&current(kind)?, saved.sort, saved.of(kind));
    let labels = labels(&list);
    Some(list.into_iter().zip(labels).collect())
}

/// The saved searches of `kind` the search box offers while it's empty,
/// with how many there are in all. Loads them the first time.
pub fn offered(kind: Kind) -> (Vec<(Item, Label)>, usize) {
    let Some(list) = arranged(kind) else {
        load(kind);
        return (Vec::new(), 0);
    };
    let all = list.len();
    (list.into_iter().take(OFFERED).collect(), all)
}

/// The searches the gallery saved, `None` until they're known. The gallery
/// keeps their places (`logic::spots`).
pub fn queries() -> Option<Vec<String>> {
    untrack(|| current(Kind::Gallery)).map(|l| l.into_iter().map(|s| s.query).collect())
}

/// Opens the sheet with every saved search of `kind`.
pub fn open_sheet(kind: Kind) {
    SHEET.with(|s| s.set(Some(kind)));
}

fn close_sheet() {
    SHEET.with(|s| s.set(None));
}

/// Where saved search `item` leads: its gallery, or its sites.
pub fn href(item: &Item) -> String {
    match item.kind {
        Kind::Gallery => route::gallery(&item.query),
        Kind::Boorus => route::boorus(&item.query, &sites_param(&item.sites)),
    }
}

/// Saved search `item` was just opened: it's the one used last.
pub fn used(item: &Item) {
    let (kind, id) = (item.kind, item.id);
    let first = store::prefs_untracked().saved.of(kind).recent.first() == Some(&id);
    if !first {
        store::update_prefs(|p| p.saved.of_mut(kind).used(id));
    }
}

/// Search `q` on `sites` is open: if it's a saved one, it's the one used last.
pub fn seen(kind: Kind, q: &str, sites: &[RemoteSite]) {
    let list = untrack(|| current(kind)).unwrap_or_default();
    if let Some(item) = find(&list, q, sites).and_then(|id| list.iter().find(|s| s.id == id)) {
        used(item);
    }
}

/// Changes this account's list of `kind`, once it's known.
fn update(kind: Kind, change: impl FnOnce(&mut Vec<Item>)) {
    let owner = store::owner();
    LISTS.with(|l| {
        l.update(|l| {
            if let Some((whose, list)) = l.of_mut(kind).as_mut()
                && *whose == owner
            {
                change(list);
            }
        });
    });
}

/// Asks the server for the saved searches of `kind`; the ones kept from
/// before show meanwhile. A failure is kept, so the button can say why.
pub fn load(kind: Kind) {
    // A guest has none.
    if store::session_untracked().is_none() {
        return;
    }
    let owner = store::owner();
    LISTS.with(|l| {
        if l.with_untracked(|l| l.failed.iter().any(|(_, k, _)| *k == kind)) {
            l.update(|l| l.failed.retain(|(_, k, _)| *k != kind));
        }
    });
    spawn_local(async move {
        let answer: Result<Vec<Item>, String> = match kind {
            Kind::Gallery => api::call(|c| async move { c.saved_searches().await })
                .await
                .map(|l| l.into_iter().map(Item::from).collect()),
            Kind::Boorus => api::call(|c| async move { c.remote_searches().await })
                .await
                .map(|l| l.into_iter().map(Item::from).collect()),
        };
        if store::owner() != owner {
            return;
        }
        LISTS.with(|l| {
            l.update(|l| match answer {
                Ok(mut list) => {
                    list.sort_by_key(|s| s.name.to_lowercase());
                    *l.of_mut(kind) = Some((owner, list));
                }
                Err(why) => l.failed.push((owner, kind, why)),
            });
        });
    });
}

/// Saves search `query` (on `sites`, for the boorus) under `name` and puts
/// it in the list, or says why not.
async fn save(
    kind: Kind,
    name: String,
    query: String,
    sites: Vec<RemoteSite>,
) -> Result<Item, String> {
    let saved = match kind {
        Kind::Gallery => {
            let (name, query) = clean_saved_search(&name, &query)
                .map_err(|e| untrack(|| saved_search_error_text(&e)))?;
            api::call(move |c| async move { c.save_search(&name, &query).await })
                .await
                .map(Item::from)?
        }
        Kind::Boorus => {
            let search = SaveRemoteSearch { name, query, sites }
                .clean()
                .map_err(|e| untrack(|| remote_error_text(&e)))?;
            api::call(move |c| async move { c.save_remote_search(&search).await })
                .await
                .map(Item::from)?
        }
    };
    update(kind, |list| add(list, saved.clone()));
    Ok(saved)
}

/// Deletes saved search `item` and takes it off the list.
async fn delete(item: &Item) -> Result<(), String> {
    let (kind, id) = (item.kind, item.id);
    match kind {
        Kind::Gallery => api::call(move |c| async move { c.delete_search(id).await }).await?,
        Kind::Boorus => api::call(move |c| async move { c.delete_remote_search(id).await }).await?,
    }
    update(kind, |list| {
        remove(list, id);
    });
    Ok(())
}

/// The ids of the saved searches of `kind` as they're shown now.
fn shown_ids(kind: Kind) -> Vec<i64> {
    untrack(|| arranged(kind))
        .unwrap_or_default()
        .iter()
        .map(|(s, _)| s.id)
        .collect()
}

/// Puts the saved searches of `kind` in order `ids`, the user's own from
/// now on, with a way back to how they were sorted.
fn reorder(item: &Item, ids: Option<Vec<i64>>) {
    let Some(ids) = ids else {
        return;
    };
    let (kind, id) = (item.kind, item.id);
    let before = store::prefs_untracked().saved;
    store::update_prefs(|p| {
        p.saved.sort = crate::logic::saved::Sort::Mine;
        p.saved.of_mut(kind).order = ids;
    });
    show_moved(kind, id);
    notify_undo(
        untrack(|| t!("saved-moved", name = item.name.clone())),
        untrack(|| t!("press-undo")),
        move || {
            let before = before.clone();
            store::update_prefs(|p| p.saved = before);
            show_moved(kind, id);
        },
    );
}

/// Lights up saved search `id` for a moment where it landed.
fn show_moved(kind: Kind, id: i64) {
    MOVED.with(|m| m.set(Some((kind, id))));
    set_timeout(
        move || {
            MOVED.with(|m| {
                if m.get_untracked() == Some((kind, id)) {
                    m.set(None);
                }
            });
        },
        std::time::Duration::from_millis(MOVED_MS),
    );
}

/// Saves `item` again after it was removed, in its old places.
fn put_back(item: Item) {
    spawn_local(async move {
        let (old, kind) = (item.id, item.kind);
        match save(kind, item.name, item.query, item.sites).await {
            Ok(again) => store::update_prefs(|p| p.saved.of_mut(kind).renamed(old, again.id)),
            Err(message) => notify(untrack(|| t!("search-save-failed", reason = message))),
        }
    });
}

/// Takes `item` off the list, with a way to put it back.
fn forget(item: Item) {
    let (kind, id) = (item.kind, item.id);
    update(kind, |list| {
        remove(list, id);
    });
    spawn_local(async move {
        let gone = match kind {
            Kind::Gallery => api::call(move |c| async move { c.delete_search(id).await }).await,
            Kind::Boorus => {
                api::call(move |c| async move { c.delete_remote_search(id).await }).await
            }
        };
        match gone {
            Ok(()) => {
                let back = item.clone();
                notify_undo(
                    untrack(|| t!("saved-removed", name = item.name.clone())),
                    untrack(|| t!("press-undo")),
                    move || put_back(back.clone()),
                );
            }
            Err(message) => {
                update(kind, |list| add(list, item));
                notify(untrack(|| t!("search-forget-failed", reason = message)));
            }
        }
    });
}

/// Renames `item` to `typed`. The server has no rename: a copy goes in
/// under the new name and the old one goes, the copy keeping its places.
async fn renamed(item: Item, typed: String) -> Result<Option<Item>, String> {
    let Some(plan) = rename(item.kind, &item.name, &typed) else {
        return Ok(None);
    };
    let kind = item.kind;
    let copy = |item: &Item| save(kind, tidy(&typed), item.query.clone(), item.sites.clone());
    let new = match plan {
        Rename::SaveFirst => {
            let new = copy(&item).await?;
            // Both are there if this fails: nothing is lost.
            delete(&item).await?;
            new
        }
        Rename::RemoveFirst => {
            delete(&item).await?;
            match copy(&item).await {
                Ok(new) => new,
                Err(message) => {
                    put_back(item.clone());
                    return Err(message);
                }
            }
        }
    };
    store::update_prefs(|p| p.saved.of_mut(kind).renamed(item.id, new.id));
    Ok(Some(new))
}

/// Renames `item` and says so, with a way back to its old name. `then` gets
/// the id it has after.
fn rename_to(item: Item, typed: String, then: impl Fn(i64) + 'static) {
    spawn_local(async move {
        let (old, id) = (item.name.clone(), item.id);
        let result = renamed(item, typed).await;
        then(match &result {
            Ok(Some(new)) => new.id,
            _ => id,
        });
        match result {
            Ok(Some(new)) => notify_undo(
                untrack(|| t!("saved-renamed", name = new.name.clone())),
                untrack(|| t!("press-undo")),
                move || {
                    let (new, old) = (new.clone(), old.clone());
                    spawn_local(async move {
                        if let Err(message) = renamed(new, old).await {
                            notify(untrack(|| t!("saved-rename-failed", reason = message)));
                        }
                    });
                },
            ),
            Ok(None) => {}
            Err(message) => notify(untrack(|| t!("saved-rename-failed", reason = message))),
        }
    });
}

/// "Save search" beside the search's title: saves the search (on `sites`,
/// for the boorus), and once saved says so and forgets it on a press, with
/// Undo either way. While the list can't be read it says why, and a press
/// reads it again.
#[component]
pub fn SaveToggle(
    kind: Kind,
    #[prop(into)] q: Signal<String>,
    #[prop(into, optional)] sites: Signal<Vec<RemoteSite>>,
) -> impl IntoView {
    let busy = RwSignal::new(false);
    let list = Memo::new(move |_| current(kind));
    let existing = Memo::new(move |_| {
        list.with(|l| {
            let l = l.as_ref()?;
            q.with(|q| sites.with(|s| find(l, q, s)))
        })
    });
    let failed = Memo::new(move |_| list.with(Option::is_none).then(|| failure(kind)).flatten());
    let title = move || {
        if let Some(why) = failed.get() {
            t!("search-list-failed", reason = why)
        } else if existing.get().is_some() {
            t!("search-forget")
        } else {
            t!("search-save")
        }
    };
    let toggle = move |_| {
        if busy.get_untracked() {
            return;
        }
        if list.with_untracked(Option::is_none) {
            load(kind);
            return;
        }
        let (query, sites) = (tidy(&q.get_untracked()), sites.get_untracked());
        let existing = existing.get_untracked();
        let item = existing.and_then(|id| {
            list.with_untracked(|l| l.as_ref()?.iter().find(|s| s.id == id).cloned())
        });
        busy.set(true);
        spawn_local(async move {
            match item {
                Some(item) => forget(item),
                None => {
                    let name = match kind {
                        Kind::Gallery => name_for(&query),
                        Kind::Boorus => search_name(&query, &sites),
                    };
                    match save(kind, name, query, sites).await {
                        Ok(saved) => {
                            store::update_prefs(|p| p.saved.of_mut(kind).used(saved.id));
                            let gone = saved.clone();
                            notify_undo(
                                untrack(|| t!("saved-added", name = saved.name)),
                                untrack(|| t!("press-undo")),
                                move || {
                                    let gone = gone.clone();
                                    spawn_local(async move {
                                        if let Err(message) = delete(&gone).await {
                                            notify(untrack(|| {
                                                t!("search-forget-failed", reason = message)
                                            }));
                                        }
                                    });
                                },
                            );
                        }
                        Err(message) => {
                            notify(untrack(|| t!("search-save-failed", reason = message)));
                        }
                    }
                }
            }
            busy.try_set(false);
        });
    };
    let on = move || existing.get().is_some();
    view! {
        <button
            type="button"
            class="btn tonal save-search"
            class:on=on
            class:failed=move || failed.with(Option::is_some)
            title=title
            aria-pressed=move || on().to_string()
            disabled=move || busy.get() || (list.with(Option::is_none) && failed.with(Option::is_none))
            on:click=toggle
        >
            {move || {
                let icon = if on() { Icon::Bookmark } else { Icon::BookmarkAdd };
                view! { <Ico icon=icon /> }
            }}
            <span class="save-label">
                {move || if on() { t!("search-saved") } else { t!("search-save-short") }}
            </span>
        </button>
    }
}
