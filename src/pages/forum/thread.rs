//! `/forum/t/{id}`: a thread, its posts oldest first, each with its
//! author's picture and name, when it was written and whether it was
//! edited; a deleted one keeps its place, empty. Posts read as Markdown,
//! and the posts of the booru they name link to them with a preview. The
//! box to reply waits at the end; a locked thread says so instead, and a
//! guest is asked to sign in. Authors edit their posts in place, with Undo,
//! and delete them after "Sure?"; administrators pin, lock, move, rename
//! and delete the thread from its menu.

use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_location, use_navigate, use_params_map};
use send_wrapper::SendWrapper;
use starbooru_core::forum::{DEFAULT_POSTS_LIMIT, MAX_TITLE_CHARS};
use starbooru_core::{ForumPost, ForumThread, ThreadUpdate};
use wasm_bindgen::JsCast;

use super::compose::Composer;
use super::{
    Ago, Author, ForumOnly, NoForum, ask_categories, categories, category_name, is_off, me, posted,
};
use crate::api;
use crate::components::back::BackButton;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::confirm::ConfirmButton;
use crate::components::icons::{Ico, Icon};
use crate::components::invite::SignInButtons;
use crate::components::markdown::Markdown;
use crate::components::menu::{ActionMenu, MenuAt, MenuItem, beside};
use crate::components::snackbar::{notify, notify_undo};
use crate::logic::forum::{
    Reply, check_title, error_text, latest_by, may_delete, may_edit, may_rename, merge_posts,
    next_page, reply, slow_down,
};
use crate::route;

/// What the page shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shown {
    Waiting,
    Ready,
    Missing,
    Off,
}

#[component]
pub fn ForumThreadPage() -> impl IntoView {
    let params = use_params_map();
    let id =
        Memo::new(move |_| params.with(|p| p.get("id").and_then(|s| s.trim().parse::<i64>().ok())));
    view! {
        <div class="page narrow forum">
            <ForumOnly>
                {move || match id.get() {
                    Some(id) => view! { <Thread id=id /> }.into_any(),
                    None => view! { <Gone /> }.into_any(),
                }}
            </ForumOnly>
        </div>
    }
}

#[component]
fn Gone() -> impl IntoView {
    view! {
        <div class="card empty">
            <Ico icon=Icon::Forum size=48 />
            <h2>{move || t!("forum-thread-missing")}</h2>
            <a class="btn" href=route::FORUM>
                {move || t!("forum-back")}
            </a>
        </div>
    }
}

