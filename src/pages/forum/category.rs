//! `/forum/c/{id}`: a category's threads, pinned ones first, then the
//! latest written in, more coming in as the list nears its end; with
//! `?q=` those that have the words. A new thread starts in place at the
//! top. The list also shows the threads found across the forum (`index.rs`).

use leptos::ev::SubmitEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_params_map, use_query_map};
use send_wrapper::SendWrapper;
use starbooru_core::forum::{DEFAULT_THREADS_LIMIT, MAX_TITLE_CHARS};
use starbooru_core::{ForumThread, NewThread};

use super::compose::Composer;
use super::{
    Ago, Author, ForumOnly, NoForum, ask_categories, categories, category_name, is_off, posted,
};
use crate::api;
use crate::components::avatar::Size;
use crate::components::back::BackButton;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::dom;
use crate::logic::forum::{check_title, error_text, merge_threads, next_page, slow_down};
use crate::{instance, route};

#[component]
pub fn ForumCategoryPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let id =
        Memo::new(move |_| params.with(|p| p.get("id").and_then(|s| s.trim().parse::<i64>().ok())));
    let q = Memo::new(move |_| query.with(|m| m.get("q").unwrap_or_default()));
    ask_categories();
    let category = Memo::new(move |_| {
        let id = id.get()?;
        categories()?.into_iter().find(|c| c.id == id)
    });
    let writing = RwSignal::new(false);
    let start = move |_| {
        if instance::guest() {
            instance::ask_sign_in(Some(t!("forum-guest-why")));
        } else {
            writing.set(true);
        }
    };
    view! {
        <div class="page narrow forum">
            <ForumOnly>
                <div class="titled">
                    <BackButton up=route::FORUM />
                    <h1 class="page-title break">
                        {move || category.with(|c| c.as_ref().map(|c| c.name.clone())).unwrap_or_else(|| t!("nav-forum"))}
                    </h1>
                </div>
                {move || {
                    category
                        .with(|c| c.as_ref().map(|c| c.description.clone()))
                        .filter(|d| !d.trim().is_empty())
                        .map(|d| view! { <p class="muted lead">{d}</p> })
                }}
                {move || {
                    let q = q.get();
                    (!q.trim().is_empty())
                        .then(|| {
                            let back = id.get().map(|id| route::forum_category(id, "")).unwrap_or_default();
                            view! {
                                <div class="forum-found">
                                    <span class="break">{t!("forum-found", q = q.clone())}</span>
                                    <a class="btn text small" href=back>
                                        <Ico icon=Icon::Close size=18 />
                                        {move || t!("forum-found-clear")}
                                    </a>
                                </div>
                            }
                        })
                }}
                {move || {
                    let Some(id) = id.get() else {
                        return view! { <NoCategory /> }.into_any();
                    };
                    let q = q.get();
                    view! {
                        <Show
                            when=move || writing.get()
                            fallback=move || {
                                view! {
                                    <div class="forum-actions">
                                        <button type="button" class="btn" on:click=start>
                                            <Ico icon=Icon::Add />
                                            {move || t!("forum-thread-new")}
                                        </button>
                                    </div>
                                }
                            }
                        >
                            <NewThreadForm category=id on_cancel=Callback::new(move |()| writing.set(false)) />
                        </Show>
                        <ThreadList category=Some(id) q=q.clone() />
                    }
                        .into_any()
                }}
            </ForumOnly>
        </div>
    }
}

#[component]
fn NoCategory() -> impl IntoView {
    view! {
        <div class="card empty">
            <Ico icon=Icon::Forum size=48 />
            <h2>{move || t!("forum-category-missing")}</h2>
            <a class="btn" href=route::FORUM>
                {move || t!("forum-back")}
            </a>
        </div>
    }
}

/// What a list of threads can show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shown {
    Waiting,
    Ready,
    Missing,
    Off,
}

