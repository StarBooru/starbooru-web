//! `/forum`: the categories, each with what it's for, how many threads it
//! has and when it was last written in; with `?q=` the threads found in all
//! of them. Administrators edit, reorder, add and delete categories here,
//! each in place under its row.

use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_query_map;
use starbooru_core::forum::{MAX_CATEGORY_DESCRIPTION_CHARS, MAX_CATEGORY_NAME_CHARS};
use starbooru_core::{CategoryUpdate, ForumCategory, NewCategory};

use super::category::ThreadList;
use super::{Ago, ForumOnly, NoForum, categories, is_off, load_categories, me};
use crate::api;
use crate::components::back::BackButton;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::confirm::ConfirmButton;
use crate::components::icons::{Ico, Icon};
use crate::components::snackbar::notify;
use crate::logic::forum::{check_category, error_text, swapped};
use crate::route;

#[component]
pub fn Forum() -> impl IntoView {
    let query = use_query_map();
    let q = Memo::new(move |_| query.with(|m| m.get("q").unwrap_or_default()));
    view! {
        <div class="page narrow forum">
            <ForumOnly>
                {move || {
                    let q = q.get();
                    if q.trim().is_empty() {
                        view! { <Categories /> }.into_any()
                    } else {
                        let heading = q.clone();
                        view! {
                            <div class="titled">
                                <BackButton up=route::FORUM />
                                <h1 class="page-title break">{move || t!("forum-found", q = heading.clone())}</h1>
                            </div>
                            <ThreadList category=None q=q.clone() />
                        }
                            .into_any()
                    }
                }}
            </ForumOnly>
        </div>
    }
}

/// The list of categories, with an administrator's tools.
#[component]
fn Categories() -> impl IntoView {
    let error = RwSignal::new(None::<String>);
    let off = RwSignal::new(false);
    let load = move || {
        error.set(None);
        spawn_local(async move {
            if let Err(r) = load_categories().await {
                if is_off(r.code.as_deref()) {
                    off.try_set(true);
                } else {
                    error.try_set(Some(r.message));
                }
            }
        });
    };
    // Fresh counts each time the page opens; the ones kept show meanwhile.
    load();
    let retry = Callback::new(move |()| load());
    let admin = Memo::new(|_| me().1);
    let editing = RwSignal::new(None::<i64>);
    let adding = RwSignal::new(false);
    let list = Memo::new(|_| categories());
    let ids =
        Memo::new(move |_| list.with(|l| l.iter().flatten().map(|c| c.id).collect::<Vec<_>>()));
    view! {
        <h1 class="page-title">{move || t!("nav-forum")}</h1>
        {move || error.get().map(|m| view! { <ErrorBanner message=m on_retry=retry /> })}
        <Show when=move || !off.get() fallback=|| view! { <NoForum /> }>
            <Show
                when=move || list.with(Option::is_some)
                fallback=move || error.with(Option::is_none).then(|| view! { <div class="sentinel"><Spinner /></div> })
            >
                <ul class="categories">
                    <For
                        each=move || ids.get()
                        key=|id| *id
                        children=move |id| view! { <CategoryRow id=id list=list admin=admin editing=editing /> }
                    />
                </ul>
                <Show when=move || list.with(|l| l.as_ref().is_some_and(Vec::is_empty))>
                    <div class="card empty">
                        <Ico icon=Icon::Forum size=48 />
                        <p class="muted">{move || t!("forum-no-categories")}</p>
                    </div>
                </Show>
            </Show>
            <Show when=move || admin.get()>
                <Show
                    when=move || adding.get()
                    fallback=move || {
                        view! {
                            <div class="row-end forum-add">
                                <button type="button" class="btn tonal" on:click=move |_| adding.set(true)>
                                    <Ico icon=Icon::Add />
                                    {move || t!("forum-category-new")}
                                </button>
                            </div>
                        }
                    }
                >
                    <CategoryForm category=None on_done=Callback::new(move |()| adding.set(false)) />
                </Show>
            </Show>
        </Show>
    }
}

