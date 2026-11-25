//! The home page at `/home`, when the server has one on: the StarBooru mark,
//! the title and the line under it from `GET /home`, and the top bar's search
//! box, over the background's thumbnails drifting by in columns. The shell
//! opens here when a visit starts at the bare address (`logic::home::start`)
//! and the brand leads here; with the page off, the address shows the
//! gallery. A guest's reads carry their blacklist, as every read does.

use std::cell::RefCell;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use send_wrapper::SendWrapper;
use starbooru_core::{Home, HomePost};

use super::shell::SearchBox;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::brand::Mark;
use crate::logic::gallery::tint;
use crate::logic::home::{Column, background, columns_for};
use crate::{api, dom, instance, route, store};

/// The space under each tile, and between columns, in pixels.
const GAP: f64 = 12.0;

thread_local! {
    /// What `GET /home` said, and for whom.
    static KEPT: RefCell<Option<(String, Home)>> = const { RefCell::new(None) };
}

/// Who the page is for: an account, or the guests of a server.
fn reader() -> String {
    format!(
        "{}|{}",
        store::owner(),
        instance::guest_server().unwrap_or_default()
    )
}

fn kept() -> Option<Home> {
    let reader = reader();
    KEPT.with_borrow(|k| {
        k.as_ref()
            .filter(|(of, _)| *of == reader)
            .map(|(_, home)| home.clone())
    })
}

/// Drops the page kept, after a change to what it shows: its settings, or
/// the blacklist.
pub fn forget() {
    KEPT.with_borrow_mut(|k| *k = None);
}

#[component]
pub fn HomePage() -> impl IntoView {
    let on = Memo::new(|_| instance::home());
    // Off: the gallery is the way in.
    let navigate = SendWrapper::new(use_navigate());
    Effect::new(move |_| {
        if on.get() == Some(false) {
            let options = NavigateOptions {
                replace: true,
                ..NavigateOptions::default()
            };
            (*navigate)(&route::gallery(""), options);
        }
    });
    let found = RwSignal::new(kept().map(Ok::<Home, String>));
    let load = move || {
        let reader = reader();
        spawn_local(async move {
            let answer = api::call(|c| async move { c.home().await }).await;
            if let Ok(home) = &answer {
                KEPT.with_borrow_mut(|k| *k = Some((reader, home.clone())));
            }
            found.try_set(Some(answer));
        });
    };
    if found.with_untracked(Option::is_none) {
        load();
    }
    let retry = Callback::new(move |()| {
        found.set(None);
        load();
    });
    let home = Memo::new(move |_| found.get().and_then(Result::ok));
    // The server's name stands in until the page's own title comes.
    let title = move || {
        home.with(|h| h.as_ref().map(|h| h.title.clone()))
            .or_else(|| instance::info().map(|i| i.name))
            .unwrap_or_default()
    };
    let subtitle = move || {
        home.with(|h| h.as_ref().and_then(|h| h.subtitle.clone()))
            .map(|line| view! { <p class="home-subtitle">{line}</p> })
    };
    let posts = Memo::new(move |_| home.with(|h| h.as_ref().map(|h| h.posts.clone())));

    view! {
        <Show
            when=move || on.get() == Some(true)
            fallback=|| {
                view! {
                    <div class="center-page">
                        <Spinner />
                    </div>
                }
            }
        >
            <div class="home-page">
                {move || posts.get().map(|posts| view! { <Drift posts=posts /> })}
                <div class="home-scrim" aria-hidden="true"></div>
                <div class="home-hero">
                    <span class="home-mark">
                        <Mark height=84 />
                    </span>
                    <h1 class="home-title break">{title}</h1>
                    {subtitle}
                    <SearchBox hero=true />
                    {move || {
                        found
                            .get()
                            .and_then(Result::err)
                            .map(|message| view! { <ErrorBanner message=message on_retry=retry /> })
                    }}
                </div>
            </div>
        </Show>
    }
}

/// The background: the posts in columns that drift up and down, each drawn
/// twice over so its loop has no seam. Decoration only, out of reach of the
/// pointer and the keyboard. Laid out again when the window's width changes.
#[component]
fn Drift(posts: Vec<HomePost>) -> impl IntoView {
    let size = RwSignal::new(dom::window_size());
    let watch = dom::on_window("resize", move || {
        let now = dom::window_size();
        let wider = size
            .try_with_untracked(|(w, h)| (now.0 - w).abs() > 1.0 || now.1 > *h + 1.0)
            .unwrap_or(false);
        if wider {
            size.try_set(now);
        }
    });
    let watch = SendWrapper::new(watch);
    on_cleanup(move || watch.take().stop());
    let sizes: Vec<(u32, u32)> = posts.iter().map(|p| (p.width, p.height)).collect();
    let columns = Memo::new(move |_| {
        let (width, height) = size.get();
        let n = columns_for(width);
        let column = (width / n as f64).max(1.0);
        background(&sizes, n, height / column + 0.5, GAP / column)
    });
    let posts = StoredValue::new(posts);
    let column = move |(i, c): (usize, Column)| {
        let tiles = posts.with_value(|posts| {
            c.tiles
                .iter()
                .filter_map(|t| posts.get(t.index).map(|p| (t.ratio, p.clone())))
                .collect::<Vec<_>>()
        });
        let once = move || {
            tiles
                .iter()
                .map(|(ratio, p)| view! { <Tile post=p.clone() ratio=*ratio /> })
                .collect_view()
        };
        // Each column starts at its own point of its loop.
        let style = format!(
            "--secs: {:.1}s; --offset: {:.3}",
            c.seconds,
            (i as f64 * 0.37).fract()
        );
        view! {
            <div class="drift-col" class:up=c.up style=style>
                <div class="drift-track">{once()} {once()}</div>
            </div>
        }
    };
    view! {
        <div class="home-drift" aria-hidden="true">
            {move || columns.get().into_iter().enumerate().map(column).collect_view()}
        </div>
    }
}

/// One picture of the background, in the post's colour until it loads.
#[component]
fn Tile(post: HomePost, ratio: f64) -> impl IntoView {
    let loaded = RwSignal::new(false);
    let mut style = format!("aspect-ratio: 1000 / {:.0}", ratio * 1000.0);
    if let Some(colour) = tint(&post.palette) {
        style.push_str(&format!("; --tint: {colour}"));
    }
    let src = api::media(&post.thumb_url);
    view! {
        <div class="drift-tile" class:loaded=move || loaded.get() style=style>
            <img
                src=src
                alt=""
                decoding="async"
                draggable="false"
                on:load=move |_| loaded.set(true)
            />
        </div>
    }
}
