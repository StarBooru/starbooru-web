//! Explore at `/explore?q=…&days=…`: the posts most upvoted over a window,
//! the tags most used on posts added in it, posts added on this day in
//! earlier years, and random picks. `q` narrows every section.

use std::sync::Arc;

use chrono::Utc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_query_map};
use send_wrapper::SendWrapper;
use starbooru_core::{Explore as Found, Post, TagCorrection, TagTrend};

use super::post_menu::PostPress;
use super::{boorus, post};
use crate::api;
use crate::components::banner::{CorrectionBanner, ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::components::tag_menu::TagPress;
use crate::components::thumb::Thumb;
use crate::logic::explore::{
    RANDOM_COUNT, Span, WINDOWS, random_query, span, tag_search, tidy, window, year_added,
    years_ago,
};
use crate::logic::layout::Place;
use crate::logic::viewer::category_class;
use crate::route;

thread_local! {
    // Where Explore was last, and whether on its boorus side, for the
    // navigation to go back to. Reference counted, as they outlive any one
    // page.
    static LAST: ArcRwSignal<String> = ArcRwSignal::new(route::explore("", window(None)));
    static ON_BOORUS: ArcRwSignal<bool> = ArcRwSignal::new(false);
}

/// Where the navigation's Explore leads: to the side open last, and there
/// to where it was left. Read inside a view, it follows both.
pub fn last_route() -> String {
    if ON_BOORUS.with(|b| b.get()) {
        boorus::last_route()
    } else {
        LAST.with(|l| l.get())
    }
}

/// Explore's boorus side is open, or its collection side.
pub fn on_boorus(on: bool) {
    ON_BOORUS.with(|b| {
        if b.get_untracked() != on {
            b.set(on);
        }
    });
}

/// The two sides of Explore, where a page's title stands: what the
/// collection has to show, and the other boorus. Both pages have it, so the
/// navigation needs no item for the boorus. Each leads back to where it was
/// left.
#[component]
pub fn Sides(boorus: bool) -> impl IntoView {
    let collection = Memo::new(|_| LAST.with(|l| l.get()));
    let other = Memo::new(|_| boorus::last_route());
    let current = |on: bool| if on { "page" } else { "false" };
    view! {
        <nav class="sides" aria-label=move || t!("nav-explore")>
            <a class="side" class:on=!boorus aria-current=current(!boorus) href=move || collection.get()>
                {move || t!("explore-collection")}
            </a>
            <a class="side" class:on=boorus aria-current=current(boorus) href=move || other.get()>
                {move || t!("nav-boorus")}
            </a>
        </nav>
    }
}

/// How a window reads in a sentence: "today", "this week", …
fn phrase(days: u32) -> String {
    match span(days) {
        Span::Today => t!("explore-phrase-today"),
        Span::Week => t!("explore-phrase-week"),
        Span::Month => t!("explore-phrase-month"),
        Span::Year => t!("explore-phrase-year"),
        Span::Days(n) => t!("explore-phrase-days", count = n),
    }
}

/// A window's name on its button.
fn window_label(days: u32) -> String {
    match span(days) {
        Span::Today => t!("explore-today"),
        Span::Week => t!("explore-week"),
        Span::Month => t!("explore-month"),
        Span::Year => t!("explore-year"),
        Span::Days(n) => t!("explore-phrase-days", count = n),
    }
}

/// What the server found, held as the tiles take it.
#[derive(Clone)]
struct Sections {
    trending: Vec<Arc<Post>>,
    tags: Vec<TagTrend>,
    on_this_day: Vec<Arc<Post>>,
    random: Vec<Arc<Post>>,
    corrected: Vec<TagCorrection>,
}

impl From<Found> for Sections {
    fn from(f: Found) -> Self {
        let arcs = |posts: Vec<Post>| posts.into_iter().map(Arc::new).collect::<Vec<_>>();
        Self {
            trending: arcs(f.trending),
            tags: f.tags,
            on_this_day: arcs(f.on_this_day),
            random: arcs(f.random),
            corrected: f.corrected,
        }
    }
}

/// Posts as the gallery's tiles, in a grid. Each opens its post with no
/// search to step through, and Back comes here.
fn tiles(posts: Vec<Arc<Post>>, open: Callback<Arc<Post>>) -> impl IntoView {
    posts
        .into_iter()
        .enumerate()
        .map(|(index, p)| {
            let href = route::post(p.id, "", None);
            let place: Option<Place> = None;
            view! { <Thumb post=p index=index href=href place=place still=false on_open=open /> }
        })
        .collect_view()
}

#[component]
pub fn Explore() -> impl IntoView {
    let query = use_query_map();
    let q = Memo::new(move |_| query.with(|m| tidy(&m.get("q").unwrap_or_default())));
    let days = Memo::new(move |_| query.with(|m| window(m.get("days").as_deref())));
    let filter = RwSignal::new(q.get_untracked());
    let found = RwSignal::new(None::<Result<Sections, String>>);
    // Replaces the random picks after a shuffle, until the next load.
    let shuffled = RwSignal::new(None::<Vec<Arc<Post>>>);
    let shuffling = RwSignal::new(false);
    let shuffle_error = RwSignal::new(None::<String>);
    let generation = StoredValue::new(0_u64);

    let load = move || {
        generation.update_value(|g| *g = g.wrapping_add(1));
        let at = generation.get_value();
        let (search, window) = (q.get_untracked(), days.get_untracked());
        found.set(None);
        shuffled.set(None);
        shuffle_error.set(None);
        spawn_local(async move {
            let result = api::call(move |c| async move { c.explore(window, &search).await }).await;
            if generation.try_get_value() == Some(at) {
                found.try_set(Some(result.map(Sections::from)));
            }
        });
    };
    Effect::watch(
        move || (q.get(), days.get()),
        move |(search, window), _, _| {
            filter.set(search.clone());
            LAST.with(|l| l.set(route::explore(search, *window)));
            load();
        },
        true,
    );

    let shuffle = move |_| {
        if shuffling.get_untracked() {
            return;
        }
        let at = generation.get_value();
        let search = random_query(&q.get_untracked());
        shuffling.set(true);
        shuffle_error.set(None);
        spawn_local(async move {
            let result =
                api::call(
                    move |c| async move { c.search_posts(&search, None, 1, RANDOM_COUNT).await },
                )
                .await;
            shuffling.try_set(false);
            if generation.try_get_value() != Some(at) {
                return;
            }
            // A failure keeps the picks there were, and says so in their section.
            match result {
                Ok(page) => {
                    shuffled.try_set(Some(page.posts.into_iter().map(Arc::new).collect()));
                }
                Err(message) => {
                    shuffle_error.try_set(Some(message));
                }
            }
        });
    };

    let navigate = SendWrapper::new(use_navigate());
    let go = move |to: String, replace: bool| {
        let options = NavigateOptions {
            replace,
            scroll: !replace,
            ..NavigateOptions::default()
        };
        (*navigate)(&to, options);
    };
    let go = SendWrapper::new(go);
    let go = Callback::new(move |(to, replace): (String, bool)| (*go)(to, replace));

    // The post a right-clicked tile stands for, in any section.
    let find = Callback::new(move |id: i64| {
        let shuffled =
            shuffled.with_untracked(|s| s.iter().flatten().find(|p| p.id == id).cloned());
        shuffled.or_else(|| {
            found.with_untracked(|f| {
                let s = f.as_ref()?.as_ref().ok()?;
                s.trending
                    .iter()
                    .chain(&s.on_this_day)
                    .chain(&s.random)
                    .find(|p| p.id == id)
                    .cloned()
            })
        })
    });
    // The viewer shows the tile's post at once, and its Back comes here.
    let open = Callback::new(move |p: Arc<Post>| post::opening(p, String::new()));

    let window_buttons = move || {
        WINDOWS
            .into_iter()
            .map(|value| {
                let on = move || days.get() == value;
                view! {
                    <button
                        type="button"
                        role="radio"
                        class="seg"
                        class:on=on
                        aria-checked=move || on().to_string()
                        on:click=move |_| go.run((route::explore(&q.get_untracked(), value), true))
                    >
                        {move || window_label(value)}
                    </button>
                }
            })
            .collect_view()
    };

    let body = move || {
        let result = found.get()?;
        let sections = match result {
            Ok(s) => s,
            Err(message) => {
                let retry = Callback::new(move |()| load());
                return Some(view! { <ErrorBanner message=message on_retry=retry /> }.into_any());
            }
        };
        let search = q.get_untracked();
        let when = phrase(days.get_untracked());
        let corrected = (!sections.corrected.is_empty()).then(|| {
            let exact = route::explore(&starbooru_core::with_exact(&search), days.get_untracked());
            view! { <CorrectionBanner corrected=sections.corrected.clone() exact=exact /> }
        });
        // Random picks come from the whole collection, narrowed: with none,
        // there's nothing to explore.
        if sections.random.is_empty() {
            let empty = if search.is_empty() {
                view! {
                    <h2>{move || t!("explore-nothing")}</h2>
                    <p class="muted">{move || t!("explore-nothing-body")}</p>
                    <a class="btn" href=route::UPLOAD>
                        <Ico icon=Icon::Upload />
                        {move || t!("nav-upload")}
                    </a>
                }
                .into_any()
            } else {
                let typed = search.clone();
                view! {
                    <h2>{move || t!("no-matches")}</h2>
                    <p class="muted break">{move || t!("nothing-matches", query = typed.clone())}</p>
                    <a class="btn tonal" href=route::explore("", days.get_untracked())>
                        {move || t!("explore-everything")}
                    </a>
                }
                .into_any()
            };
            return Some(
                view! {
                    {corrected}
                    <div class="card empty">
                        <Ico icon=Icon::Explore size=48 />
                        {empty}
                    </div>
                }
                .into_any(),
            );
        }

        let trending = if sections.trending.is_empty() {
            let when = when.clone();
            view! { <p class="muted">{move || t!("explore-no-upvotes", when = when.clone())}</p> }
                .into_any()
        } else {
            view! { <div class="grid" data-press="post">{tiles(sections.trending, open)}</div> }
                .into_any()
        };

        let tags = if sections.tags.is_empty() {
            let when = when.clone();
            view! { <p class="muted">{move || t!("explore-no-tags", when = when.clone())}</p> }
                .into_any()
        } else {
            let added = t!("explore-added", when = when.clone());
            let chips = sections
                .tags
                .into_iter()
                .map(|trend| {
                    let name = trend.tag.name;
                    let href = route::gallery(&tag_search(&search, &name));
                    let class = format!("{} trend", category_class(trend.tag.category));
                    let key = name.clone();
                    view! {
                        <a class=class href=href data-tag=key>
                            <span class="tag-name">{name}</span>
                            <span class="trend-count" title=added.clone()>
                                {format!("+{}", trend.recent)}
                            </span>
                        </a>
                    }
                })
                .collect_view();
            view! { <div class="chips" data-press="tag">{chips}</div> }.into_any()
        };
        let tags_note = t!("explore-tags-note", when = when.clone());

        let now = Utc::now();
        let memories = if sections.on_this_day.is_empty() {
            view! { <p class="muted">{move || t!("explore-no-memories")}</p> }.into_any()
        } else {
            let cards = sections
                .on_this_day
                .into_iter()
                .enumerate()
                .map(|(index, p)| {
                    let year = year_added(&p.created_at);
                    let ago = t!("explore-years-ago", count = years_ago(&p.created_at, &now));
                    let href = route::post(p.id, "", None);
                    let place: Option<Place> = None;
                    view! {
                        <div class="memory" title=ago>
                            <span class="memory-year">{year}</span>
                            <Thumb post=p index=index href=href place=place still=false on_open=open />
                        </div>
                    }
                })
                .collect_view();
            view! { <div class="grid" data-press="post">{cards}</div> }.into_any()
        };

        let random = move || {
            let picks = shuffled.get().unwrap_or_else(|| sections.random.clone());
            view! { <div class="grid" data-press="post">{tiles(picks, open)}</div> }
        };

        Some(
            view! {
                {corrected}
                <section class="explore-section" aria-labelledby="trending">
                    <div class="section-head">
                        <Ico icon=Icon::TrendingUp />
                        <h2 id="trending">{move || t!("explore-trending")}</h2>
                    </div>
                    {trending}
                </section>
                <section class="explore-section" aria-labelledby="popular-tags">
                    <div class="section-head">
                        <Ico icon=Icon::Label />
                        <h2 id="popular-tags">{move || t!("explore-popular-tags")}</h2>
                        <span class="muted">{tags_note}</span>
                    </div>
                    {tags}
                </section>
                <section class="explore-section" aria-labelledby="on-this-day">
                    <div class="section-head">
                        <Ico icon=Icon::History />
                        <h2 id="on-this-day">{move || t!("explore-on-this-day")}</h2>
                    </div>
                    {memories}
                </section>
                <section class="explore-section" aria-labelledby="random-picks">
                    <div class="section-head">
                        <Ico icon=Icon::Shuffle />
                        <h2 id="random-picks">{move || t!("explore-random")}</h2>
                        <button
                            type="button"
                            class="btn text small"
                            disabled=move || shuffling.get()
                            on:click=shuffle
                        >
                            {move || t!("explore-shuffle")}
                        </button>
                    </div>
                    {move || shuffle_error.get().map(|message| view! { <ErrorBanner message=message /> })}
                    {random}
                </section>
            }
            .into_any(),
        )
    };

    on_boorus(false);
    view! {
        <div class="page explore">
            <div class="page-head">
                <h1 class="page-title visually-hidden">{move || t!("nav-explore")}</h1>
                <Sides boorus=false />
                <div class="segmented" role="radiogroup" aria-label=move || t!("explore-window")>
                    {window_buttons}
                </div>
                <form
                    class="explore-filter"
                    role="search"
                    on:submit=move |e| {
                        e.prevent_default();
                        go.run((route::explore(&filter.get_untracked(), days.get_untracked()), false));
                    }
                >
                    <Ico icon=Icon::Search size=20 />
                    <input
                        type="search"
                        enterkeyhint="search"
                        autocomplete="off"
                        autocapitalize="none"
                        spellcheck="false"
                        placeholder=move || t!("explore-filter-placeholder")
                        aria-label=move || t!("explore-filter-label")
                        prop:value=move || filter.get()
                        on:input=move |e| filter.set(event_target_value(&e))
                    />
                </form>
            </div>
            {move || {
                if found.with(Option::is_none) {
                    Some(
                        view! {
                            <div class="sentinel">
                                <Spinner />
                            </div>
                        },
                    )
                } else {
                    None
                }
            }}
            {body}
            <PostPress find=find on_open=open />
            <TagPress q=q />
        </div>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn each_window_has_a_name_and_a_phrase() {
        let en = include_str!("../../locales/en.ftl");
        for id in [
            "explore-today",
            "explore-week",
            "explore-month",
            "explore-year",
            "explore-phrase-today",
            "explore-phrase-week",
            "explore-phrase-month",
            "explore-phrase-year",
            "explore-phrase-days",
        ] {
            assert!(en.contains(&format!("\n{id} = ")), "{id}");
        }
    }

    #[test]
    fn every_section_says_when_it_is_empty() {
        let source = include_str!("explore.rs");
        for id in [
            "explore-no-upvotes",
            "explore-no-tags",
            "explore-no-memories",
            "explore-nothing",
            "nothing-matches",
        ] {
            assert!(source.contains(&format!("t!(\"{id}\"")), "{id}");
        }
        // The tiles and chips have their menus.
        assert!(source.contains(r#"data-press="post""#));
        assert!(source.contains(r#"data-press="tag""#));
    }
}