#[component]
fn Thread(id: i64) -> impl IntoView {
    ask_categories();
    let thread = RwSignal::new(None::<ForumThread>);
    let posts = RwSignal::new(Vec::<ForumPost>::new());
    let page = RwSignal::new(0_u32);
    let more = RwSignal::new(false);
    let loading = RwSignal::new(false);
    let shown = RwSignal::new(Shown::Waiting);
    let error = RwSignal::new(None::<String>);
    let location = use_location();

    let load = move |next: u32| {
        if loading.get_untracked() {
            return;
        }
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            let result = api::attempt(move |c| async move {
                c.forum_thread(id, next, DEFAULT_POSTS_LIMIT).await
            })
            .await;
            loading.try_set(false);
            match result {
                Ok(view) => {
                    if let Some(me) = me().0
                        && let Some(at) = latest_by(&view.posts, &me)
                    {
                        posted(at);
                    }
                    let first = shown.get_untracked() == Shown::Waiting;
                    thread.set(Some(view.thread));
                    posts.update(|p| merge_posts(p, view.posts));
                    page.set(view.page.max(next));
                    more.set(view.has_more);
                    shown.set(Shown::Ready);
                    // An address with a post's anchor goes down to it.
                    let hash = location.hash.get_untracked();
                    if first && hash.len() > 1 {
                        reveal(hash.trim_start_matches('#').to_owned());
                    }
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
    load(1);
    let retry = Callback::new(move |()| {
        let next = if shown.get_untracked() == Shown::Waiting {
            1
        } else {
            next_page(page.get_untracked(), true).unwrap_or(1)
        };
        load(next);
    });
    let load_more = move |_| {
        if let Some(next) = next_page(page.get_untracked(), more.get_untracked()) {
            load(next);
        }
    };
    let ids = Memo::new(move |_| posts.with(|p| p.iter().map(|p| p.id).collect::<Vec<_>>()));

    move || {
        match shown.get() {
        Shown::Off => view! { <NoForum /> }.into_any(),
        Shown::Missing => view! { <Gone /> }.into_any(),
        Shown::Waiting => view! {
            {move || error.get().map(|m| view! { <ErrorBanner message=m on_retry=retry /> })}
            <div class="sentinel">
                <Spinner />
            </div>
        }
        .into_any(),
        Shown::Ready => view! {
            <ThreadHead thread=thread />
            <ol class="forum-posts">
                <For
                    each=move || ids.get()
                    key=|id| *id
                    children=move |post_id| view! { <PostItem id=post_id posts=posts thread=thread /> }
                />
            </ol>
            {move || error.get().map(|m| view! { <ErrorBanner message=m on_retry=retry /> })}
            <Show when=move || more.get()>
                <div class="sentinel">
                    <Show
                        when=move || !loading.get()
                        fallback=|| view! { <Spinner /> }
                    >
                        <button type="button" class="btn tonal" on:click=load_more>
                            {move || t!("forum-more-posts")}
                        </button>
                    </Show>
                </div>
            </Show>
            <ReplyBox thread=thread posts=posts />
        }
        .into_any(),
    }
    }
}

/// Scrolls to the post with this anchor once it's drawn.
fn reveal(anchor: String) {
    request_animation_frame(move || {
        let el = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(&anchor));
        if let Some(el) = el {
            let options = web_sys::ScrollIntoViewOptions::new();
            options.set_block(web_sys::ScrollLogicalPosition::Center);
            el.scroll_into_view_with_scroll_into_view_options(&options);
            let _ = el.class_list().add_1("flash");
        }
    });
}

/// The thread's title with its marks, the category it's in, and the menu
/// of what its author or an administrator may do with it.
#[component]
fn ThreadHead(thread: RwSignal<Option<ForumThread>>) -> impl IntoView {
    let read =
        move |f: fn(&ForumThread) -> bool| move || thread.with(|t| t.as_ref().is_some_and(f));
    let pinned = read(|t| t.pinned);
    let locked = read(|t| t.locked);
    let category = Memo::new(move |_| thread.with(|t| t.as_ref().map(|t| t.category_id)));
    let up = untrack(|| {
        category
            .get()
            .map_or_else(|| route::FORUM.to_owned(), |c| route::forum_category(c, ""))
    });
    let title = move || {
        thread
            .with(|t| t.as_ref().map(|t| t.title.clone()))
            .unwrap_or_default()
    };
    let renaming = RwSignal::new(false);
    let deleting = RwSignal::new(false);
    let menu = RwSignal::new(None::<MenuAt>);
    let (me, admin) = untrack(me);
    let tools = Memo::new(move |_| {
        let manage = admin;
        let rename = thread.with(|t| {
            t.as_ref()
                .is_some_and(|t| may_rename(t, me.as_deref(), admin))
        });
        manage || rename
    });
    let open_menu = move |e: leptos::ev::MouseEvent| {
        if let Some(button) = e
            .current_target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
        {
            menu.set(Some(beside(&button)));
        }
    };
    view! {
        <div class="thread-head">
            <div class="titled">
                <BackButton up=up />
                <Show
                    when=move || renaming.get()
                    fallback=move || view! { <h1 class="page-title break">{title}</h1> }
                >
                    <RenameForm thread=thread on_done=Callback::new(move |()| renaming.set(false)) />
                </Show>
                <Show when=move || tools.get() && !renaming.get()>
                    <button
                        type="button"
                        class="icon-btn thread-menu"
                        title=move || t!("forum-thread-manage")
                        aria-label=move || t!("forum-thread-manage")
                        aria-haspopup="menu"
                        on:click=open_menu
                    >
                        <Ico icon=Icon::Tune />
                    </button>
                </Show>
            </div>
            <div class="thread-marks">
                {move || {
                    category
                        .get()
                        .map(|c| {
                            let name = move || category_name(c).unwrap_or_else(|| t!("nav-forum"));
                            view! {
                                <a class="pill thread-category" href=route::forum_category(c, "")>
                                    <Ico icon=Icon::Forum size=16 />
                                    {name}
                                </a>
                            }
                        })
                }}
                <Show when=pinned>
                    <span class="pill thread-pill">
                        <Ico icon=Icon::Pin size=16 />
                        {move || t!("forum-pinned")}
                    </span>
                </Show>
                <Show when=locked>
                    <span class="pill thread-pill locked">
                        <Ico icon=Icon::Lock size=16 />
                        {move || t!("forum-locked")}
                    </span>
                </Show>
            </div>
            <Show when=move || deleting.get()>
                <DeleteThread thread=thread on_cancel=Callback::new(move |()| deleting.set(false)) />
            </Show>
        </div>
        {move || {
            menu.get()
                .map(|at| {
                    view! {
                        <ThreadMenu
                            at=at
                            thread=thread
                            admin=admin
                            on_rename=Callback::new(move |()| renaming.set(true))
                            on_delete=Callback::new(move |()| deleting.set(true))
                            on_close=Callback::new(move |()| menu.set(None))
                        />
                    }
                })
        }}
    }
}

/// Sends `change` to the thread, and offers to take it back with `undo`.
fn change_thread(
    thread: RwSignal<Option<ForumThread>>,
    change: ThreadUpdate,
    said: String,
    undo: Option<ThreadUpdate>,
) {
    let Some(id) = thread.with_untracked(|t| t.as_ref().map(|t| t.id)) else {
        return;
    };
    spawn_local(async move {
        match api::call(move |c| async move { c.update_forum_thread(id, &change).await }).await {
            Ok(fresh) => {
                thread.try_set(Some(fresh));
                match undo {
                    Some(back) => notify_undo(said, untrack(|| t!("press-undo")), move || {
                        let back = back.clone();
                        change_thread(thread, back, untrack(|| t!("forum-undone")), None);
                    }),
                    None => notify(said),
                }
            }
            Err(message) => notify(message),
        }
    });
}

/// What an administrator, or the thread's author, may do with it.
#[component]
fn ThreadMenu(
    at: MenuAt,
    thread: RwSignal<Option<ForumThread>>,
    admin: bool,
    on_rename: Callback<()>,
    on_delete: Callback<()>,
    on_close: Callback<()>,
) -> impl IntoView {
    let Some(now) = thread.get_untracked() else {
        return ().into_any();
    };
    let toggle = |pinned: Option<bool>, locked: Option<bool>| ThreadUpdate {
        pinned,
        locked,
        ..ThreadUpdate::default()
    };
    let pin = Callback::new(move |()| {
        on_close.run(());
        let on = !now.pinned;
        let said = untrack(|| {
            if on {
                t!("forum-pinned-done")
            } else {
                t!("forum-unpinned-done")
            }
        });
        change_thread(
            thread,
            toggle(Some(on), None),
            said,
            Some(toggle(Some(!on), None)),
        );
    });
    let lock = Callback::new(move |()| {
        on_close.run(());
        let on = !now.locked;
        let said = untrack(|| {
            if on {
                t!("forum-locked-done")
            } else {
                t!("forum-unlocked-done")
            }
        });
        change_thread(
            thread,
            toggle(None, Some(on)),
            said,
            Some(toggle(None, Some(!on))),
        );
    });
    let rename = Callback::new(move |()| {
        on_close.run(());
        on_rename.run(());
    });
    let delete = Callback::new(move |()| {
        on_close.run(());
        on_delete.run(());
    });
    let from = now.category_id;
    let moves = admin.then(|| {
        categories()
            .unwrap_or_default()
            .into_iter()
            .filter(|c| c.id != from)
            .map(|c| {
                let to = c.id;
                let name = c.name.clone();
                let go = Callback::new(move |()| {
                    on_close.run(());
                    let said = untrack(|| t!("forum-moved-done", name = name.clone()));
                    let there = ThreadUpdate {
                        category_id: Some(to),
                        ..ThreadUpdate::default()
                    };
                    let back = ThreadUpdate {
                        category_id: Some(from),
                        ..ThreadUpdate::default()
                    };
                    change_thread(thread, there, said, Some(back));
                });
                let label = c.name.clone();
                view! {
                    <MenuItem
                        icon=Icon::Forum
                        label=Signal::derive(move || t!("forum-move-to", name = label.clone()))
                        on_click=go
                    />
                }
            })
            .collect_view()
    });
    let pinned = now.pinned;
    let locked = now.locked;
    view! {
        <ActionMenu at=at title=untrack(|| t!("forum-thread-manage")) on_close=on_close>
            {admin.then(|| view! {
                <MenuItem
                    icon=Icon::Pin
                    label=Signal::derive(move || if pinned { t!("forum-unpin") } else { t!("forum-pin") })
                    hint=Signal::derive(move || if pinned { t!("forum-unpin-hint") } else { t!("forum-pin-hint") })
                    on_click=pin
                />
                <MenuItem
                    icon=Icon::Lock
                    label=Signal::derive(move || if locked { t!("forum-unlock") } else { t!("forum-lock") })
                    hint=Signal::derive(move || if locked { t!("forum-unlock-hint") } else { t!("forum-lock-hint") })
                    on_click=lock
                />
            })}
            <MenuItem icon=Icon::Edit label=Signal::derive(|| t!("forum-rename")) on_click=rename />
            {moves}
            {admin.then(|| view! {
                <MenuItem
                    icon=Icon::Delete
                    label=Signal::derive(|| t!("forum-thread-delete"))
                    danger=true
                    on_click=delete
                />
            })}
        </ActionMenu>
    }
    .into_any()
}

/// The title as a field, in the heading's place.
#[component]
fn RenameForm(thread: RwSignal<Option<ForumThread>>, on_done: Callback<()>) -> impl IntoView {
    let typed = RwSignal::new(
        thread
            .with_untracked(|t| t.as_ref().map(|t| t.title.clone()))
            .unwrap_or_default(),
    );
    let busy = RwSignal::new(false);
    let problem = RwSignal::new(None::<String>);
    let input: NodeRef<html::Input> = NodeRef::new();
    input.on_load(|el| {
        let _ = el.focus();
        el.select();
    });
    let save = move |e: leptos::ev::SubmitEvent| {
        e.prevent_default();
        let title = match typed.with_untracked(|t| check_title(t)) {
            Ok(title) => title,
            Err(e) => {
                problem.set(Some(error_text(&e)));
                return;
            }
        };
        let (Some(id), Some(old)) = (
            thread.with_untracked(|t| t.as_ref().map(|t| t.id)),
            thread.with_untracked(|t| t.as_ref().map(|t| t.title.clone())),
        ) else {
            return;
        };
        if title == old {
            on_done.run(());
            return;
        }
        busy.set(true);
        let change = ThreadUpdate {
            title: Some(title),
            ..ThreadUpdate::default()
        };
        spawn_local(async move {
            let result =
                api::call(move |c| async move { c.update_forum_thread(id, &change).await }).await;
            busy.try_set(false);
            match result {
                Ok(fresh) => {
                    thread.try_set(Some(fresh));
                    on_done.try_run(());
                    let back = ThreadUpdate {
                        title: Some(old),
                        ..ThreadUpdate::default()
                    };
                    notify_undo(
                        untrack(|| t!("forum-renamed-done")),
                        untrack(|| t!("press-undo")),
                        move || {
                            change_thread(
                                thread,
                                back.clone(),
                                untrack(|| t!("forum-undone")),
                                None,
                            );
                        },
                    );
                }
                Err(message) => {
                    problem.try_set(Some(message));
                }
            }
        });
    };
    view! {
        <form class="rename" on:submit=save>
            <input
                node_ref=input
                class="rename-input"
                maxlength=MAX_TITLE_CHARS
                aria-label=move || t!("forum-thread-title")
                bind:value=typed
            />
            <button type="button" class="btn text small" on:click=move |_| on_done.run(())>
                {move || t!("cancel")}
            </button>
            <button type="submit" class="btn small" disabled=move || busy.get()>
                {move || t!("admin-save")}
            </button>
            {move || problem.get().map(|p| view! { <small class="field-error">{p}</small> })}
        </form>
    }
}

/// Deleting the thread asks first, under its title, saying how much goes.
#[component]
fn DeleteThread(thread: RwSignal<Option<ForumThread>>, on_cancel: Callback<()>) -> impl IntoView {
    let busy = RwSignal::new(false);
    let navigate = SendWrapper::new(use_navigate());
    let count = move || thread.with(|t| t.as_ref().map_or(0, |t| t.posts));
    let remove = move |_| {
        let Some((id, category)) =
            thread.with_untracked(|t| t.as_ref().map(|t| (t.id, t.category_id)))
        else {
            return;
        };
        busy.set(true);
        let navigate = navigate.clone();
        spawn_local(async move {
            match api::call(move |c| async move { c.delete_forum_thread(id).await }).await {
                Ok(()) => {
                    notify(untrack(|| t!("forum-thread-deleted")));
                    (*navigate)(
                        &route::forum_category(category, ""),
                        NavigateOptions::default(),
                    );
                }
                Err(message) => {
                    busy.try_set(false);
                    notify(message);
                }
            }
        });
    };
    view! {
        <div class="thread-delete" role="alertdialog" aria-labelledby="thread-delete-text">
            <p id="thread-delete-text">{move || t!("forum-thread-delete-sure", count = count())}</p>
            <div class="row-end">
                <button type="button" class="btn text" on:click=move |_| on_cancel.run(())>
                    {move || t!("cancel")}
                </button>
                <button type="button" class="btn danger" disabled=move || busy.get() on:click=remove>
                    <Ico icon=Icon::Delete />
                    {move || t!("forum-thread-delete")}
                </button>
            </div>
        </div>
    }
}

/// One post. Its row follows the list, so an edit or a deletion shows in
/// place.
#[component]
fn PostItem(
    id: i64,
    posts: RwSignal<Vec<ForumPost>>,
    thread: RwSignal<Option<ForumThread>>,
) -> impl IntoView {
    let post = Memo::new(move |_| posts.with(|p| p.iter().find(|p| p.id == id).cloned()));
    let editing = RwSignal::new(false);
    let draft = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let (me, admin) = untrack(me);
    let thread_id = untrack(|| thread.with(|t| t.as_ref().map_or(0, |t| t.id)));
    let locked = Memo::new(move |_| thread.with(|t| t.as_ref().is_some_and(|t| t.locked)));
    let me_edit = me.clone();
    let editable = Memo::new(move |_| {
        post.with(|p| {
            p.as_ref()
                .is_some_and(|p| may_edit(p, me_edit.as_deref(), locked.get()))
        })
    });
    let deletable = Memo::new(move |_| {
        post.with(|p| {
            p.as_ref()
                .is_some_and(|p| may_delete(p, me.as_deref(), admin))
        })
    });
    let put = move |fresh: ForumPost| {
        posts.try_update(|list| {
            if let Some(slot) = list.iter_mut().find(|p| p.id == fresh.id) {
                *slot = fresh;
            }
        });
    };
    let save = Callback::new(move |body: String| {
        let Some(old) = post.get_untracked() else {
            return;
        };
        if body == old.body {
            editing.set(false);
            return;
        }
        busy.set(true);
        spawn_local(async move {
            let sent = body.clone();
            let result =
                api::call(move |c| async move { c.edit_forum_post(id, &sent).await }).await;
            busy.try_set(false);
            match result {
                Ok(fresh) => {
                    put(fresh);
                    editing.try_set(false);
                    let before = old.body;
                    notify_undo(
                        untrack(|| t!("forum-edited-done")),
                        untrack(|| t!("press-undo")),
                        move || {
                            let before = before.clone();
                            spawn_local(async move {
                                match api::call(move |c| async move {
                                    c.edit_forum_post(id, &before).await
                                })
                                .await
                                {
                                    Ok(back) => put(back),
                                    Err(message) => notify(message),
                                }
                            });
                        },
                    );
                }
                Err(message) => notify(message),
            }
        });
    });
    let remove = Callback::new(move |()| {
        busy.set(true);
        spawn_local(async move {
            let result = api::call(move |c| async move { c.delete_forum_post(id).await }).await;
            busy.try_set(false);
            match result {
                Ok(()) => {
                    posts.try_update(|list| {
                        if let Some(p) = list.iter_mut().find(|p| p.id == id) {
                            p.deleted = true;
                            p.body.clear();
                            p.author = None;
                        }
                    });
                    notify(untrack(|| t!("forum-post-deleted")));
                }
                Err(message) => notify(message),
            }
        });
    });
    let start_edit = move |_| {
        draft.set(
            post.with_untracked(|p| p.as_ref().map(|p| p.body.clone()))
                .unwrap_or_default(),
        );
        editing.set(true);
    };
    let anchor = route::forum_anchor(id);
    let link = route::forum_post(thread_id, id);
    move || {
        let p = post.get()?;
        if p.deleted {
            return Some(
                view! {
                    <li class="forum-post gone" id=anchor.clone()>
                        <span class="avatar" aria-hidden="true">
                            <Ico icon=Icon::Delete size=18 />
                        </span>
                        <p class="muted">
                            {move || t!("forum-post-gone")} " · " <Ago at=p.created_at />
                        </p>
                    </li>
                }
                .into_any(),
            );
        }
        let edited = p.edited_at.map(|at| {
            let exact = crate::logic::viewer::when(&at, &chrono::Local);
            view! {
                <span class="forum-edited" title=move || t!("forum-edited-at", when = exact.clone())>
                    {move || t!("forum-edited")}
                </span>
            }
        });
        let body = p.body.clone();
        Some(
            view! {
                <li class="forum-post" id=anchor.clone()>
                    <div class="forum-post-head">
                        <Author card=p.author.clone() />
                        <a class="forum-post-time muted" href=link.clone()>
                            <Ago at=p.created_at />
                        </a>
                        {edited}
                        <span class="forum-post-tools">
                            <Show when=move || editable.get() && !editing.get()>
                                <button
                                    type="button"
                                    class="icon-btn small"
                                    title=move || t!("forum-edit")
                                    aria-label=move || t!("forum-edit")
                                    on:click=start_edit
                                >
                                    <Ico icon=Icon::Edit size=18 />
                                </button>
                            </Show>
                            <Show when=move || deletable.get() && !editing.get()>
                                <ConfirmButton
                                    icon=Icon::Delete
                                    title=Signal::derive(|| t!("forum-delete"))
                                    label=Signal::derive(|| t!("forum-delete"))
                                    disabled=busy
                                    on_confirm=remove
                                />
                            </Show>
                        </span>
                    </div>
                    <Show
                        when=move || editing.get()
                        fallback=move || {
                            let body = body.clone();
                            view! { <Markdown text=body mentions=true /> }
                        }
                    >
                        <Composer
                            text=draft
                            label=Signal::derive(|| t!("forum-edit"))
                            send_label=Signal::derive(|| t!("admin-save"))
                            busy=busy
                            on_send=save
                            on_cancel=Callback::new(move |()| editing.set(false))
                            focus=true
                        />
                    </Show>
                </li>
            }
            .into_any(),
        )
    }
}

/// The end of the thread: the box to reply in, or why there's none.
#[component]
fn ReplyBox(
    thread: RwSignal<Option<ForumThread>>,
    posts: RwSignal<Vec<ForumPost>>,
) -> impl IntoView {
    let text = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let (me, admin) = untrack(me);
    let signed_in = me.is_some();
    let offer = Memo::new(move |_| thread.with(|t| t.as_ref().map(|t| reply(t, signed_in, admin))));
    let send = Callback::new(move |body: String| {
        let Some(id) = thread.with_untracked(|t| t.as_ref().map(|t| t.id)) else {
            return;
        };
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let result =
                api::attempt(move |c| async move { c.reply_in_forum(id, &body).await }).await;
            busy.try_set(false);
            match result {
                Ok(post) => {
                    posted(post.created_at);
                    let anchor = route::forum_anchor(post.id);
                    let at = post.created_at;
                    posts.update(|p| merge_posts(p, vec![post]));
                    thread.update(|t| {
                        if let Some(t) = t {
                            t.posts = t.posts.saturating_add(1);
                            t.last_activity_at = at;
                        }
                    });
                    text.set(String::new());
                    reveal(anchor);
                }
                Err(r) if r.status == Some(429) => {
                    error.try_set(Some(slow_down(r.retry_after)));
                }
                Err(r) if r.code.as_deref() == Some("locked") => {
                    thread.try_update(|t| {
                        if let Some(t) = t {
                            t.locked = true;
                        }
                    });
                }
                Err(r) => {
                    error.try_set(Some(r.message));
                }
            }
        });
    });
    move || match offer.get()? {
        Reply::Open => Some(
            view! {
                <section class="card reply" aria-labelledby="reply-title">
                    <h2 class="card-title" id="reply-title">{move || t!("forum-reply")}</h2>
                    {move || error.get().map(|m| view! { <ErrorBanner message=m /> })}
                    <Composer
                        text=text
                        label=Signal::derive(|| t!("forum-reply"))
                        send_label=Signal::derive(|| t!("forum-reply-send"))
                        busy=busy
                        on_send=send
                        waits=true
                    />
                </section>
            }
            .into_any(),
        ),
        Reply::Locked => Some(
            view! {
                <div class="banner warn locked-note">
                    <Ico icon=Icon::Lock />
                    <span>{move || t!("forum-locked-note")}</span>
                </div>
            }
            .into_any(),
        ),
        Reply::SignIn => Some(
            view! {
                <section class="card reply guest">
                    <p class="muted">{move || t!("forum-guest-reply")}</p>
                    <SignInButtons />
                </section>
            }
            .into_any(),
        ),
    }
}
