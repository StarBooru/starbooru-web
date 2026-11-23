//! The duplicate finder at `/duplicates`: groups of posts that look alike,
//! closest first. In each, keep the best copy, taking the others' tags if
//! wanted, and the rest go to the recycle bin with Undo; or mark the group
//! as not duplicates.

use chrono::Local;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::{
    DEFAULT_DUPLICATE_DISTANCE, DuplicateGroup, MAX_DUPLICATE_DISTANCE, MediaKind, Post,
    PostUpdate, ResolveDuplicates,
};

use super::{gallery, post, trash};
use crate::api;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::components::snackbar::{notify, notify_undo};
use crate::logic::bulk::worth_telling;
use crate::logic::duplicates::{
    LEVELS, Level, Likeness, PAGE, Summary, day, group_key, has_more, level, likeness, more,
    resolve_request, size_of, suggested_keeper, summary, unmerge,
};
use crate::logic::gallery::{format_duration, tint};
use crate::logic::viewer::human_size;
use crate::route;

thread_local! {
    // Reference counted: the notice's Undo, which bumps it, outlives the page.
    static CHANGED: ArcRwSignal<u32> = ArcRwSignal::new(0);
}

/// A resolve was undone: a finder on the page asks for its groups again.
fn changed() {
    CHANGED.with(|c| c.update(|n| *n = n.wrapping_add(1)));
}

fn level_label(distance: u32) -> String {
    match level(distance) {
        Level::Strict => t!("dup-strict"),
        Level::Normal => t!("dup-normal"),
        Level::Loose => t!("dup-loose"),
    }
}

fn likeness_label(distance: u32) -> String {
    match likeness(distance) {
        Likeness::Identical => t!("dup-identical"),
        Likeness::NearlyIdentical => t!("dup-nearly-identical"),
        Likeness::VerySimilar => t!("dup-very-similar"),
        Likeness::Similar => t!("dup-similar"),
    }
}

fn summary_text(s: Summary) -> String {
    match s {
        Summary::All(count) => t!("posts-count", count = count),
        Summary::Some { shown, size } => t!("dup-some-of", shown = shown, count = size),
    }
}

fn groups_text(total: u64) -> String {
    t!("dup-groups", count = total)
}

