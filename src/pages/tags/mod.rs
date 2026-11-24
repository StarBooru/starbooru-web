//! The Tags page at `/tags`: the collection's vocabulary as cards, each with
//! a picture of a post that has the tag, its name in its category's colour,
//! how many posts have it, and the ways to search it and change its
//! category. The search box filters it (`?q=`). Each tag's own page is in
//! `tag.rs`.

mod tag;

use std::cell::RefCell;
use std::collections::HashSet;

use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_query_map;
use starbooru_core::{Tag, TagCategory, TagOrder, TagUpdate};
use wasm_bindgen::JsCast;

pub use self::tag::TagPage;
use super::{CATEGORIES, category_label};
use crate::api;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::components::menu::{self, ActionMenu, MenuAt};
use crate::components::preview::SearchPreview;
use crate::components::snackbar::{notify, notify_undo};
use crate::components::tag_menu::TagPress;
use crate::dom;
use crate::logic::search::{category_class, short_count};
use crate::logic::tags::{TAG_PAGE, go_on, may_edit_tag, shows, tag_list_request};
use crate::route;
use crate::store;

/// What the Tags page showed, so coming back to it shows the same. The
/// filter is in the address, from the search box.
#[derive(Clone, Default)]
struct Shown {
    order: TagOrder,
    category: Option<TagCategory>,
}

thread_local! {
    static SHOWN: RefCell<Shown> = RefCell::new(Shown::default());
}

fn sort_label(order: TagOrder) -> String {
    match order {
        TagOrder::Count => t!("tags-most-used"),
        TagOrder::Name => t!("tags-a-z"),
    }
}

/// The class that colours a tag's card, dot and bar by its category.
pub(crate) fn tint_class(category: TagCategory) -> &'static str {
    match category {
        TagCategory::General => "cat-general",
        other => category_class(other),
    }
}

/// Gives tag `name` category `to`, puts the answer in `list`, and says so
/// with a way back to `was`.
fn set_category(name: String, to: TagCategory, was: Option<TagCategory>, list: RwSignal<Vec<Tag>>) {
    spawn_local(async move {
        let update = TagUpdate {
            category: Some(to),
            parent: None,
        };
        let asked = name.clone();
        match api::call(move |c| async move { c.update_tag(&asked, &update).await }).await {
            Ok(tag) => {
                api::forget_tags();
                list.try_update(|l| {
                    if let Some(t) = l.iter_mut().find(|t| t.id == tag.id) {
                        *t = tag;
                    }
                });
                refocus(&name);
                let said = t!(
                    "tags-recoloured",
                    name = name.clone(),
                    category = category_label(to)
                );
                match was {
                    Some(was) => notify_undo(said, t!("press-undo"), move || {
                        set_category(name.clone(), was, None, list);
                    }),
                    None => notify(said),
                }
            }
            Err(message) => notify(t!("tags-recolour-failed", reason = message)),
        }
    });
}

/// Its card comes again in the new colour: the focus goes to its category
/// button, if it was on the one that left.
fn refocus(name: &str) {
    let selector = format!(r#".tag-card:has(a[data-tag="{name}"]) .cat-badge"#);
    request_animation_frame(move || {
        let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
            return;
        };
        let lost = doc.active_element().is_none_or(|el| {
            doc.body()
                .is_some_and(|b| AsRef::<web_sys::Element>::as_ref(&b) == &el)
        });
        if lost
            && let Some(button) = doc
                .query_selector(&selector)
                .ok()
                .flatten()
                .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let _ = button.focus();
        }
    });
}

/// One tag: its picture, its name and how many posts have it, leading to
/// its page, and its category and search beside them.
fn card(
    tag: Tag,
    index: usize,
    editable: Signal<bool>,
    on_category: Callback<(Tag, MenuAt)>,
) -> impl IntoView {
    let Tag {
        name,
        category,
        post_count,
        ..
    } = tag.clone();
    let count = u64::try_from(post_count).unwrap_or(0);
    let label = t!("search-hint", tag = name.clone(), count = count);
    let style = format!("--i: {}", index % TAG_PAGE as usize);
    let search_label = t!("tags-search-named", name = name.clone());
    let kind_label = {
        let name = name.clone();
        move || {
            t!(
                "tags-category-of",
                name = name.clone(),
                category = category_label(category)
            )
        }
    };
    let q = name.clone();
    let unused = post_count <= 0;
    view! {
        <li class=format!("tag-card {}", tint_class(category)) style=style>
            <a class="tag-main" href=route::tag(&name) data-tag=name.clone() aria-label=label>
                <SearchPreview q=q nothing=unused />
                <span class="tag-text">
                    <span class="tag-name">{name.clone()}</span>
                    <span class="tag-count" title=count.to_string()>
                        {move || t!("tags-posts", count = count, shown = short_count(count))}
                    </span>
                </span>
            </a>
            <div class="tag-actions">
                {move || {
                    let kind = view! {
                        <span class="cat-dot" aria-hidden="true"></span>
                        <span class="cat-name">{move || category_label(category)}</span>
                    };
                    if editable.get() {
                        let tag = tag.clone();
                        view! {
                            <button
                                type="button"
                                class="cat-badge"
                                aria-haspopup="menu"
                                title=move || t!("tags-change-category")
                                aria-label=kind_label.clone()
                                on:click=move |e| {
                                    if let Some(el) = e
                                        .current_target()
                                        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                                    {
                                        on_category.run((tag.clone(), menu::beside(&el)));
                                    }
                                }
                            >
                                {kind}
                            </button>
                        }
                            .into_any()
                    } else {
                        view! { <span class="cat-badge">{kind}</span> }.into_any()
                    }
                }}
                <a
                    class="icon-btn"
                    href=route::gallery(&name)
                    title=search_label.clone()
                    aria-label=search_label
                >
                    <Ico icon=Icon::Gallery />
                </a>
            </div>
        </li>
    }
}

