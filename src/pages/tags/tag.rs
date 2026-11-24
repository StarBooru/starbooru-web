//! A tag's own page at `/tag?name=…`: a few of its posts, its category, its
//! place in the tree, its aliases and what it implies, with the ways to
//! change each for those who may.

use std::sync::Arc;

use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_query_map};
use send_wrapper::SendWrapper;
use starbooru_core::{Post, Tag, TagCategory, TagDetail, TagUpdate};

use super::tint_class;
use crate::api::{self, Failure};
use crate::components::back::BackButton;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::logic::gallery::tint;
use crate::logic::tags::{by_alias, may_edit_tag, may_relate_tags, merges};
use crate::logic::viewer::category_class as chip_class;
use crate::pages::{CATEGORIES, category_label, gallery};
use crate::route;
use crate::store;

/// Posts the top of the page shows, from one search.
const SHOTS: u32 = 4;
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
    // A few of its posts, to show what it is. Only a picture: a failed
    // request leaves the page as it is.
    let shots = RwSignal::new(None::<Vec<Arc<Post>>>);

    let load = move |wanted: String| {
        generation.update_value(|g| *g = g.wrapping_add(1));
        let at = generation.get_value();
        detail.set(None);
        failure.set(None);
        error.set(None);
        merging.set(None);
        merge_done.set_value(None);
        busy.set(false);
        shots.set(None);
        let searched = wanted.clone();
        spawn_local(async move {
            let found =
                api::call(move |c| async move { c.search_posts(&searched, None, 1, SHOTS).await })
                    .await;
            if generation.try_get_value() == Some(at) {
                let posts = found.map(|p| p.posts).unwrap_or_default();
                shots.set(Some(posts.into_iter().map(Arc::new).collect()));
            }
        });
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
        let category_tabs = move || {
            let now = tag.with(|t| t.as_ref().map(|t| t.category));
            CATEGORIES
                .map(|c| {
                    let on = now == Some(c);
                    view! {
                        <button
                            type="button"
                            role="radio"
                            class=format!("cat-tab {}", tint_class(c))
                            class:on=on
                            aria-checked=if on { "true" } else { "false" }
                            disabled=move || busy.get() || !editable.get()
                            on:click=move |_| {
                                if !on {
                                    act(Change::Category(c), None);
                                }
                            }
                        >
                            <span class="cat-dot" aria-hidden="true"></span>
                            {move || category_label(c)}
                        </button>
                    }
                })
                .collect_view()
        };
        let hero = move || {
            let t = tag.get()?;
            let browse = route::gallery(&t.name);
            let count = t.post_count;
            Some(view! {
                <header class=move || {
                    let now = tag.with(|t| t.as_ref().map_or(TagCategory::General, |t| t.category));
                    format!("tag-hero {}", tint_class(now))
                }>
                    <Shots name=t.name.clone() shots=shots />
                    <div class="hero-text">
                        <div class="titled">
                            <BackButton up=route::TAGS />
                            <h1 class="page-title tag-title break">{t.name.clone()}</h1>
                        </div>
                        {alias_note}
                        <div class="cat-tabs" role="radiogroup" aria-label=move || t!("tags-category")>
                            {category_tabs}
                        </div>
                        <a class="btn" href=browse>
                            <Ico icon=Icon::Gallery />
                            {move || t!("tags-browse", count = count)}
                        </a>
                    </div>
                </header>
            })
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
            {hero}
            {move || error.get().map(|message| view! { <ErrorBanner message=message /> })}
            <div class="tag-cards">
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
                    <h2 class="card-title with-icon">
                        <Ico icon=Icon::ArrowUp size=20 />
                        {move || t!("tags-implied-by")}
                    </h2>
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

/// A few of tag `name`'s posts, each leading to it in the tag's search, with
/// room kept for them while they come. None leaves a plain tile.
#[component]
fn Shots(name: String, shots: RwSignal<Option<Vec<Arc<Post>>>>) -> impl IntoView {
    let client = api::client();
    let tiles = move || {
        let posts = shots.get()?;
        let tiles = posts
            .into_iter()
            .map(|p| {
                let src = p
                    .thumb_url
                    .as_deref()
                    .and_then(|u| client.as_ref().map(|c| c.absolute(u)));
                let style = tint(&p.palette).map(|c| format!("--tint: {c}"));
                let loaded = RwSignal::new(false);
                view! {
                    <a
                        class="shot"
                        class:loaded=move || loaded.get()
                        href=route::post(p.id, &name, None)
                        style=style
                        tabindex="-1"
                        aria-hidden="true"
                    >
                        {src
                            .map(|src| {
                                view! {
                                    <img
                                        src=src
                                        alt=""
                                        decoding="async"
                                        on:load=move |_| loaded.set(true)
                                    />
                                }
                            })}
                    </a>
                }
            })
            .collect_view();
        Some(tiles)
    };
    view! {
        <div
            class="hero-shots"
            class:empty=move || shots.with(|s| s.as_ref().is_some_and(Vec::is_empty))
            data-count=move || shots.with(|s| s.as_ref().map_or(0, Vec::len))
        >
            {tiles}
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
            <h2 class="card-title with-icon">
                <Ico icon=Icon::Folder size=20 />
                {move || t!("tags-tree")}
            </h2>
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
    let icon = match relation {
        Relation::Alias => Icon::Link,
        Relation::Implies => Icon::Redo,
    };
    view! {
        <section class="card">
            <h2 class="card-title with-icon">
                <Ico icon=icon size=20 />
                {title}
            </h2>
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
