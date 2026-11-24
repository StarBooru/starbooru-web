//! Administration › Moderation, on a public server: the tags people voted
//! off posts, newest first, each with its post's picture, the score it
//! reached and a way to put it back, which also clears its votes. More come
//! as the list is read to its end.

use chrono::Utc;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use send_wrapper::SendWrapper;
use starbooru_core::TagRemoval;

use super::Waiting;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::components::snackbar::notify;
use crate::logic::library::ago;
use crate::logic::tag_votes::score_text;
use crate::{api, dom, instance, route};

/// How many a page asks for.
const PAGE: u32 = 30;

#[component]
pub(super) fn Moderation() -> impl IntoView {
    let public = Memo::new(|_| instance::public());
    view! {
        <Show
            when=move || public.get()
            fallback=|| {
                view! {
                    <div class="card empty">
                        <Ico icon=Icon::Shield size=48 />
                        <p class="muted">{move || t!("admin-moderation-private")}</p>
                    </div>
                }
            }
        >
            <VotedOff />
        </Show>
    }
}

#[component]
fn VotedOff() -> impl IntoView {
    let list = RwSignal::new(Vec::<TagRemoval>::new());
    let page = RwSignal::new(0_u32);
    let more = RwSignal::new(true);
    let loading = RwSignal::new(false);
    let first = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let load_more = move || {
        if loading.get_untracked() || !more.get_untracked() {
            return;
        }
        let next = page.get_untracked().saturating_add(1);
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            let result = api::call(move |c| async move { c.tag_removals(next, PAGE).await }).await;
            loading.try_set(false);
            match result {
                Ok(found) => {
                    list.update(|l| {
                        for row in found.removals {
                            if !l.iter().any(|r| r.id == row.id) {
                                l.push(row);
                            }
                        }
                    });
                    page.set(found.page.max(next));
                    more.set(found.has_more);
                    first.set(false);
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };
    load_more();
    let sentinel: NodeRef<html::Div> = NodeRef::new();
    let watch = StoredValue::new(None::<SendWrapper<dom::Watch>>);
    sentinel.on_load(move |el| {
        let near = dom::on_near(&el, "0px 0px 120% 0px", load_more);
        watch.set_value(Some(SendWrapper::new(near)));
    });
    on_cleanup(move || {
        if let Some(near) = watch.try_update_value(Option::take).flatten() {
            near.take().stop();
        }
    });
    let retry = Callback::new(move |()| load_more());
    let keys = Memo::new(move |_| list.with(|l| l.iter().map(|r| r.id).collect::<Vec<_>>()));
    view! {
        <section class="card">
            <h2 class="card-title">{move || t!("admin-voted-off")}</h2>
            <p class="muted hint">{move || t!("admin-voted-off-lead")}</p>
            {move || error.get().map(|m| view! { <ErrorBanner message=m on_retry=retry /> })}
            <Show when=move || !first.get() || error.with(Option::is_some) fallback=|| view! { <Waiting /> }>
                <Show when=move || !first.get() && list.with(Vec::is_empty)>
                    <div class="voted-none">
                        <Ico icon=Icon::Check size=32 />
                        <p class="muted">{move || t!("admin-voted-off-none")}</p>
                    </div>
                </Show>
                <ul class="voted-off">
                    <For
                        each=move || keys.get()
                        key=|k| *k
                        children=move |id| {
                            let row = list.with_untracked(|l| l.iter().find(|r| r.id == id).cloned());
                            row.map(|row| view! { <VotedOffRow row=row list=list /> })
                        }
                    />
                </ul>
            </Show>
            <div class="sentinel" class:done=move || !more.get() node_ref=sentinel>
                <Show when=move || loading.get() && !first.get()>
                    <Spinner />
                </Show>
            </div>
        </section>
    }
}

/// One tag off one post.
#[component]
fn VotedOffRow(row: TagRemoval, list: RwSignal<Vec<TagRemoval>>) -> impl IntoView {
    let busy = RwSignal::new(false);
    let (id, post, tag) = (row.id, row.post_id, row.tag.clone());
    let restore = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        let tag = tag.clone();
        spawn_local(async move {
            let result = api::call(move |c| async move { c.undo_tag_removal(id).await }).await;
            busy.try_set(false);
            match result {
                Ok(_) => {
                    list.try_update(|l| l.retain(|r| r.id != id));
                    notify(untrack(|| {
                        t!("admin-voted-off-back", tag = tag.clone(), id = post)
                    }));
                }
                Err(message) => notify(message),
            }
        });
    };
    let thumb = row.thumb_url.as_deref().map(api::media);
    let when = row.removed_at;
    let shown_tag = row.tag.replace('_', " ");
    view! {
        <li class="voted-row">
            <a class="voted-thumb" href=route::post(row.post_id, "", None) aria-label=move || t!("mention-post", id = post)>
                {match thumb {
                    Some(src) => view! { <img src=src alt="" loading="lazy" decoding="async" /> }.into_any(),
                    None => view! { <Ico icon=Icon::Label /> }.into_any(),
                }}
            </a>
            <div class="voted-text">
                <span class="chip voted-tag">
                    <span class="break">{shown_tag}</span>
                </span>
                <span class="muted">
                    {move || t!("mention-post", id = post)}
                    " · "
                    <span class="voted-score">{score_text(row.score)}</span>
                    " · "
                    {move || ago(when, Utc::now())}
                </span>
            </div>
            <button type="button" class="btn tonal small" disabled=move || busy.get() on:click=restore>
                <Ico icon=Icon::Undo size=18 />
                {move || t!("admin-voted-off-restore")}
            </button>
        </li>
    }
}
