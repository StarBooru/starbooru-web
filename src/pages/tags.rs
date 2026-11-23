//! The Tags page at `/tags`, every tag in use with its category and how many
//! posts have it, and each tag's own page at `/tag?name=…`: its category,
//! its place in the tree, its aliases and what it implies.

use std::cell::RefCell;
use std::collections::HashSet;

use leptos::ev::SubmitEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_query_map};
use send_wrapper::SendWrapper;
use starbooru_core::{Tag, TagCategory, TagDetail, TagOrder, TagUpdate};

use super::{CATEGORIES, category_label, gallery};
use crate::api::{self, Failure};
use crate::components::back::BackButton;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::components::tag_menu::TagPress;
use crate::dom;
use crate::logic::search::{Latest, TYPING_PAUSE_MS, category_class, short_count};
use crate::logic::tags::{
    TAG_PAGE, by_alias, go_on, may_edit_tag, may_relate_tags, merges, shows, tag_list_request,
};
use crate::logic::viewer::category_class as chip_class;
use crate::route;
use crate::store;

// ---------- the Tags page ----------

/// What the Tags page showed, so coming back to it shows the same.
#[derive(Clone, Default)]
struct Shown {
    filter: String,
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

/// One tag of the list, leading to its page.
fn row(tag: Tag) -> impl IntoView {
    let Tag {
        name,
        category,
        post_count,
        ..
    } = tag;
    let count = u64::try_from(post_count).unwrap_or(0);
    let label = t!("search-hint", tag = name.clone(), count = count);
    let href = route::tag(&name);
    let key = name.clone();
    view! {
        <li>
            <a class="tag-row" href=href data-tag=key aria-label=label>
                <span class=format!("tag-name {}", category_class(category))>{name}</span>
                <span class="pill">{move || category_label(category)}</span>
                <span class="count" title=count.to_string()>
                    {short_count(count)}
                </span>
            </a>
        </li>
    }
}

#[component]
pub fn Tags() -> impl IntoView {
    let was = SHOWN.with_borrow(Clone::clone);
    // What's in the box, and what the list is for once the typing pauses.
    let typed = RwSignal::new(was.filter.clone());
    let filter = RwSignal::new(was.filter);
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
    let pause = StoredValue::new(Latest::default());
    let sentinel: NodeRef<html::Div> = NodeRef::new();

    Effect::new(move |_| {
        let now = Shown {
            filter: filter.get(),
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

    let on_input = move |e: leptos::ev::Event| {
        let value = event_target_value(&e);
        typed.set(value.clone());
        let Some(n) = pause.try_update_value(Latest::next) else {
            return;
        };
        spawn_local(async move {
            dom::sleep(TYPING_PAUSE_MS).await;
            let latest = pause.try_with_value(|p| p.is(n)) == Some(true);
            if latest && filter.try_get_untracked().is_some_and(|f| f != value) {
                filter.set(value);
            }
        });
    };
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

    let visible = move || {
        let wanted = category.get();
        tags.with(|list| {
            list.iter()
                .filter(|t| shows(t, wanted))
                .cloned()
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
                class=chip_class(c)
                class:on=on
                aria-checked=move || if on() { "true" } else { "false" }
                on:click=move |_| pick(Some(c))
            >
                {move || category_label(c)}
            </button>
        }
    });

    view! {
        <div class="page tags-page">
            <div class="page-head">
                <h1 class="page-title">{move || t!("nav-tags")}</h1>
                <div class="segmented" role="radiogroup" aria-label=move || t!("tags-sort")>
                    {sorts}
                </div>
            </div>
            <label class="field tag-filter">
                <span class="visually-hidden">{move || t!("tags-filter-label")}</span>
                <input
                    type="search"
                    autocomplete="off"
                    spellcheck="false"
                    autocapitalize="none"
                    placeholder=move || t!("tags-filter-placeholder")
                    prop:value=move || typed.get()
                    on:input=on_input
                />
            </label>
            <div class="chips tag-kinds" role="radiogroup" aria-label=move || t!("tags-category")>
                <button
                    type="button"
                    role="radio"
                    class="chip"
                    class:on=move || category.get().is_none()
                    aria-checked=move || if category.get().is_none() { "true" } else { "false" }
                    on:click=move |_| pick(None)
                >
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
                            let all = filter.with(|f| f.trim().is_empty()) && category.get().is_none();
                            if all { t!("tags-none-yet") } else { t!("tags-none-match") }
                        }}
                    </h2>
                    <p class="muted">{move || t!("tags-appear")}</p>
                </div>
            </Show>
            <Show when=move || !none_shown()>
                <ul class="tag-list card" data-press="tag">
                    <For each=visible key=|t| t.id children=row />
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
            <TagPress q="" />
        </div>
    }
}

// ---------- a tag's page ----------

/// Which of a tag's lists a change is to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Relation {
    Alias,
    Implies,
}

/// A change made on a tag's page.
#[derive(Clone, Debug)]
enum Change {
    Category(TagCategory),
    /// The parent's name; empty for the top of the tree.
    Parent(String),
    Add(Relation, String),
    Remove(Relation, String),
}

/// A name typed to add to one of the lists, and what empties its box once
/// the server has it.
type Adding = (Relation, String, Callback<()>);

/// What the server has after a change: the tag alone, or all of the page.
enum Changed {
    Tag(Tag),
    Detail(TagDetail),
}

/// Makes `change` to the tag `name`.
async fn apply(name: String, change: Change) -> Result<Changed, String> {
    match change {
        Change::Category(category) => {
            let update = TagUpdate {
                category: Some(category),
                parent: None,
            };
            api::call(move |c| async move { c.update_tag(&name, &update).await })
                .await
                .map(Changed::Tag)
        }
        Change::Parent(parent) => {
            let update = TagUpdate {
                category: None,
                parent: Some(parent),
            };
            // The answer is the tag alone; the page shows its whole tree.
            api::call(move |c| async move {
                c.update_tag(&name, &update).await?;
                c.tag(&name).await
            })
            .await
            .map(Changed::Detail)
        }
        Change::Add(Relation::Alias, alias) => {
            api::call(move |c| async move { c.add_alias(&name, &alias).await })
                .await
                .map(Changed::Detail)
        }
        Change::Add(Relation::Implies, implied) => {
            api::call(move |c| async move { c.add_implication(&name, &implied).await })
                .await
                .map(Changed::Detail)
        }
        Change::Remove(Relation::Alias, alias) => {
            api::call(move |c| async move { c.remove_alias(&name, &alias).await })
                .await
                .map(Changed::Detail)
        }
        Change::Remove(Relation::Implies, implied) => {
            api::call(move |c| async move { c.remove_implication(&name, &implied).await })
                .await
                .map(Changed::Detail)
        }
    }
}

#[component]
pub fn TagPage() -> impl IntoView {
    let query = use_query_map();
    let name = Memo::new(move |_| query.with(|q| q.get("name").unwrap_or_default()));
    let detail = RwSignal::new(None::<TagDetail>);
    let failure = RwSignal::new(None::<Failure>);
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    // An alias that is a tag already, and its posts: merging it in waits for
    // a yes, as it can't be taken back. Its box keeps the name until then.
    let merging = RwSignal::new(None::<(String, i32)>);
    let merge_done = StoredValue::new(None::<Callback<()>>);
    let generation = StoredValue::new(0_u64);
    let user = Memo::new(|_| store::session().map(|s| s.user));
    let editable = Signal::derive(move || user.with(|u| may_edit_tag(u.as_ref())));
    let relatable = Signal::derive(move || user.with(|u| may_relate_tags(u.as_ref())));

    let load = move |wanted: String| {
        generation.update_value(|g| *g = g.wrapping_add(1));
        let at = generation.get_value();
        detail.set(None);
        failure.set(None);
        error.set(None);
        merging.set(None);
        merge_done.set_value(None);
        busy.set(false);
        spawn_local(async move {
            let result = api::fetch(move |c| async move { c.tag(&wanted).await }).await;
            if generation.try_get_value() != Some(at) {
                return;
            }
            match result {
                Ok(d) => detail.set(Some(d)),
                Err(f) => failure.set(Some(f)),
            }
        });
    };
    let navigate = SendWrapper::new(use_navigate());
    Effect::watch(
        move || name.get(),
        move |wanted, _, _| {
            if wanted.trim().is_empty() {
                let options = NavigateOptions {
                    replace: true,
                    ..NavigateOptions::default()
                };
                (*navigate)(route::TAGS, options);
            } else {
                load(wanted.clone());
            }
        },
        true,
    );

    // `done` hears when the change is saved: the box it was typed in empties
    // then, and keeps the text when it fails.
    let act = move |change: Change, done: Option<Callback<()>>| {
        let Some(current) = detail.with_untracked(|d| d.as_ref().map(|d| d.tag.name.clone()))
        else {
            return;
        };
        if busy.get_untracked() {
            return;
        }
        // A category changes no search; the rest move posts in or out of some.
        let moves_posts = !matches!(change, Change::Category(_));
        busy.set(true);
        error.set(None);
        let at = generation.get_value();
        spawn_local(async move {
            let result = apply(current, change).await;
            if generation.try_get_value() != Some(at) {
                return;
            }
            busy.set(false);
            match result {
                Ok(changed) => {
                    api::forget_tags();
                    if moves_posts {
                        gallery::forget_lists();
                    }
                    match changed {
                        Changed::Tag(tag) => detail.update(|d| {
                            if let Some(d) = d {
                                d.tag = tag;
                            }
                        }),
                        Changed::Detail(d) => detail.set(Some(d)),
                    }
                    if let Some(done) = done {
                        done.try_run(());
                    }
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };
    // An alias that names a tag of its own merges that tag in: ask first.
    let add = Callback::new(move |(relation, value, done): Adding| {
        let Some(this) = detail.with_untracked(|d| d.as_ref().map(|d| d.tag.id)) else {
            return;
        };
        if relation == Relation::Implies {
            act(Change::Add(relation, value), Some(done));
            return;
        }
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        let at = generation.get_value();
        spawn_local(async move {
            let asked = value.clone();
            let found = api::fetch(move |c| async move { c.tag(&asked).await }).await;
            if generation.try_get_value() != Some(at) {
                return;
            }
            busy.set(false);
            match found {
                Ok(other) if merges(&other.tag, &value, this) => {
                    merge_done.set_value(Some(done));
                    merging.set(Some((value, other.tag.post_count)));
                }
                Ok(_) | Err(Failure { missing: true, .. }) => {
                    act(Change::Add(Relation::Alias, value), Some(done));
                }
                Err(f) => error.set(Some(f.message)),
            }
        });
    });
    let remove = Callback::new(move |(relation, value): (Relation, String)| {
        act(Change::Remove(relation, value), None);
    });
    let set_parent = Callback::new(move |(parent, done): (String, Option<Callback<()>>)| {
        act(Change::Parent(parent), done);
    });

    let failed = move || {
        let f = failure.get()?;
        let wanted = name.get();
        Some(if f.missing {
            view! {
                <div class="card empty">
                    <Ico icon=Icon::Label size=48 />
                    <h2 class="break">{move || t!("tags-missing", name = wanted.clone())}</h2>
                    <a class="btn tonal" href=route::TAGS>
                        {move || t!("tags-all")}
                    </a>
                </div>
            }
            .into_any()
        } else {
            let retry = Callback::new(move |()| load(name.get_untracked()));
            view! { <ErrorBanner message=f.message on_retry=retry /> }.into_any()
        })
    };

    let have = Memo::new(move |_| detail.with(Option::is_some));
    let tag = Memo::new(move |_| detail.with(|d| d.as_ref().map(|d| d.tag.clone())));
    let aliases = Signal::derive(move || {
        detail.with(|d| d.as_ref().map(|d| d.aliases.clone()).unwrap_or_default())
    });
    let implies = Signal::derive(move || {
        detail.with(|d| {
            d.as_ref()
                .map(|d| d.implies.iter().map(|t| t.name.clone()).collect())
                .unwrap_or_default()
        })
    });
    let page = move || {
        let head = move || {
            tag.get().map(|t| {
                let browse = route::gallery(&t.name);
                let count = t.post_count;
                view! {
                    <div class="page-head">
                        <div class="titled">
                            <BackButton up=route::TAGS />
                            <h1 class="page-title break">{t.name.clone()}</h1>
                        </div>
                        <a class="btn tonal" href=browse>
                            <Ico icon=Icon::Gallery />
                            {move || t!("tags-browse", count = count)}
                        </a>
                    </div>
                }
            })
        };
        let alias_note = move || {
            let shown = tag.with(|t| t.as_ref().map(|t| t.name.clone()))?;
            let asked = name.get();
            by_alias(&asked, &shown).then(|| {
                view! {
                    <div class="banner info" role="status">
                        <span class="break">{move || t!("tags-is-alias", name = asked.clone())}</span>
                    </div>
                }
            })
        };
        let category_chips = move || {
            let now = tag.with(|t| t.as_ref().map(|t| t.category));
            CATEGORIES
                .map(|c| {
                    let on = now == Some(c);
                    view! {
                        <button
                            type="button"
                            role="radio"
                            class="chip"
                            class:on=on
                            aria-checked=if on { "true" } else { "false" }
                            disabled=move || busy.get() || !editable.get()
                            on:click=move |_| {
                                if !on {
                                    act(Change::Category(c), None);
                                }
                            }
                        >
                            {move || category_label(c)}
                        </button>
                    }
                })
                .collect_view()
        };
        let confirm = move || {
            merging.get().map(|(alias, count)| {
                let merged = alias.clone();
                view! {
                    <div class="banner warn merge-confirm" role="alert">
                        <span class="break">
                            {move || t!("tags-merge-confirm", name = alias.clone(), count = count)}
                        </span>
                        <div class="row-end">
                            <button
                                type="button"
                                class="btn text"
                                on:click=move |_| {
                                    merging.set(None);
                                    merge_done.set_value(None);
                                }
                            >
                                {move || t!("cancel")}
                            </button>
                            <button
                                type="button"
                                class="btn danger"
                                on:click=move |_| {
                                    merging.set(None);
                                    let done = merge_done.try_update_value(Option::take).flatten();
                                    act(Change::Add(Relation::Alias, merged.clone()), done);
                                }
                            >
                                {move || t!("tags-merge")}
                            </button>
                        </div>
                    </div>
                }
            })
        };
        let implied_by = move || {
            let list =
                detail.with(|d| d.as_ref().map(|d| d.implied_by.clone()).unwrap_or_default());
            if list.is_empty() {
                return view! { <p class="muted">{move || t!("tags-implied-by-none")}</p> }
                    .into_any();
            }
            let chips = list
                .into_iter()
                .map(|t| tag_link(t.name, "chip"))
                .collect_view();
            view! { <div class="chips">{chips}</div> }.into_any()
        };
        view! {
            {head}
            {alias_note}
            {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
            <div class="tag-cards">
                <section class="card">
                    <h2 class="card-title">{move || t!("tags-category")}</h2>
                    <div class="chips" role="radiogroup" aria-label=move || t!("tags-category")>
                        {category_chips}
                    </div>
                </section>
                <TreeCard detail=detail busy=busy editable=editable on_parent=set_parent />
                <RelationCard
                    relation=Relation::Alias
                    names=aliases
                    busy=busy
                    may=relatable
                    on_add=add
                    on_remove=remove
                >
                    {confirm}
                </RelationCard>
                <RelationCard
                    relation=Relation::Implies
                    names=implies
                    busy=busy
                    may=relatable
                    on_add=add
                    on_remove=remove
                />
                <section class="card">
                    <h2 class="card-title">{move || t!("tags-implied-by")}</h2>
                    <p class="muted hint">{move || t!("tags-implied-by-hint")}</p>
                    {implied_by}
                </section>
            </div>
        }
    };

    view! {
        <div class="page tag-page">
            {move || {
                if have.get() {
                    page().into_any()
                } else if failure.with(Option::is_some) {
                    failed().into_any()
                } else {
                    view! {
                        <div class="center-page">
                            <Spinner />
                        </div>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

/// A chip leading to the tag's page.
fn tag_link(name: String, class: &'static str) -> impl IntoView {
    let href = route::tag(&name);
    view! {
        <a class=class href=href>
            {name}
        </a>
    }
}

/// A box and a button that hands what was typed on, with a way to empty the
/// box once it's saved. Until then the text stays, to mend or try again.
#[component]
fn InlineAdd(
    #[prop(into)] label: Signal<String>,
    #[prop(into)] button: Signal<String>,
    #[prop(optional)] icon: Option<Icon>,
    busy: RwSignal<bool>,
    on_add: Callback<(String, Callback<()>)>,
) -> impl IntoView {
    let text = RwSignal::new(String::new());
    let saved = Callback::new(move |()| text.set(String::new()));
    let submit = move |e: SubmitEvent| {
        e.prevent_default();
        let value = text.get_untracked().trim().to_owned();
        if !value.is_empty() && !busy.get_untracked() {
            on_add.run((value, saved));
        }
    };
    view! {
        <form class="inline-add" on:submit=submit>
            <input
                aria-label=move || label.get()
                placeholder=move || label.get()
                autocomplete="off"
                autocapitalize="none"
                spellcheck="false"
                prop:value=move || text.get()
                on:input=move |e| text.set(event_target_value(&e))
            />
            <button
                class="btn tonal small"
                type="submit"
                disabled=move || busy.get() || text.with(|t| t.trim().is_empty())
            >
                {icon.map(|icon| view! { <Ico icon=icon /> })}
                {move || button.get()}
            </button>
        </form>
    }
}

/// Where the tag sits: the tags above it, a way to move it, and the tags
/// under it.
#[component]
fn TreeCard(
    detail: RwSignal<Option<TagDetail>>,
    busy: RwSignal<bool>,
    editable: Signal<bool>,
    on_parent: Callback<(String, Option<Callback<()>>)>,
) -> impl IntoView {
    let trail = move || {
        let (here, above) = detail.with(|d| {
            d.as_ref()
                .map(|d| (d.tag.name.clone(), d.ancestors.clone()))
                .unwrap_or_default()
        });
        if above.is_empty() {
            return view! { <p class="muted">{move || t!("tags-tree-top")}</p> }.into_any();
        }
        let steps = above
            .into_iter()
            .map(|a| {
                view! {
                    {tag_link(a.name, "chip")}
                    <span class="sep" aria-hidden="true">
                        "›"
                    </span>
                }
            })
            .collect_view();
        view! {
            <nav class="tag-trail" aria-label=move || t!("tags-tree")>
                {steps}
                <span class="here" aria-current="page">
                    {here}
                </span>
                <Show when=move || editable.get()>
                    <button
                        class="btn text small"
                        type="button"
                        disabled=move || busy.get()
                        on:click=move |_| on_parent.run((String::new(), None))
                    >
                        {move || t!("tags-move-top")}
                    </button>
                </Show>
            </nav>
        }
        .into_any()
    };
    let children = move || {
        let under = detail.with(|d| d.as_ref().map(|d| d.children.clone()).unwrap_or_default());
        if under.is_empty() {
            return view! { <p class="muted">{move || t!("tags-children-none")}</p> }.into_any();
        }
        let chips = under
            .into_iter()
            .map(|c| tag_link(c.name, chip_class(c.category)))
            .collect_view();
        view! { <div class="chips">{chips}</div> }.into_any()
    };
    view! {
        <section class="card">
            <h2 class="card-title">{move || t!("tags-tree")}</h2>
            <p class="muted hint">{move || t!("tags-tree-hint")}</p>
            {trail}
            <Show when=move || editable.get()>
                <InlineAdd
                    label=Signal::derive(move || t!("tags-parent"))
                    button=Signal::derive(move || t!("tags-move"))
                    busy=busy
                    on_add=Callback::new(move |(parent, done): (String, Callback<()>)| {
                        on_parent.run((parent, Some(done)));
                    })
                />
            </Show>
            <h3 class="sub-title">{move || t!("tags-children")}</h3>
            {children}
        </section>
    }
}

/// The tag's aliases or the tags it implies, with ways to add and remove
/// them for those who `may`. `children` go under the box.
#[component]
fn RelationCard(
    relation: Relation,
    names: Signal<Vec<String>>,
    busy: RwSignal<bool>,
    may: Signal<bool>,
    on_add: Callback<Adding>,
    on_remove: Callback<(Relation, String)>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let title = move || match relation {
        Relation::Alias => t!("tags-aliases"),
        Relation::Implies => t!("tags-implies"),
    };
    let hint = move || match relation {
        Relation::Alias => t!("tags-aliases-hint"),
        Relation::Implies => t!("tags-implies-hint"),
    };
    let chips = move || {
        let list = names.get();
        if list.is_empty() {
            let none = move || match relation {
                Relation::Alias => t!("tags-aliases-none"),
                Relation::Implies => t!("tags-implies-none"),
            };
            return view! { <p class="muted">{none}</p> }.into_any();
        }
        let chips = list
            .into_iter()
            .map(|n| {
                // An alias is only another name; an implied tag has a page.
                let shown = match relation {
                    Relation::Alias => view! { <span>{n.clone()}</span> }.into_any(),
                    Relation::Implies => {
                        view! { <a href=route::tag(&n)>{n.clone()}</a> }.into_any()
                    }
                };
                let gone = n.clone();
                let named = n.clone();
                view! {
                    <span class="chip removable">
                        {shown}
                        <Show when=move || may.get()>
                            <button
                                type="button"
                                class="chip-x"
                                disabled=move || busy.get()
                                title=move || t!("tags-remove")
                                aria-label={
                                    let named = named.clone();
                                    move || t!("tags-remove-named", name = named.clone())
                                }
                                on:click={
                                    let gone = gone.clone();
                                    move |_| on_remove.run((relation, gone.clone()))
                                }
                            >
                                <Ico icon=Icon::Close size=18 />
                            </button>
                        </Show>
                    </span>
                }
            })
            .collect_view();
        view! { <div class="chips">{chips}</div> }.into_any()
    };
    let label = Signal::derive(move || match relation {
        Relation::Alias => t!("tags-new-alias"),
        Relation::Implies => t!("tags-implied-tag"),
    });
    view! {
        <section class="card">
            <h2 class="card-title">{title}</h2>
            <p class="muted hint">{hint}</p>
            {chips}
            <Show
                when=move || may.get()
                fallback=|| view! { <p class="muted fine">{move || t!("tags-admins-only")}</p> }
            >
                <InlineAdd
                    label=label
                    button=Signal::derive(move || t!("add"))
                    icon=Icon::Add
                    busy=busy
                    on_add=Callback::new(move |(value, done): (String, Callback<()>)| {
                        on_add.run((relation, value, done));
                    })
                />
            </Show>
            {children.map(|c| c())}
        </section>
    }
}
