//! Explore at `/explore?q=…`: the posts opened lately, posts added on this
//! day in earlier years and random picks. The last two are narrowed by `q`,
//! which the top bar's search box sets while Explore is open (`shell.rs`);
//! the server can't narrow a list of posts by id, so the history steps aside
//! while a search is on. A public server also has Trending, the posts most
//! liked over the window in `days`, and the tags most used on posts added in
//! it; a private one has neither, and its address's `days` is ignored.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use futures_util::future::join_all;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_query_map;
use starbooru_core::{Explore as Found, Post, TagCorrection, TagTrend};

use super::post_menu::PostPress;
use super::{boorus, post};
use crate::components::banner::{CorrectionBanner, ErrorBanner, Spinner};
use crate::components::icons::{Ico, Icon};
use crate::components::snackbar::notify_undo;
use crate::components::thumb::Thumb;
use crate::logic::explore::{
    RANDOM_COUNT, Span, WINDOWS, days_asked, random_query, span, tag_search, tidy, year_added,
    years_ago,
};
use crate::logic::history::{SHOWN, to_fetch};
use crate::logic::layout::Place;
use crate::logic::search::category_class;
use crate::{api, dom, instance, power, route, store};

/// How long the history takes to fold away once cleared: `--med`.
const LEAVE_MS: u64 = 250;

thread_local! {
    // Where Explore was last, and whether on its boorus side, for the
    // navigation to go back to. Reference counted, as they outlive any one
    // page.
    static LAST: ArcRwSignal<String> = ArcRwSignal::new(route::explore(""));
    static ON_BOORUS: ArcRwSignal<bool> = ArcRwSignal::new(false);
}