/// One category: a link to its threads, and for an administrator its tools.
#[component]
fn CategoryRow(
    id: i64,
    list: Memo<Option<Vec<ForumCategory>>>,
    admin: Memo<bool>,
    editing: RwSignal<Option<i64>>,
) -> impl IntoView {
    let category =
        Memo::new(move |_| list.with(|l| l.iter().flatten().find(|c| c.id == id).cloned()));
    let read = move |f: fn(&ForumCategory) -> String| {
        move || category.with(|c| c.as_ref().map(f)).unwrap_or_default()
    };
    let name = read(|c| c.name.clone());
    let description = read(|c| c.description.clone());
    let threads = move || category.with(|c| c.as_ref().map_or(0, |c| c.threads));
    let last = move || category.with(|c| c.as_ref().and_then(|c| c.last_activity_at));
    let open = move || editing.get() == Some(id);
    let busy = RwSignal::new(false);
    // Up or down a place, by trading positions with the neighbour.
    let shift = move |by: i32| {
        if busy.get_untracked() {
            return;
        }
        let Some(all) = list.get_untracked() else {
            return;
        };
        let Some(at) = all.iter().position(|c| c.id == id) else {
            return;
        };
        let Some(other) = isize::try_from(by)
            .ok()
            .and_then(|by| at.checked_add_signed(by))
            .and_then(|i| all.get(i))
            .cloned()
        else {
            return;
        };
        let Some(me) = all.get(at).cloned() else {
            return;
        };
        let (mine, theirs) = swapped(me.position, other.position, by < 0);
        busy.set(true);
        spawn_local(async move {
            let a = CategoryUpdate {
                position: Some(mine),
                ..CategoryUpdate::default()
            };
            let b = CategoryUpdate {
                position: Some(theirs),
                ..CategoryUpdate::default()
            };
            let first =
                api::call(move |c| async move { c.update_forum_category(me.id, &a).await }).await;
            let second = match first {
                Ok(_) => {
                    api::call(move |c| async move { c.update_forum_category(other.id, &b).await })
                        .await
                        .map(|_| ())
                }
                Err(e) => Err(e),
            };
            if let Err(message) = second {
                notify(message);
            }
            let _ = load_categories().await;
            busy.try_set(false);
        });
    };
    let first = move || list.with(|l| l.as_ref().and_then(|l| l.first()).map(|c| c.id)) == Some(id);
    let last_one =
        move || list.with(|l| l.as_ref().and_then(|l| l.last()).map(|c| c.id)) == Some(id);
    view! {
        <li class="category" class:editing=open>
            <a class="category-link" href=route::forum_category(id, "")>
                <span class="category-icon" aria-hidden="true">
                    <Ico icon=Icon::Forum />
                </span>
                <span class="category-text">
                    <strong class="break">{name}</strong>
                    <span class="muted break">{description}</span>
                </span>
                <span class="category-stats">
                    <span class="category-count">{move || t!("forum-threads", count = threads())}</span>
                    <span class="muted">
                        {move || match last() {
                            Some(at) => view! { <Ago at=at /> }.into_any(),
                            None => t!("forum-no-activity").into_any(),
                        }}
                    </span>
                </span>
            </a>
            <Show when=move || admin.get()>
                <button
                    type="button"
                    class="icon-btn small category-tools"
                    class:on=open
                    title=move || t!("forum-category-edit")
                    aria-label=move || t!("forum-category-edit-named", name = name())
                    aria-expanded=move || if open() { "true" } else { "false" }
                    on:click=move |_| editing.set(if open() { None } else { Some(id) })
                >
                    <Ico icon=Icon::Edit size=20 />
                </button>
            </Show>
            <Show when=open>
                <CategoryForm
                    category=category.get_untracked()
                    on_done=Callback::new(move |()| editing.set(None))
                    place=Place {
                        first: Signal::derive(first),
                        last: Signal::derive(last_one),
                        shift: Callback::new(shift),
                    }
                />
            </Show>
        </li>
    }
}

/// Where a category stands among the others, and the way to move it.
#[derive(Clone, Copy)]
struct Place {
    first: Signal<bool>,
    last: Signal<bool>,
    shift: Callback<i32>,
}