/// The threads of `category`, or of the whole forum, found by `q` when it
/// has words: a page at a time as the end comes near.
#[component]
pub fn ThreadList(category: Option<i64>, q: String) -> impl IntoView {
    let threads = RwSignal::new(Vec::<ForumThread>::new());
    let page = RwSignal::new(0_u32);
    let more = RwSignal::new(true);
    let loading = RwSignal::new(false);
    let shown = RwSignal::new(Shown::Waiting);
    let error = RwSignal::new(None::<String>);
    let searching = !q.trim().is_empty();
    let q = StoredValue::new(q);

    let load_more = move || {
        if loading.get_untracked() || !more.get_untracked() {
            return;
        }
        let Some(next) = next_page(page.get_untracked(), true) else {
            return;
        };
        loading.set(true);
        error.set(None);
        let words = q.get_value();
        spawn_local(async move {
            let result = api::attempt(move |c| async move {
                c.forum_threads(category, &words, next, DEFAULT_THREADS_LIMIT)
                    .await
            })
            .await;
            loading.try_set(false);
            match result {
                Ok(found) => {
                    threads.update(|t| merge_threads(t, found.threads));
                    page.set(found.page.max(next));
                    more.set(found.has_more);
                    shown.set(Shown::Ready);
                }
                Err(r) if is_off(r.code.as_deref()) => {
                    shown.try_set(Shown::Off);
                }
                Err(r) if r.missing => {
                    shown.try_set(Shown::Missing);
                }
                Err(r) => {
                    error.try_set(Some(r.message));
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
    let ids = Memo::new(move |_| threads.with(|t| t.iter().map(|t| t.id).collect::<Vec<_>>()));
    let empty = move || {
        let text = if searching {
            t!("forum-found-none")
        } else {
            t!("forum-threads-none")
        };
        view! {
            <div class="card empty">
                <Ico icon=Icon::Forum size=48 />
                <p class="muted">{text}</p>
            </div>
        }
    };
    // The list stays while its first page comes, so its end, watched for
    // the next page, is drawn once.
    let gone = Memo::new(move |_| matches!(shown.get(), Shown::Off | Shown::Missing));
    move || {
        if gone.get() {
            return match shown.get_untracked() {
                Shown::Off => view! { <NoForum /> }.into_any(),
                _ => view! { <NoCategory /> }.into_any(),
            };
        }
        view! {
            {move || error.get().map(|m| view! { <ErrorBanner message=m on_retry=retry /> })}
            <ul class="threads">
                <For
                    each=move || ids.get()
                    key=|id| *id
                    children=move |id| {
                        let thread = threads.with_untracked(|t| t.iter().find(|t| t.id == id).cloned());
                        thread.map(|t| view! { <ThreadRow thread=t across=category.is_none() /> })
                    }
                />
            </ul>
            <Show when=move || shown.get() == Shown::Ready && threads.with(Vec::is_empty)>{empty}</Show>
            <div
                class="sentinel"
                class:done=move || !more.get() && !loading.get() && error.with(Option::is_none)
                node_ref=sentinel
            >
                <Show when=move || loading.get()>
                    <Spinner />
                </Show>
                <Show when=move || !loading.get() && more.get() && shown.get() == Shown::Ready && error.with(Option::is_none)>
                    <button type="button" class="btn text" on:click=move |_| load_more()>
                        {move || t!("forum-more")}
                    </button>
                </Show>
            </div>
        }
        .into_any()
    }
}

/// A thread in a list: whether it's pinned or locked, its title, who
/// started it, its replies and when it was last written in. `across` names
/// its category, for a list of the whole forum.
#[component]
fn ThreadRow(thread: ForumThread, across: bool) -> impl IntoView {
    let href = route::forum_thread(thread.id);
    let replies = thread.posts.saturating_sub(1);
    let category = thread.category_id;
    let place = move || across.then(|| category_name(category)).flatten();
    view! {
        <li class="thread" class:pinned=thread.pinned class:locked=thread.locked>
            <a class="thread-link" href=href>
                <span class="thread-title">
                    {thread.pinned.then(|| view! {
                        <span class="thread-mark" title=move || t!("forum-pinned")>
                            <Ico icon=Icon::Pin size=16 />
                            <span class="visually-hidden">{move || t!("forum-pinned")}</span>
                        </span>
                    })}
                    {thread.locked.then(|| view! {
                        <span class="thread-mark" title=move || t!("forum-locked")>
                            <Ico icon=Icon::Lock size=16 />
                            <span class="visually-hidden">{move || t!("forum-locked")}</span>
                        </span>
                    })}
                    <strong class="break">{thread.title.clone()}</strong>
                </span>
                <span class="thread-meta muted">
                    {move || place().map(|name| view! { <span class="thread-place">{name}</span> })}
                    <span>{move || t!("forum-replies", count = replies)}</span>
                    <Ago at=thread.last_activity_at />
                </span>
            </a>
            <span class="thread-by">
                <Author card=thread.author.clone() size=Size::Small />
            </span>
        </li>
    }
}

/// A new thread in `category`: its title and first post. Posting opens it.
#[component]
fn NewThreadForm(category: i64, on_cancel: Callback<()>) -> impl IntoView {
    let title = RwSignal::new(String::new());
    let body = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let tried = RwSignal::new(false);
    let navigate = SendWrapper::new(use_navigate());
    let title_ok = Signal::derive(move || title.with(|t| check_title(t).is_ok()));
    let title_problem = move || {
        tried
            .get()
            .then(|| title.with(|t| check_title(t).err().map(|e| error_text(&e))))
            .flatten()
    };
    let send = Callback::new(move |text: String| {
        tried.set(true);
        let Ok(heading) = title.with_untracked(|t| check_title(t)) else {
            return;
        };
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        let new = NewThread {
            category_id: category,
            title: heading,
            body: text,
        };
        let navigate = navigate.clone();
        spawn_local(async move {
            let result =
                api::attempt(move |c| async move { c.create_forum_thread(&new).await }).await;
            busy.try_set(false);
            match result {
                Ok(thread) => {
                    posted(thread.created_at);
                    (*navigate)(&route::forum_thread(thread.id), NavigateOptions::default());
                }
                Err(r) if r.status == Some(429) => {
                    error.try_set(Some(slow_down(r.retry_after)));
                }
                Err(r) => {
                    error.try_set(Some(r.message));
                }
            }
        });
    });
    let no_submit = |e: SubmitEvent| e.prevent_default();
    view! {
        <form class="card new-thread" on:submit=no_submit>
            <h2 class="card-title">{move || t!("forum-thread-new")}</h2>
            <label class="field">
                <span>{move || t!("forum-thread-title")}</span>
                <input
                    maxlength=MAX_TITLE_CHARS
                    autocomplete="off"
                    aria-invalid=move || if title_problem().is_some() { "true" } else { "false" }
                    bind:value=title
                />
                {move || title_problem().map(|p| view! { <small class="field-error">{p}</small> })}
            </label>
            <Composer
                text=body
                label=Signal::derive(|| t!("forum-thread-body"))
                send_label=Signal::derive(|| t!("forum-thread-post"))
                busy=busy
                on_send=send
                waits=true
                ready=title_ok
                on_not_ready=Callback::new(move |()| tried.set(true))
                on_cancel=on_cancel
            />
            {move || error.get().map(|m| view! { <ErrorBanner message=m /> })}
        </form>
    }
}