/// Where the navigation's Explore leads: to the side open last, and there
/// to where it was left. Read inside a view, it follows both. A public
/// server has no boorus side.
pub fn last_route() -> String {
    if ON_BOORUS.with(|b| b.get()) && !instance::public() {
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

/// The posts opened lately, as far as they have come.
#[derive(Clone, Default)]
struct Recent {
    /// The history as it was when the page opened, less the posts found
    /// gone or in the recycle bin.
    ids: Vec<i64>,
    /// How many of them the section shows.
    wanted: usize,
    posts: HashMap<i64, Arc<Post>>,
}

/// A place in the history's grid: a post, or one still on its way.
#[derive(Clone)]
enum Spot {
    Post(Arc<Post>),
    Waiting(i64),
}

impl Spot {
    fn key(&self) -> (i64, bool) {
        match self {
            Spot::Post(p) => (p.id, true),
            Spot::Waiting(id) => (*id, false),
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
    // Trending and Popular tags, on a public server, over the address's window.
    let public = Memo::new(|_| instance::public());
    let days = Memo::new(move |_| {
        let asked = query.with(|m| m.get("days"));
        days_asked(public.get(), asked.as_deref())
    });
    // Where Explore is, as its address keeps it.
    let here = move |search: &str, window: u32| {
        if public.get_untracked() {
            route::explore_over(search, window)
        } else {
            route::explore(search)
        }
    };
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
    // Once the server has said whether it's public, so a public one isn't
    // asked twice.
    let known = Memo::new(|_| instance::known());
    Effect::watch(
        move || (known.get(), q.get(), days.get()),
        move |(known, search, window), _, _| {
            if *known {
                LAST.with(|l| l.set(here(search, *window)));
                load();
            }
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

    // The posts opened lately. They stay while the page does, so a search
    // that hides them and is cleared again asks for none of them twice.
    let recent = RwSignal::new(Recent {
        ids: store::opened_posts(),
        wanted: SHOWN,
        posts: HashMap::new(),
    });
    let recent_busy = RwSignal::new(false);
    let recent_error = RwSignal::new(None::<String>);
    // Cleared, and folding away.
    let leaving = RwSignal::new(false);

    // Asks for the shown posts that haven't come, one request each, all at
    // once. Posts gone or in the bin leave the history quietly, and the next
    // ones take their places.
    let fetch_recent = move || {
        if recent_busy.try_get_untracked() != Some(false) {
            return;
        }
        recent_busy.try_set(true);
        recent_error.try_set(None);
        spawn_local(async move {
            let mut failed = HashSet::new();
            let mut problem = None;
            loop {
                let Some(asking) = recent.try_with_untracked(|r| {
                    to_fetch(&r.ids, r.wanted, |id| {
                        r.posts.contains_key(&id) || failed.contains(&id)
                    })
                }) else {
                    return;
                };
                if asking.is_empty() {
                    break;
                }
                let answers = join_all(
                    asking
                        .iter()
                        .map(|&id| api::fetch(move |c| async move { c.post(id).await })),
                )
                .await;
                let mut gone = Vec::new();
                let mut came = Vec::new();
                for (id, answer) in asking.into_iter().zip(answers) {
                    match answer {
                        Ok(p) if !p.deleted => came.push(Arc::new(p)),
                        Ok(_) => gone.push(id),
                        Err(f) if f.missing => gone.push(id),
                        Err(f) => {
                            failed.insert(id);
                            problem.get_or_insert(f.message);
                        }
                    }
                }
                store::forget_opened(&gone);
                let here = recent.try_update(|r| {
                    r.ids.retain(|id| !gone.contains(id));
                    r.posts.extend(came.into_iter().map(|p| (p.id, p)));
                });
                if here.is_none() {
                    return;
                }
            }
            recent_busy.try_set(false);
            recent_error.try_set(problem);
        });
    };
    // Only while no search hides them.
    Effect::watch(
        move || q.with(String::is_empty),
        move |plain, _, _| {
            if *plain {
                fetch_recent();
            }
        },
        true,
    );
    // The section folds away and the notice can bring it back. Without the
    // page, Undo still puts the history back for next time.
    let clear = move |_| {
        if leaving.get_untracked() {
            return;
        }
        let cleared = store::clear_opened();
        leaving.set(true);
        let ms = if dom::reduced_motion() || power::active_untracked() {
            0
        } else {
            LEAVE_MS
        };
        set_timeout(
            move || {
                if leaving.try_get_untracked() == Some(true) {
                    recent.try_update(|r| r.ids.clear());
                    leaving.try_set(false);
                }
            },
            Duration::from_millis(ms),
        );
        notify_undo(t!("explore-recent-cleared"), t!("press-undo"), move || {
            let Some(ids) = store::put_back_opened(&cleared) else {
                return;
            };
            leaving.try_set(false);
            if recent.try_update(|r| r.ids = ids).is_some() {
                fetch_recent();
            }
        });
    };
    let more = move |_| {
        recent.update(|r| r.wanted += SHOWN);
        fetch_recent();
    };

    // The post a right-clicked tile stands for, in any section.
    let find = Callback::new(move |id: i64| {
        let opened = recent.with_untracked(|r| r.posts.get(&id).cloned());
        let shuffled =
            || shuffled.with_untracked(|s| s.iter().flatten().find(|p| p.id == id).cloned());
        opened.or_else(shuffled).or_else(|| {
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

    // The shown posts in the history's order, with a place held for each
    // one on its way.
    let spots = move || {
        let busy = recent_busy.get();
        recent.with(|r| {
            r.ids
                .iter()
                .take(r.wanted)
                .filter_map(|id| match r.posts.get(id) {
                    Some(p) => Some(Spot::Post(Arc::clone(p))),
                    None => busy.then_some(Spot::Waiting(*id)),
                })
                .enumerate()
                .collect::<Vec<_>>()
        })
    };
    let history = move || {
        view! {
            <section
                class="explore-section recent"
                class:leaving=move || leaving.get()
                aria-labelledby="recently-viewed"
            >
                <div class="recent-body">
                    <div class="section-head">
                        <Ico icon=Icon::History />
                        <h2 id="recently-viewed">{move || t!("explore-recent")}</h2>
                        <button
                            type="button"
                            class="btn text small"
                            aria-label=move || t!("explore-recent-clear-label")
                            disabled=move || leaving.get()
                            on:click=clear
                        >
                            {move || t!("explore-recent-clear")}
                        </button>
                    </div>
                    {move || {
                        recent_error
                            .get()
                            .map(|message| {
                                let retry = Callback::new(move |()| fetch_recent());
                                view! { <ErrorBanner message=message on_retry=retry /> }
                            })
                    }}
                    <div class="grid" data-press="post">
                        <For
                            each=spots
                            key=|(_, spot)| spot.key()
                            children=move |(index, spot)| match spot {
                                Spot::Post(p) => {
                                    let href = route::post(p.id, "", None);
                                    let place: Option<Place> = None;
                                    view! {
                                        <Thumb
                                            post=p
                                            index=index
                                            href=href
                                            place=place
                                            still=false
                                            on_open=open
                                        />
                                    }
                                        .into_any()
                                }
                                Spot::Waiting(_) => {
                                    view! { <div class="thumb waiting" aria-hidden="true"></div> }
                                        .into_any()
                                }
                            }
                        />
                    </div>
                    <Show when=move || {
                        !recent_busy.get() && recent.with(|r| r.ids.len() > r.wanted)
                    }>
                        <div class="recent-more">
                            <button type="button" class="btn tonal" on:click=more>
                                {move || t!("load-more")}
                            </button>
                        </div>
                    </Show>
                </div>
            </section>
        }
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
        let window = days.get_untracked();
        let corrected = (!sections.corrected.is_empty()).then(|| {
            let exact = here(&starbooru_core::with_exact(&search), window);
            view! { <CorrectionBanner corrected=sections.corrected.clone() exact=exact /> }
        });
        // Random picks come from the whole collection, narrowed: with none,
        // there's nothing to explore.
        if sections.random.is_empty() {
            let empty = if search.is_empty() && instance::guest() {
                view! {
                    <h2>{move || t!("explore-nothing")}</h2>
                    <p class="muted">{move || t!("guest-gallery-empty")}</p>
                }
                .into_any()
            } else if search.is_empty() {
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
                    <a class="btn tonal" href=here("", window)>
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

        // Posts from this day lead when there are any, since they're gone
        // tomorrow. Otherwise the random picks lead and the note saying
        // there are none goes last.
        let memories_first = !sections.on_this_day.is_empty();
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
        let memories = view! {
            <section class="explore-section" aria-labelledby="on-this-day">
                <div class="section-head">
                    <Ico icon=Icon::Calendar />
                    <h2 id="on-this-day">{move || t!("explore-on-this-day")}</h2>
                </div>
                {memories}
            </section>
        };

        let picks = move || {
            let picks = shuffled.get().unwrap_or_else(|| sections.random.clone());
            view! { <div class="grid" data-press="post">{tiles(picks, open)}</div> }
        };
        let random = view! {
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
                {picks}
            </section>
        };

        let content = if memories_first {
            view! {
                {memories}
                {random}
            }
            .into_any()
        } else {
            view! {
                {random}
                {memories}
            }
            .into_any()
        };
        // What's popular lately leads on a public server.
        let popular = public
            .get_untracked()
            .then(|| popular(sections.trending, sections.tags, &search, window, open));
        Some(
            view! {
                {corrected}
                {popular}
                {content}
            }
            .into_any(),
        )
    };

    // The window Trending is over: a public server's, where the sides would
    // be, as it has no boorus.
    let windows = move || {
        WINDOWS
            .into_iter()
            .map(|value| {
                let on = move || days.get() == value;
                view! {
                    <a
                        class="seg"
                        class:on=on
                        role="radio"
                        aria-checked=move || on().to_string()
                        href=move || route::explore_over(&q.get(), value)
                    >
                        {move || window_label(value)}
                    </a>
                }
            })
            .collect_view()
    };

    on_boorus(false);
    view! {
        <div class="page explore">
            <div class="page-head">
                // The search box above narrows Explore; the two sides stand
                // where the gallery has its title.
                <h1 class="page-title visually-hidden">
                    {move || {
                        let q = q.get();
                        if q.is_empty() { t!("nav-explore") } else { q }
                    }}
                </h1>
                <Show when=move || known.get()>
                    <Show
                        when=move || public.get()
                        fallback=|| view! { <Sides boorus=false /> }
                    >
                        <div class="segmented explore-window" role="radiogroup" aria-label=move || t!("explore-window")>
                            {windows}
                        </div>
                    </Show>
                </Show>
            </div>
            // First, as a post looked for again is most often one just seen.
            <Show when=move || q.with(String::is_empty) && recent.with(|r| !r.ids.is_empty())>
                {history}
            </Show>
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
        </div>
    }
}

/// Trending and Popular tags, over `window`: the posts most liked, then the
/// tags most used on the posts added, each leading to the gallery narrowed
/// to it.
fn popular(
    trending: Vec<Arc<Post>>,
    tags: Vec<TagTrend>,
    search: &str,
    window: u32,
    open: Callback<Arc<Post>>,
) -> impl IntoView {
    let when = phrase(window);
    let trending = if trending.is_empty() {
        let when = when.clone();
        view! { <p class="muted">{move || t!("explore-no-likes", when = when.clone())}</p> }
            .into_any()
    } else {
        view! { <div class="grid" data-press="post">{tiles(trending, open)}</div> }.into_any()
    };
    let tags = if tags.is_empty() {
        let when = when.clone();
        view! { <p class="muted">{move || t!("explore-no-tags", when = when.clone())}</p> }
            .into_any()
    } else {
        let added = t!("explore-added", when = when.clone());
        let chips = tags
            .into_iter()
            .map(|trend| {
                let name = trend.tag.name;
                let href = route::gallery(&tag_search(search, &name));
                let class = format!("chip {} trend", category_class(trend.tag.category));
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
    let note = t!("explore-tags-note", when = when);
    view! {
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
                <span class="muted">{note}</span>
            </div>
            {tags}
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_window_has_a_name_and_a_phrase() {
        for days in WINDOWS {
            assert!(!window_label(days).is_empty());
            assert!(!phrase(days).is_empty());
        }
        assert_eq!(window_label(7), "Week");
        assert_eq!(phrase(30), "this month");
        assert_eq!(phrase(14), "in the last 14 days");
    }

    #[test]
    fn a_public_server_shows_whats_popular_first() {
        let source = include_str!("explore.rs");
        let (page, _) = source.split_once("#[cfg(test)]").unwrap_or_default();
        assert!(page.contains("c.explore(window, &search)"));
        assert!(page.contains(".then(|| popular("));
        for id in ["explore-no-likes", "explore-no-tags"] {
            assert!(page.contains(&format!("t!(\"{id}\"")), "{id}");
        }
    }

    #[test]
    fn every_section_says_when_it_is_empty() {
        let source = include_str!("explore.rs");
        for id in ["explore-no-memories", "explore-nothing", "nothing-matches"] {
            assert!(source.contains(&format!("t!(\"{id}\"")), "{id}");
        }
        // The tiles have their menu.
        assert!(source.contains(r#"data-press="post""#));
    }

    #[test]
    fn the_top_bar_searches_explore_and_a_search_finding_nothing_leads_back() {
        let source = include_str!("explore.rs");
        let (page, _) = source.split_once("#[cfg(test)]").unwrap_or_default();
        // No box of its own: the shell's narrows Explore.
        for own in ["<input", "<form", "role=\"search\""] {
            assert!(!page.contains(own), "{own}");
        }
        assert!(page.contains("t!(\"explore-everything\")"));
        assert!(page.contains("href=here(\"\", window)"));
        assert!(page.contains("<CorrectionBanner"));
        let css = include_str!("../../style/pages/explore.css");
        assert!(!css.contains("explore-filter"));
    }
}