#[component]
pub fn Duplicates() -> impl IntoView {
    let distance = RwSignal::new(DEFAULT_DUPLICATE_DISTANCE);
    let limit = RwSignal::new(PAGE);
    let groups = RwSignal::new(Vec::<DuplicateGroup>::new());
    let total = RwSignal::new(0_u64);
    let unhashed = RwSignal::new(0_i64);
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let retries = RwSignal::new(0_u32);
    // Only the latest answer counts, when the level changes mid-request.
    let asked = StoredValue::new(0_u64);
    let undone = CHANGED.with(Clone::clone);

    Effect::new(move |_| {
        let (d, l) = (distance.get(), limit.get());
        retries.track();
        undone.track();
        asked.update_value(|n| *n = n.wrapping_add(1));
        let at = asked.get_value();
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            let result = api::call(move |c| async move { c.duplicates(d, l).await }).await;
            // A newer request went out, or the page has gone.
            if asked.try_get_value() != Some(at) {
                return;
            }
            match result {
                Ok(found) => {
                    total.set(found.total);
                    unhashed.set(found.unhashed);
                    groups.set(found.groups);
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    });

    let retry = Callback::new(move |()| retries.update(|n| *n = n.wrapping_add(1)));
    // A resolved or dismissed group leaves the list and the count.
    let done = Callback::new(move |key: i64| {
        groups.update(|g| g.retain(|group| group_key(group) != Some(key)));
        total.update(|t| *t = t.saturating_sub(1));
        // The page ran out, but the server has more.
        if groups.with_untracked(Vec::is_empty) && total.get_untracked() > 0 {
            retry.run(());
        }
    });
    let empty = Memo::new(move |_| groups.with(Vec::is_empty));
    let more_left = Memo::new(move |_| has_more(groups.with(Vec::len), total.get(), limit.get()));

    let levels = LEVELS
        .into_iter()
        .map(|d| {
            let on = move || distance.get() == d;
            view! {
                <button
                    type="button"
                    role="radio"
                    class="seg"
                    class:on=on
                    aria-checked=move || if on() { "true" } else { "false" }
                    on:click=move |_| {
                        if distance.get_untracked() != d {
                            // The old level's groups go, so the wait shows.
                            groups.set(Vec::new());
                            limit.set(PAGE);
                            distance.set(d);
                        }
                    }
                >
                    {move || level_label(d)}
                </button>
            }
        })
        .collect_view();

    view! {
        <div class="page duplicates">
            <div class="page-head">
                <h1 class="page-title">{move || t!("dup-title")}</h1>
                <div class="segmented" role="radiogroup" aria-label=move || t!("dup-matching")>
                    {levels}
                </div>
            </div>
            <p class="muted lead">{move || t!("dup-lead")}</p>
            <Show when=move || { unhashed.get() > 0 }>
                <div class="banner info" role="status">
                    <span class="break">{move || t!("dup-unhashed", count = unhashed.get())}</span>
                </div>
            </Show>
            {move || error.get().map(|message| view! { <ErrorBanner message=message on_retry=retry /> })}
            <Show when=move || loading.get() && empty.get()>
                <div class="center-page">
                    <Spinner />
                </div>
            </Show>
            <Show when=move || !loading.get() && empty.get() && error.with(Option::is_none)>
                <div class="card empty">
                    <Ico icon=Icon::Copy size=48 />
                    <h2>{move || t!("dup-none")}</h2>
                    <p class="muted">
                        {move || {
                            if distance.get() < MAX_DUPLICATE_DISTANCE {
                                t!("dup-none-try-loose")
                            } else {
                                t!("dup-none-loose")
                            }
                        }}
                    </p>
                </div>
            </Show>
            <Show when=move || !empty.get()>
                <p class="dup-count muted">{move || groups_text(total.get())}</p>
                <div class="dup-groups">
                    // A group that changed between answers is drawn again.
                    <For
                        each=move || groups.get()
                        key=|g| (g.distance, g.posts.iter().map(|p| p.id).collect::<Vec<_>>())
                        children=move |group| view! { <DupGroup group=group on_done=done /> }
                    />
                </div>
                <Show when=move || more_left.get()>
                    <div class="sentinel">
                        {move || {
                            if loading.get() {
                                view! { <Spinner /> }.into_any()
                            } else {
                                view! {
                                    <button
                                        type="button"
                                        class="btn tonal"
                                        on:click=move |_| limit.update(|l| *l = more(*l))
                                    >
                                        {move || t!("load-more")}
                                    </button>
                                }
                                    .into_any()
                            }
                        }}
                    </div>
                </Show>
            </Show>
        </div>
    }
}

/// Posts in `req` changed: the copies kept of them, and the lists and tag
/// counts they were in, are out of date.
fn resolved(req: &ResolveDuplicates) {
    let mut ids = req.remove.clone();
    ids.push(req.keep);
    post::forget_kept(&ids);
    api::forget_tags();
    gallery::forget_lists();
}

/// Undo, from the notice: the binned copies come back, and the kept post
/// gives back the tags and source it took from them.
fn undo_resolve(removed: Vec<i64>, revert: Option<(i64, PostUpdate)>) {
    spawn_local(async move {
        let trash::Restored { failed, .. } = trash::restore_posts(removed).await;
        let telling: Vec<_> = failed.iter().filter(|r| worth_telling(r.status)).collect();
        if let Some(first) = telling.first() {
            notify(t!(
                "bulk-restore-failed",
                count = telling.len(),
                reason = first.message.clone()
            ));
        }
        if let Some((id, update)) = revert {
            match api::call(move |c| async move { c.update_post(id, &update).await }).await {
                Ok(_) => {
                    post::forget_kept(&[id]);
                    gallery::forget_lists();
                }
                Err(message) => notify(t!("dup-unmerge-failed", id = id, reason = message)),
            }
        }
        // The tags' counts changed either way.
        api::forget_tags();
        changed();
    });
}

/// One group: its copies side by side, the one to keep picked, and what to
/// do with the rest.
#[component]
fn DupGroup(group: DuplicateGroup, on_done: Callback<i64>) -> impl IntoView {
    let key = group_key(&group).unwrap_or_default();
    let ids: Vec<i64> = group.posts.iter().map(|p| p.id).collect();
    let others = ids.len().saturating_sub(1);
    let keep = RwSignal::new(suggested_keeper(&group.posts).unwrap_or(key));
    let merge = RwSignal::new(true);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let listed = StoredValue::new(group.posts.clone());

    let resolve = {
        let ids = ids.clone();
        move |_| {
            if busy.get_untracked() {
                return;
            }
            let Some(req) = resolve_request(&ids, keep.get_untracked(), merge.get_untracked())
            else {
                return;
            };
            // As it was, for Undo to put back.
            let before = listed
                .with_value(|posts| posts.iter().find(|p| p.id == req.keep).cloned())
                .filter(|_| req.merge_tags);
            busy.set(true);
            error.set(None);
            spawn_local(async move {
                let sent = req.clone();
                let result =
                    api::call(move |c| async move { c.resolve_duplicates(&sent).await }).await;
                // Settled first: `on_done` takes this group off the page.
                busy.try_set(false);
                match result {
                    Ok(kept) => {
                        resolved(&req);
                        let revert = before
                            .as_ref()
                            .and_then(|b| unmerge(b, &kept))
                            .map(|update| (req.keep, update));
                        let removed = req.remove;
                        on_done.try_run(key);
                        notify_undo(
                            t!("bulk-moved", count = removed.len()),
                            t!("press-undo"),
                            move || undo_resolve(removed.clone(), revert.clone()),
                        );
                    }
                    Err(message) => {
                        error.try_set(Some(message));
                    }
                }
            });
        }
    };
    let dismiss = move |_| {
        if busy.get_untracked() {
            return;
        }
        let ids = ids.clone();
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let result = api::call(move |c| async move { c.dismiss_duplicates(ids).await }).await;
            busy.try_set(false);
            match result {
                Ok(()) => {
                    on_done.try_run(key);
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };

    let head = summary_text(summary(&group));
    let distance = group.distance;
    let items = group
        .posts
        .into_iter()
        .enumerate()
        .map(|(index, p)| view! { <DupItem post=p index=index group=key keep=keep /> })
        .collect_view();
    view! {
        <section class="card dup-group">
            <div class="dup-head">
                <h2 class="card-title">{move || likeness_label(distance)}</h2>
                <span class="muted">{head}</span>
            </div>
            <div class="dup-items" role="radiogroup" aria-label=move || t!("dup-keep-which")>
                {items}
            </div>
            {move || {
                error.get().map(|message| view! { <p class="field-error" role="alert">{message}</p> })
            }}
            <div class="dup-actions">
                <label class="check">
                    <input
                        type="checkbox"
                        prop:checked=move || merge.get()
                        on:change=move |e| merge.set(event_target_checked(&e))
                    />
                    <span>{move || t!("dup-merge")}</span>
                </label>
                <div class="dup-buttons">
                    <button
                        type="button"
                        class="btn text"
                        disabled=move || busy.get()
                        on:click=dismiss
                    >
                        {move || t!("dup-dismiss")}
                    </button>
                    <button
                        type="button"
                        class="btn"
                        disabled=move || busy.get()
                        on:click=resolve
                    >
                        <Ico icon=Icon::Delete />
                        {move || t!("dup-resolve", count = others)}
                    </button>
                </div>
            </div>
        </section>
    }
}

/// A copy in a group: its thumbnail, which picks it as the one to keep, and
/// its shape, size, tags, day and a link to it.
#[component]
fn DupItem(post: Post, index: usize, group: i64, keep: RwSignal<i64>) -> impl IntoView {
    let id = post.id;
    let chosen = move || keep.get() == id;
    let src = post
        .thumb_url
        .as_deref()
        .and_then(|u| api::client().map(|c| c.absolute(u)));
    let loaded = RwSignal::new(false);
    let mut style = format!("--delay: {}ms", index.min(12) * 16);
    if let Some(colour) = tint(&post.palette) {
        style.push_str(&format!("; --tint: {colour}"));
    }
    let picture = match src {
        Some(src) => view! {
            <img
                src=src
                alt=move || t!("post-alt", id = id)
                loading="lazy"
                decoding="async"
                on:load=move |_| loaded.set(true)
            />
        }
        .into_any(),
        None => view! {
            <span class="thumb-placeholder">
                <Ico icon=Icon::Play size=40 />
            </span>
        }
        .into_any(),
    };
    let badge = match post.kind {
        MediaKind::Video => {
            let length = post.duration_ms.and_then(format_duration);
            Some(
                view! {
                    <span class="badge">
                        <Ico icon=Icon::Play size=16 />
                        {length}
                    </span>
                }
                .into_any(),
            )
        }
        MediaKind::Animation => {
            Some(view! { <span class="badge">{move || t!("badge-gif")}</span> }.into_any())
        }
        MediaKind::Image => None,
    };
    let shape = size_of(&post);
    let size = human_size(post.file_size);
    let tags = post.tags.len();
    let added = day(&post.created_at, &Local);
    view! {
        <div class="dup-item" class:on=chosen>
            <label class="dup-pick">
                <input
                    type="radio"
                    class="visually-hidden"
                    name=format!("keep-{group}")
                    prop:checked=chosen
                    on:change=move |_| keep.set(id)
                />
                <span class="thumb" class:loaded=loaded style=style>
                    {picture}
                    {badge}
                    <Show when=chosen>
                        <span class="keep-badge">
                            <Ico icon=Icon::Check size=16 />
                            {move || t!("dup-keep")}
                        </span>
                    </Show>
                </span>
            </label>
            <div class="dup-meta">
                <span class="dup-dims">
                    {move || shape.clone().unwrap_or_else(|| t!("dup-unknown-size"))}
                </span>
                <span class="muted">
                    {move || format!("{size} · {}", t!("tags-count", count = tags))}
                </span>
                <span class="muted">{move || t!("dup-added", date = added.clone())}</span>
                <a
                    class="dup-open"
                    href=route::post(id, "", None)
                    title=move || t!("upload-open-post", id = id)
                >
                    {move || t!("dup-post", id = id)}
                </a>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_and_levels_read_as_the_reference_did() {
        assert_eq!(likeness_label(0), "Look identical");
        assert_eq!(likeness_label(2), "Nearly identical");
        assert_eq!(likeness_label(6), "Very similar");
        assert_eq!(likeness_label(10), "Similar");
        assert_eq!(groups_text(1), "1 group");
        assert_eq!(groups_text(12), "12 groups");
        assert_eq!(level_label(LEVELS[0]), "Strict");
        assert_eq!(level_label(LEVELS[1]), "Normal");
        assert_eq!(level_label(LEVELS[2]), "Loose");
        assert_eq!(summary_text(Summary::All(1)), "1 post");
        assert_eq!(summary_text(Summary::All(3)), "3 posts");
        assert_eq!(
            summary_text(Summary::Some {
                shown: 50,
                size: 64
            }),
            "50 of 64 posts"
        );
        assert_eq!(t!("dup-resolve", count = 1), "Keep one, bin the other");
        assert_eq!(t!("dup-resolve", count = 2), "Keep one, bin 2 others");
    }

    #[test]
    fn the_finder_keeps_to_the_tokens_and_the_reference_layout() {
        let css = include_str!("../../style/pages/duplicates.css");
        assert!(!css.contains("backdrop-filter"));
        assert!(css.contains("grid-template-columns: repeat(auto-fill, minmax(168px, 1fr))"));
        assert!(css.contains(".dup-item.on .dup-pick .thumb { outline-color: var(--primary); }"));
        assert!(css.contains("@media (prefers-reduced-motion: reduce)"));
        // Linked from Settings' Library card, beside the recycle bin.
        let settings = include_str!("soon.rs");
        let trash = settings.find("href=route::TRASH");
        let here = settings.find("href=route::DUPLICATES");
        assert!(trash.zip(here).is_some_and(|(t, d)| t < d));
    }
}