#[component]
pub fn Tags() -> impl IntoView {
    let was = SHOWN.with_borrow(Clone::clone);
    // The search box filters the page (`shell.rs`), through the address.
    let query = use_query_map();
    let filter =
        Memo::new(move |_| query.with(|q| q.get("q").unwrap_or_default().trim().to_owned()));
    let order = RwSignal::new(was.order);
    let category = RwSignal::new(was.category);
    let tags = RwSignal::new(Vec::<Tag>::new());
    let next_page = StoredValue::new(1_u32);
    let has_more = RwSignal::new(true);
    let loading = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let generation = StoredValue::new(0_u64);
    // Pages in a row with nothing of the chosen category, and whether the
    // list still goes on by itself while its end is in sight.
    let quiet = StoredValue::new(0_u32);
    let going = RwSignal::new(true);
    let sentinel: NodeRef<html::Div> = NodeRef::new();
    let user = Memo::new(|_| store::session().map(|s| s.user));
    let editable = Signal::derive(move || user.with(|u| may_edit_tag(u.as_ref())));
    let menu = RwSignal::new(None::<(Tag, MenuAt)>);

    Effect::new(move |_| {
        let now = Shown {
            order: order.get(),
            category: category.get(),
        };
        SHOWN.with_borrow_mut(|s| *s = now);
    });

    let load_more = move || {
        if loading.get_untracked()
            || !has_more.get_untracked()
            || error.with_untracked(Option::is_some)
        {
            return;
        }
        let at = generation.get_value();
        let request = tag_list_request(
            &filter.get_untracked(),
            order.get_untracked(),
            next_page.get_value(),
        );
        let page = request.page;
        loading.set(true);
        spawn_local(async move {
            let result = api::call(move |c| async move {
                c.tag_list(&request.prefix, request.order, request.page, request.limit)
                    .await
            })
            .await;
            if generation.try_get_value() != Some(at) {
                return;
            }
            loading.set(false);
            match result {
                Ok(found) => {
                    let wanted = category.get_untracked();
                    let shown = found.iter().filter(|t| shows(t, wanted)).count();
                    let (now, on) = go_on(quiet.get_value(), shown);
                    quiet.set_value(now);
                    going.set(on);
                    has_more.set(u32::try_from(found.len()).unwrap_or(u32::MAX) >= TAG_PAGE);
                    next_page.set_value(page.saturating_add(1));
                    // A tag that moved up a page since is there once.
                    tags.update(|list| {
                        let seen: HashSet<i64> = list.iter().map(|t| t.id).collect();
                        list.extend(found.into_iter().filter(|t| !seen.contains(&t.id)));
                    });
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };
    let restart = move || {
        generation.update_value(|g| *g = g.wrapping_add(1));
        tags.set(Vec::new());
        next_page.set_value(1);
        has_more.set(true);
        loading.set(false);
        error.set(None);
        quiet.set_value(0);
        going.set(true);
        load_more();
    };
    Effect::watch(
        move || (filter.get(), order.get()),
        move |_, _, _| restart(),
        true,
    );

    // The end of the list in sight, after a page that filled too little of
    // the screen: the next one comes by itself.
    let near_end = move || {
        sentinel.get_untracked().is_some_and(|el| {
            let (_, tall) = dom::scroll();
            el.get_bounding_client_rect().top() < tall * 2.5
        })
    };
    Effect::new(move |_| {
        let ready = going.get() && has_more.get() && !loading.get() && error.with(Option::is_none);
        tags.track();
        category.track();
        if ready && near_end() {
            load_more();
        }
    });
    let watch = StoredValue::new(None::<dom::Watch>);
    sentinel.on_load(move |el| {
        let w = dom::on_near(&el, "0px 0px 150% 0px", move || {
            if going.try_get_untracked() == Some(true) {
                load_more();
            }
        });
        watch.set_value(Some(w));
    });
    on_cleanup(move || {
        if let Some(w) = watch.try_update_value(Option::take).flatten() {
            w.stop();
        }
    });

    let pick = move |wanted: Option<TagCategory>| {
        if category.get_untracked() != wanted {
            category.set(wanted);
            quiet.set_value(0);
            going.set(true);
        }
    };
    let more = move || {
        quiet.set_value(0);
        going.set(true);
        load_more();
    };
    let retry = Callback::new(move |()| {
        error.set(None);
        more();
    });

    // A tag's category changes in place, and its card comes again in its
    // new colour.
    let visible = move || {
        let wanted = category.get();
        tags.with(|list| {
            list.iter()
                .filter(|t| shows(t, wanted))
                .cloned()
                .enumerate()
                .collect::<Vec<_>>()
        })
    };
    let none_shown = move || {
        let wanted = category.get();
        tags.with(|list| !list.iter().any(|t| shows(t, wanted)))
    };
    let empty =
        move || !loading.get() && !has_more.get() && error.with(Option::is_none) && none_shown();
    let sorts = [TagOrder::Count, TagOrder::Name].map(|o| {
        let on = move || order.get() == o;
        view! {
            <button
                type="button"
                role="radio"
                class="seg"
                class:on=on
                aria-checked=move || if on() { "true" } else { "false" }
                on:click=move |_| order.set(o)
            >
                {move || sort_label(o)}
            </button>
        }
    });
    let kinds = CATEGORIES.map(|c| {
        let on = move || category.get() == Some(c);
        view! {
            <button
                type="button"
                role="radio"
                class=format!("cat-tab {}", tint_class(c))
                class:on=on
                aria-checked=move || if on() { "true" } else { "false" }
                on:click=move |_| pick(Some(c))
            >
                <span class="cat-dot" aria-hidden="true"></span>
                {move || category_label(c)}
            </button>
        }
    });
    let on_category = Callback::new(move |at: (Tag, MenuAt)| menu.set(Some(at)));
    let category_menu = move || {
        let (tag, at) = menu.get()?;
        let close = Callback::new(move |()| menu.set(None));
        let items = CATEGORIES
            .map(|c| {
                let on = tag.category == c;
                let (name, was) = (tag.name.clone(), tag.category);
                view! {
                    <button
                        type="button"
                        role="menuitemradio"
                        aria-checked=if on { "true" } else { "false" }
                        class=format!("menu-item cat-pick {}", tint_class(c))
                        on:click=move |_| {
                            menu.set(None);
                            if !on {
                                set_category(name.clone(), c, Some(was), tags);
                            }
                        }
                    >
                        <span class="cat-dot" aria-hidden="true"></span>
                        <span>{move || category_label(c)}</span>
                        {on.then(|| view! { <Ico icon=Icon::Check size=20 /> })}
                    </button>
                }
            })
            .collect_view();
        Some(view! {
            <ActionMenu at=at title=tag.name.clone() on_close=close>
                {items}
            </ActionMenu>
        })
    };

    view! {
        <div class="page tags-page">
            <div class="page-head">
                <h1 class="page-title">{move || t!("nav-tags")}</h1>
                <div class="segmented" role="radiogroup" aria-label=move || t!("tags-sort")>
                    {sorts}
                </div>
            </div>
            <div class="cat-tabs" role="radiogroup" aria-label=move || t!("tags-category")>
                <button
                    type="button"
                    role="radio"
                    class="cat-tab cat-all"
                    class:on=move || category.get().is_none()
                    aria-checked=move || if category.get().is_none() { "true" } else { "false" }
                    on:click=move |_| pick(None)
                >
                    <span class="cat-dot" aria-hidden="true"></span>
                    {move || t!("tags-any-category")}
                </button>
                {kinds}
            </div>
            {move || error.get().map(|message| view! { <ErrorBanner message=message on_retry=retry /> })}
            <Show when=empty>
                <div class="card empty">
                    <Ico icon=Icon::Label size=48 />
                    <h2>
                        {move || {
                            let all = filter.with(String::is_empty) && category.get().is_none();
                            if all { t!("tags-none-yet") } else { t!("tags-none-match") }
                        }}
                    </h2>
                    <p class="muted">{move || t!("tags-appear")}</p>
                    <Show when=move || filter.with(|f| !f.is_empty())>
                        <a class="btn tonal" href=route::TAGS>
                            {move || t!("tags-show-all")}
                        </a>
                    </Show>
                </div>
            </Show>
            <Show when=move || !none_shown()>
                <ul class="tag-grid" data-press="tag">
                    <For
                        each=visible
                        key=|(_, t)| (t.id, t.category)
                        children=move |(i, t)| card(t, i, editable, on_category)
                    />
                </ul>
            </Show>
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
                            <button type="button" class="btn tonal" on:click=move |_| more()>
                                {move || t!("load-more")}
                            </button>
                        }
                            .into_any()
                    }
                }}
            </div>
            {category_menu}
            <TagPress q="" />
        </div>
    }
}