/// A category's name and description, new or to change, its place in the
/// list, and deleting an empty one.
#[component]
fn CategoryForm(
    category: Option<ForumCategory>,
    on_done: Callback<()>,
    #[prop(optional)] place: Option<Place>,
) -> impl IntoView {
    let id = category.as_ref().map(|c| c.id);
    let threads = category.as_ref().map_or(0, |c| c.threads);
    let name = RwSignal::new(
        category
            .as_ref()
            .map(|c| c.name.clone())
            .unwrap_or_default(),
    );
    let description = RwSignal::new(
        category
            .as_ref()
            .map(|c| c.description.clone())
            .unwrap_or_default(),
    );
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let tried = RwSignal::new(false);
    let checked = Memo::new(move |_| check_category(&name.read(), &description.read()));
    let save = move |e: SubmitEvent| {
        e.prevent_default();
        tried.set(true);
        let Ok((name, description)) = checked.get_untracked() else {
            return;
        };
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let result = match id {
                Some(id) => {
                    let change = CategoryUpdate {
                        name: Some(name),
                        description: Some(description),
                        position: None,
                    };
                    api::call(move |c| async move { c.update_forum_category(id, &change).await })
                        .await
                }
                None => {
                    let new = NewCategory {
                        name,
                        description,
                        position: None,
                    };
                    api::call(move |c| async move { c.create_forum_category(&new).await }).await
                }
            };
            busy.try_set(false);
            match result {
                Ok(_) => {
                    let _ = load_categories().await;
                    on_done.try_run(());
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    };
    let remove = Callback::new(move |()| {
        let Some(id) = id else {
            return;
        };
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let result = api::call(move |c| async move { c.delete_forum_category(id).await }).await;
            busy.try_set(false);
            match result {
                Ok(()) => {
                    let _ = load_categories().await;
                    notify(untrack(|| t!("forum-category-deleted")));
                    on_done.try_run(());
                }
                Err(message) => {
                    error.try_set(Some(message));
                }
            }
        });
    });
    let problem = move || {
        tried
            .get()
            .then(|| checked.with(|c| c.as_ref().err().map(error_text)))
            .flatten()
    };
    view! {
        <form class="category-form" on:submit=save>
            <label class="field">
                <span>{move || t!("forum-category-name")}</span>
                <input maxlength=MAX_CATEGORY_NAME_CHARS bind:value=name />
            </label>
            <label class="field">
                <span>{move || t!("forum-category-description")}</span>
                <textarea
                    rows="2"
                    maxlength=MAX_CATEGORY_DESCRIPTION_CHARS
                    prop:value=move || description.get()
                    on:input=move |e| description.set(event_target_value(&e))
                ></textarea>
            </label>
            {place
                .map(|p| {
                    view! {
                        <div class="field category-place">
                            <span>{move || t!("forum-category-place")}</span>
                            <div class="category-moves">
                                <button
                                    type="button"
                                    class="btn tonal small"
                                    disabled=move || p.first.get()
                                    on:click=move |_| p.shift.run(-1)
                                >
                                    <Ico icon=Icon::ChevronUp size=18 />
                                    {move || t!("forum-category-up")}
                                </button>
                                <button
                                    type="button"
                                    class="btn tonal small"
                                    disabled=move || p.last.get()
                                    on:click=move |_| p.shift.run(1)
                                >
                                    <Ico icon=Icon::ChevronDown size=18 />
                                    {move || t!("forum-category-down")}
                                </button>
                            </div>
                        </div>
                    }
                })}
            {move || problem().map(|p| view! { <p class="field-error">{p}</p> })}
            {move || error.get().map(|m| view! { <ErrorBanner message=m /> })}
            <div class="row-end">
                {id.map(|_| {
                    if threads == 0 {
                        view! {
                            <ConfirmButton
                                icon=Icon::Delete
                                title=Signal::derive(|| t!("forum-category-delete"))
                                label=Signal::derive(|| t!("forum-category-delete"))
                                text=Signal::derive(|| t!("forum-category-delete"))
                                small=true
                                disabled=busy
                                on_confirm=remove
                            />
                        }
                            .into_any()
                    } else {
                        view! { <small class="muted category-full">{move || t!("forum-category-not-empty")}</small> }
                            .into_any()
                    }
                })}
                <button type="button" class="btn text" on:click=move |_| on_done.run(())>
                    {move || t!("cancel")}
                </button>
                <button type="submit" class="btn" disabled=move || busy.get()>
                    {move || if id.is_some() { t!("admin-save") } else { t!("forum-category-add") }}
                </button>
            </div>
        </form>
    }
}
