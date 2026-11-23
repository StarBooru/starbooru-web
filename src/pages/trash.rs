//! The recycle bin at `/trash`: the posts moved there, a page at a time, each
//! to restore or delete for good, and the whole bin to restore or empty.
//! Also the restore that the viewer, the menus' Undo and mass edits share, so
//! the kept posts and lists agree whichever of them ran.

use std::collections::HashSet;
use std::sync::Arc;

use futures_util::future::join_all;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use send_wrapper::SendWrapper;
use starbooru_core::{Post, Role};

use super::{gallery, post};
use crate::api::{self, Refused};
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::confirm::ConfirmButton;
use crate::components::icons::{Ico, Icon};
use crate::components::snackbar::notify;
use crate::components::thumb::Thumb;
use crate::dom;
use crate::logic::bulk::{
    BIN_PAGE, BIN_SWEEP, BIN_SWEEPS, PurgeNote, RESTORES_AT_ONCE, bin_page, purge_note,
    worth_telling,
};
use crate::logic::layout::Place;
use crate::route;
use crate::store;

/// The search that lists the bin.
const BIN: &str = "status:deleted";

/// What came back from the bin, and what didn't, with why.
pub struct Restored {
    pub back: Vec<Arc<Post>>,
    pub failed: Vec<Refused>,
}

/// Brings posts `ids` back from the recycle bin, a few at a time. Updates the
/// viewer's kept copies, and drops the kept lists once at the end.
pub async fn restore_posts(ids: Vec<i64>) -> Restored {
    let mut back = Vec::new();
    let mut failed = Vec::new();
    for chunk in ids.chunks(RESTORES_AT_ONCE) {
        let asks = chunk
            .iter()
            .map(|&id| api::attempt(move |c| async move { c.restore_post(id).await }));
        for result in join_all(asks).await {
            match result {
                Ok(p) => {
                    let p = Arc::new(p);
                    post::restored(&p);
                    back.push(p);
                }
                Err(refused) => failed.push(refused),
            }
        }
    }
    if !back.is_empty() {
        gallery::forget_lists();
    }
    Restored { back, failed }
}

/// Undo, after posts `ids` went to the bin: from the notice, which outlives
/// the page.
pub fn undo_delete(ids: Vec<i64>) {
    spawn_local(async move {
        let Restored { failed, .. } = restore_posts(ids).await;
        let telling: Vec<&Refused> = failed.iter().filter(|r| worth_telling(r.status)).collect();
        if let Some(first) = telling.first() {
            notify(t!(
                "bulk-restore-failed",
                count = telling.len(),
                reason = first.message.clone()
            ));
        }
    });
}

/// Deletes post `id`, which is in the bin, for good.
async fn purge(id: i64) -> Result<(), String> {
    api::call(move |c| async move { c.purge_post(id).await }).await?;
    post::forget_kept(&[id]);
    gallery::forget_post(id);
    Ok(())
}

/// Every post in the bin, read before any leaves it, so that the pages
/// don't shift under the reading.
async fn whole_bin() -> Result<Vec<i64>, String> {
    let mut ids = Vec::new();
    for page in 1..=BIN_SWEEPS {
        let found =
            api::call(move |c| async move { c.search_posts(BIN, None, page, BIN_SWEEP).await })
                .await?;
        ids.extend(found.posts.iter().map(|p| p.id));
        if !found.has_more {
            break;
        }
    }
    Ok(ids)
}

/// The step for the whole bin that waits for a second press.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Whole {
    Restore,
    Empty,
}

#[component]
pub fn Trash() -> impl IntoView {
    let posts = RwSignal::new(Vec::<Arc<Post>>::new());
    let has_more = RwSignal::new(true);
    let loading = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    // What the last step for the whole bin did, or why it failed.
    let notice = RwSignal::new(None::<String>);
    let problem = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let confirm = RwSignal::new(None::<Whole>);
    // Posts with a restore or a purge on its way.
    let working = RwSignal::new(HashSet::<i64>::new());
    let admin = store::session_untracked().is_some_and(|s| s.user.role == Role::Admin);
    // Only administrators may read the server's settings.
    let days = LocalResource::new(move || async move {
        if !admin {
            return None;
        }
        api::call(|c| async move { c.admin_settings().await })
            .await
            .ok()
            .map(|s| s.trash_days)
    });

    // Bumped when the list starts again, so a page asked for before is dropped.
    let generation = StoredValue::new(0_u32);
    let load_more = move || {
        if loading.get_untracked()
            || !has_more.get_untracked()
            || error.with_untracked(Option::is_some)
        {
            return;
        }
        // Counted from what's listed: a post restored or purged here moves
        // the rest of the bin up a place.
        let page = bin_page(posts.with_untracked(Vec::len));
        let at_start = generation.get_value();
        loading.set(true);
        spawn_local(async move {
            let result =
                api::call(move |c| async move { c.search_posts(BIN, None, page, BIN_PAGE).await })
                    .await;
            // The page may have gone meanwhile, or started the list again.
            if generation.try_get_value() != Some(at_start) {
                return;
            }
            loading.set(false);
            match result {
                Ok(found) => {
                    posts.update(|list| {
                        let seen: HashSet<i64> = list.iter().map(|p| p.id).collect();
                        let new = found.posts.into_iter().filter(|p| !seen.contains(&p.id));
                        list.extend(new.map(Arc::new));
                    });
                    has_more.set(found.has_more);
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };
    // From the top again, after a step that changed the whole bin.
    let reload = move || {
        generation.update_value(|g| *g = g.wrapping_add(1));
        loading.set(false);
        posts.set(Vec::new());
        has_more.set(true);
        error.set(None);
        load_more();
    };
    load_more();

    let sentinel: NodeRef<html::Div> = NodeRef::new();
    let watch = StoredValue::new(None::<SendWrapper<dom::Watch>>);
    sentinel.on_load(move |el| {
        let near = dom::on_near(&el, "0px 0px 150% 0px", load_more);
        watch.set_value(Some(SendWrapper::new(near)));
    });
    on_cleanup(move || {
        if let Some(near) = watch.try_update_value(Option::take).flatten() {
            near.take().stop();
        }
    });

    let restore_one = Callback::new(move |id: i64| {
        working.update(|w| {
            w.insert(id);
        });
        spawn_local(async move {
            let Restored { back, failed } = restore_posts(vec![id]).await;
            working.try_update(|w| w.remove(&id));
            if back.is_empty() {
                let reason = failed
                    .first()
                    .map(|f| f.message.clone())
                    .unwrap_or_default();
                notify(t!("trash-restore-failed", id = id, reason = reason));
                return;
            }
            posts.try_update(|list| list.retain(|p| p.id != id));
            notify(t!("trash-restored", count = 1));
        });
    });
    let purge_one = Callback::new(move |id: i64| {
        working.update(|w| {
            w.insert(id);
        });
        spawn_local(async move {
            let result = purge(id).await;
            working.try_update(|w| w.remove(&id));
            match result {
                Ok(()) => {
                    posts.try_update(|list| list.retain(|p| p.id != id));
                    notify(t!("trash-purged", count = 1));
                }
                Err(message) => notify(t!("trash-purge-failed", id = id, reason = message)),
            }
        });
    });

    // A step for the whole bin: the first press asks, the second does it.
    let press = move |step: Whole| {
        if busy.get_untracked() {
            return;
        }
        if confirm.get_untracked() != Some(step) {
            confirm.set(Some(step));
            return;
        }
        confirm.set(None);
        busy.set(true);
        notice.set(None);
        problem.set(None);
        let listed: Vec<i64> = posts.with_untracked(|l| l.iter().map(|p| p.id).collect());
        spawn_local(async move {
            let outcome = match step {
                Whole::Restore => restore_all().await,
                Whole::Empty => empty_all(listed).await,
            };
            if busy.try_set(false).is_some() {
                return;
            }
            match outcome {
                Ok(message) => notice.set(Some(message)),
                Err(message) => problem.set(Some(message)),
            }
            reload();
        });
    };

    let open = Callback::new(|p: Arc<Post>| post::opening(p, BIN.to_owned()));
    let retry = Callback::new(move |()| {
        error.set(None);
        load_more();
    });
    let empty = move || {
        posts.with(Vec::is_empty)
            && !loading.get()
            && !has_more.get()
            && error.with(Option::is_none)
    };
    let note = move || {
        let text = match purge_note(days.get().flatten())? {
            PurgeNote::Never => t!("trash-purge-never"),
            PurgeNote::After(days) => t!("trash-purge-after", days = days),
        };
        Some(view! { <p class="muted lead trash-note">{text}</p> })
    };
    let asking = move |step: Whole| confirm.get() == Some(step);
    let empty_button = move || {
        admin.then(|| view! {
            <button
                type="button"
                class=move || if asking(Whole::Empty) { "btn danger" } else { "btn tonal" }
                disabled=move || busy.get()
                on:click=move |_| press(Whole::Empty)
                on:blur=move |_| confirm.set(None)
            >
                <Ico icon=Icon::DeleteForever />
                {move || {
                    if asking(Whole::Empty) { t!("trash-empty-confirm") } else { t!("trash-empty") }
                }}
            </button>
        })
    };

    view! {
        <div class="page trash">
            <div class="page-head">
                <h1 class="page-title">{move || t!("trash-title")}</h1>
                <Show when=move || !posts.with(Vec::is_empty)>
                    <div class="page-tools">
                        <button
                            type="button"
                            class=move || if asking(Whole::Restore) { "btn" } else { "btn tonal" }
                            disabled=move || busy.get()
                            on:click=move |_| press(Whole::Restore)
                            on:blur=move |_| confirm.set(None)
                        >
                            <Ico icon=Icon::Restore />
                            {move || {
                                if asking(Whole::Restore) {
                                    t!("trash-restore-all-confirm")
                                } else {
                                    t!("trash-restore-all")
                                }
                            }}
                        </button>
                        {empty_button}
                        <Show when=move || busy.get()>
                            <Spinner />
                        </Show>
                    </div>
                </Show>
            </div>
            <p class="muted lead">{move || t!("trash-lead")}</p>
            {note}
            {move || {
                notice
                    .get()
                    .map(|message| {
                        view! {
                            <div class="banner info" role="status">
                                <span class="break">{message}</span>
                            </div>
                        }
                    })
            }}
            {move || problem.get().map(|message| view! { <ErrorBanner message=message /> })}
            {move || error.get().map(|message| view! { <ErrorBanner message=message on_retry=retry /> })}
            <Show when=empty>
                <div class="card empty">
                    <Ico icon=Icon::Delete size=48 />
                    <h2>{move || t!("trash-none")}</h2>
                    <a class="btn tonal" href="/">
                        {move || t!("notfound-back")}
                    </a>
                </div>
            </Show>
            <div class="trash-grid">
                <For
                    each=move || posts.get().into_iter().enumerate()
                    key=|(_, p)| p.id
                    children=move |(index, p)| {
                        let id = p.id;
                        let busy = Signal::derive(move || working.with(|w| w.contains(&id)));
                        view! {
                            <TrashItem
                                post=p
                                index=index
                                busy=busy
                                on_open=open
                                on_restore=restore_one
                                on_purge=purge_one
                            />
                        }
                    }
                />
            </div>
            <div
                class="sentinel"
                class:done=move || !has_more.get() || error.with(Option::is_some)
                node_ref=sentinel
            >
                {move || {
                    if loading.get() {
                        view! { <Spinner /> }.into_any()
                    } else {
                        view! {
                            <button type="button" class="btn tonal" on:click=move |_| load_more()>
                                {move || t!("load-more")}
                            </button>
                        }
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

/// Restores everything in the bin, and says how that went.
async fn restore_all() -> Result<String, String> {
    let ids = whole_bin().await?;
    let Restored { back, failed } = restore_posts(ids).await;
    let mut message = t!("trash-restored", count = back.len());
    if let Some(first) = failed.first() {
        message.push(' ');
        message.push_str(&t!(
            "bulk-restore-failed",
            count = failed.len(),
            reason = first.message.clone()
        ));
    }
    Ok(message)
}

/// Empties the bin from another page. What's in it is read first, so that
/// no page keeps showing those posts.
pub(crate) async fn empty_whole_bin() -> Result<String, String> {
    let listed = whole_bin().await.unwrap_or_default();
    empty_all(listed).await
}

/// Empties the bin, which held at least `listed`.
async fn empty_all(listed: Vec<i64>) -> Result<String, String> {
    let done = api::call(|c| async move { c.empty_trash().await }).await?;
    post::forget_kept(&listed);
    // A gallery searching the bin itself.
    gallery::forget_lists();
    Ok(t!("trash-purged", count = done.purged))
}

/// A post in the bin: its tile, which opens it, and the buttons that bring it
/// back or delete it for good. The second asks first.
#[component]
fn TrashItem(
    post: Arc<Post>,
    index: usize,
    #[prop(into)] busy: Signal<bool>,
    on_open: Callback<Arc<Post>>,
    on_restore: Callback<i64>,
    on_purge: Callback<i64>,
) -> impl IntoView {
    let id = post.id;
    let href = route::post(id, BIN, None);
    let place: Option<Place> = None;
    view! {
        <div class="trash-item">
            <Thumb post=post index=index href=href place=place still=false on_open=on_open />
            <div class="trash-actions">
                <button
                    type="button"
                    class="icon-btn"
                    title=move || t!("trash-restore")
                    aria-label=move || t!("trash-restore-post", id = id)
                    disabled=move || busy.get()
                    on:click=move |_| on_restore.run(id)
                >
                    <Ico icon=Icon::Restore />
                </button>
                <ConfirmButton
                    icon=Icon::DeleteForever
                    title=Signal::derive(|| t!("trash-purge"))
                    label=Signal::derive(move || t!("trash-purge-post", id = id))
                    disabled=busy
                    on_confirm=Callback::new(move |()| on_purge.run(id))
                />
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_bin_reads_as_the_reference_did() {
        let css = include_str!("../../style/pages/trash.css");
        assert!(css.contains(".trash-item .thumb { filter: saturate(0.6); }"));
        assert!(!css.contains("backdrop-filter"));
        // Deleting for good always asks first; restoring and emptying the
        // whole bin too.
        let source = include_str!("trash.rs");
        assert!(source.contains("<ConfirmButton"));
        for id in ["trash-empty-confirm", "trash-restore-all-confirm"] {
            assert!(source.contains(&format!("t!(\"{id}\")")), "{id}");
        }
    }

    #[test]
    fn the_viewer_and_the_bin_restore_the_same_way() {
        let viewer = include_str!("post/mod.rs");
        assert!(viewer.contains("trash::restore_posts(vec![id])"));
        assert!(!viewer.contains("c.restore_post("));
        let menu = include_str!("post_menu.rs");
        assert!(menu.contains("trash::undo_delete(vec![id])"));
        assert!(!include_str!("gallery.rs").contains("c.restore_post("));
    }
}
