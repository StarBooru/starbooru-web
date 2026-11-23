//! `/s/:token`: what someone shared, for anyone with the link. No account is
//! needed and none is used: a client with no session asks the server that
//! serves this page for the link, and the media comes through the signed
//! addresses in its answer. One post stands on the viewer's stage, with zoom;
//! several are tiles that open a lightbox.

use std::time::Duration;

use chrono::{DateTime, Local, Utc};
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use send_wrapper::SendWrapper;
use starbooru_client::Client;
use starbooru_core::MediaKind;
use starbooru_core::share::is_share_token;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

use crate::api;
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::brand::{Mark, Wordmark};
use crate::components::focus::hold_focus;
use crate::components::icons::{Ico, Icon};
use crate::components::player::Player;
use crate::logic::share::{Item, Layout, heading, items, layout, step};
use crate::logic::viewer::{fit_style, format_duration, when};
use crate::logic::zoom::Zoom;
use crate::pages::post::glow::PostGlow;
use crate::pages::post::stage::{Shown, Stage};
use crate::{dom, power};

/// What the link holds, as the page shows it.
#[derive(Clone, Debug, PartialEq)]
struct Found {
    title: Option<String>,
    /// The server's name.
    server: String,
    expires_at: Option<DateTime<Utc>>,
    items: Vec<Item>,
}

#[derive(Clone, Debug, PartialEq)]
enum State {
    Waiting,
    /// Expired, removed, never there, or its owner's account is off: the
    /// server doesn't say which, and neither does the page.
    Missing,
    Failed(String),
    Ready(Found),
}

/// Asks the server this page came from what the link holds.
async fn fetch(token: String) -> State {
    if !is_share_token(&token) {
        return State::Missing;
    }
    let Some(origin) = dom::origin() else {
        return State::Failed(t!("error-bad-address"));
    };
    let client = match Client::new(&origin) {
        Ok(client) => client,
        Err(e) => return State::Failed(api::error_text(&e, &origin)),
    };
    match client.shared(&token).await {
        Ok(view) => State::Ready(Found {
            items: items(&view, |u| client.absolute(u)),
            title: view.title,
            server: view.server,
            expires_at: view.expires_at,
        }),
        Err(e) if api::is_missing(&e) => State::Missing,
        Err(e) => State::Failed(api::error_text(&e, &origin)),
    }
}

/// Keeps search engines off the page while it's open: the address is the
/// key to what it shows.
fn hide_from_robots() -> Option<web_sys::Element> {
    let doc = web_sys::window()?.document()?;
    let meta = doc.create_element("meta").ok()?;
    meta.set_attribute("name", "robots").ok()?;
    meta.set_attribute("content", "noindex, nofollow").ok()?;
    doc.query_selector("head").ok()??.append_child(&meta).ok()?;
    Some(meta)
}

#[component]
pub fn SharedPage() -> impl IntoView {
    let params = use_params_map();
    let token = Memo::new(move |_| params.with(|p| p.get("token").unwrap_or_default()));
    let state = RwSignal::new(State::Waiting);
    let generation = StoredValue::new(0_u64);

    let load = move |token: String| {
        generation.update_value(|g| *g = g.wrapping_add(1));
        let at = generation.get_value();
        state.set(State::Waiting);
        spawn_local(async move {
            let found = fetch(token).await;
            if generation.try_get_value() == Some(at) {
                state.try_set(found);
            }
        });
    };
    Effect::watch(
        move || token.get(),
        move |token, _, _| load(token.clone()),
        true,
    );

    let robots = SendWrapper::new(hide_from_robots());
    on_cleanup(move || {
        if let Some(meta) = robots.take() {
            meta.remove();
        }
    });

    let body = move || match state.get() {
        State::Waiting => view! {
            <div class="center-page">
                <Spinner />
            </div>
        }
        .into_any(),
        State::Missing => view! {
            <div class="center-page">
                <div class="card empty shared-missing">
                    <h1>{move || t!("shared-missing-title")}</h1>
                    <p class="muted">{move || t!("shared-missing-body")}</p>
                </div>
            </div>
        }
        .into_any(),
        State::Failed(message) => {
            let retry = Callback::new(move |()| load(token.get_untracked()));
            view! {
                <div class="center-page">
                    <ErrorBanner message=message on_retry=retry />
                </div>
            }
            .into_any()
        }
        State::Ready(found) => {
            // A file that didn't load comes again with the link's fresh
            // addresses, in a page made anew.
            let again = Callback::new(move |_: Callback<()>| load(token.get_untracked()));
            ready(found, again).into_any()
        }
    };

    view! {
        <div class="shared-page">
            <div class="shared-brand" role="img" aria-label=move || t!("app-name")>
                <Mark height=30 compact=true />
                <Wordmark height=19 />
            </div>
            {body}
        </div>
    }
}

/// The heading, where the link comes from and until when, and the posts.
/// `again` asks for the link anew.
fn ready(found: Found, again: Callback<Callback<()>>) -> impl IntoView {
    let Found {
        title,
        server,
        expires_at,
        items,
    } = found;
    let count = items.len();
    let named = Signal::derive(move || heading(title.as_deref(), count));
    // So the date doesn't wrap at its dashes.
    let until = expires_at.map(|at| {
        let date = when(&at, &Local);
        view! {
            " · "
            <span class="shared-until">{move || t!("shared-until", date = date.clone())}</span>
        }
    });
    let from = move || t!("shared-from", server = server.clone());
    let empty = || view! { <p class="muted">{move || t!("shared-empty")}</p> }.into_any();
    let posts = match layout(count) {
        Layout::Empty => empty(),
        Layout::One => match items.into_iter().next() {
            Some(item) => view! { <SharedOne item=item label=named on_retry=again /> }.into_any(),
            None => empty(),
        },
        Layout::Many => view! { <SharedGrid items=items /> }.into_any(),
    };
    view! {
        <header class="shared-head">
            <h1 class="page-title">{move || named.get()}</h1>
            <p class="muted">{from}{until}</p>
        </header>
        {posts}
    }
}

/// One post, on the viewer's stage: zoom, fullscreen and the light around
/// it, with no way back or to a neighbour.
#[component]
fn SharedOne(item: Item, label: Signal<String>, on_retry: Callback<Callback<()>>) -> impl IntoView {
    let zoom = RwSignal::new(Zoom::default());
    let full = RwSignal::new(false);
    let enter = RwSignal::new(None::<bool>);
    // Zooming in brings the original, as in the viewer.
    let original = RwSignal::new(false);
    Effect::new(move |_| {
        if zoom.with(Zoom::is_zoomed) && !original.get_untracked() {
            original.set(true);
        }
    });
    let src = {
        let item = item.clone();
        Signal::derive(move || item.shown(original.get()).to_owned())
    };
    let thumb = {
        let thumb = item.thumb.clone();
        Signal::derive(move || thumb.clone())
    };
    let shown = Signal::derive(move || {
        let label = label.get();
        Some(Shown {
            key: "shared".to_owned(),
            kind: item.kind,
            size: item.size,
            tint: None,
            alt: label.clone(),
            poster: item.sample.clone().or_else(|| item.thumb.clone()),
            label,
        })
    });
    view! {
        <div class="shared-one" class:full-open=move || full.get()>
            <PostGlow thumb=thumb />
            <Stage
                shown=shown
                src=src
                thumb=thumb
                zoom=zoom
                full=full
                first=true
                last=true
                enter=enter
                editing=false
                on_step=Callback::new(|_: bool| {})
                on_edit=Callback::new(|()| {})
                on_retry=on_retry
            />
        </div>
    }
}

/// Several posts, as tiles in the link's order, each opening the lightbox.
#[component]
fn SharedGrid(items: Vec<Item>) -> impl IntoView {
    let count = items.len();
    let open = RwSignal::new(None::<usize>);
    let tiles = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            view! {
                <SharedTile
                    item=item.clone()
                    index=index
                    on_open=Callback::new(move |()| open.set(Some(index)))
                />
            }
        })
        .collect_view();
    let items = StoredValue::new(items);
    let showing = Memo::new(move |_| open.with(Option::is_some));
    view! {
        <div class="shared-grid">{tiles}</div>
        <Show when=move || showing.get()>
            <Lightbox items=items open=open count=count />
        </Show>
    }
}

#[component]
fn SharedTile(item: Item, index: usize, on_open: Callback<()>) -> impl IntoView {
    let loaded = RwSignal::new(false);
    let picture = match item.thumb {
        Some(src) => view! {
            <img src=src alt="" loading="lazy" decoding="async" on:load=move |_| loaded.set(true) />
        }
        .into_any(),
        None => view! {
            <div class="thumb-placeholder">
                <Ico icon=Icon::Gallery size=40 />
            </div>
        }
        .into_any(),
    };
    let badge = match item.kind {
        MediaKind::Video => {
            let length = item.duration_ms.and_then(format_duration);
            Some(
                view! {
                    <span class="badge">
                        <Ico icon=Icon::Play size=16 />
                        {length}
                    </span>
                }
                .into_any(),
            )
        }
        MediaKind::Animation => {
            Some(view! { <span class="badge">{move || t!("badge-gif")}</span> }.into_any())
        }
        MediaKind::Image => None,
    };
    // The first screenful comes in one after another, as in the gallery.
    let delay = index.min(40) * 16;
    view! {
        <button
            type="button"
            class="thumb"
            class:loaded=loaded
            style=format!("--delay: {delay}ms")
            aria-label=move || t!("shared-open", n = index + 1)
            aria-haspopup="dialog"
            on:click=move |_| on_open.run(())
        >
            <div class="thumb-media">{picture}</div>
            {badge}
        </button>
    }
}

/// Puts the focus back on the tile the lightbox was opened from.
fn focus_tile(index: usize) {
    let tile = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| {
            d.query_selector(&format!(".shared-grid > :nth-child({})", index + 1))
                .ok()
                .flatten()
        })
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok());
    if let Some(tile) = tile {
        let _ = tile.focus();
    }
}

/// One post at a time over the whole window, with previous and next.
#[component]
fn Lightbox(
    items: StoredValue<Vec<Item>>,
    open: RwSignal<Option<usize>>,
    count: usize,
) -> impl IntoView {
    let root: NodeRef<html::Div> = NodeRef::new();
    root.on_load(|el| {
        request_animation_frame(move || {
            let _ = el.focus();
        });
    });
    // Closing puts the focus on the tile last shown, not the one opened.
    hold_focus(root, false);
    // A video's fullscreen takes the whole lightbox, so its buttons and keys
    // come along.
    let full = RwSignal::new(false);
    let toggle_full = Callback::new(move |()| {
        let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
            return;
        };
        if doc.fullscreen_element().is_some() {
            dom::call_quietly(&doc, "exitFullscreen");
        } else if let Some(el) = root.try_get_untracked().flatten() {
            dom::call_quietly(&el, "requestFullscreen");
        }
    });
    let at = move || open.get().unwrap_or(0);
    let close = move || {
        let from = open.get_untracked();
        open.set(None);
        if let Some(index) = from {
            request_animation_frame(move || focus_tile(index));
        }
    };
    let go = move |forward: bool| {
        if let Some(next) = open.get_untracked().and_then(|i| step(i, count, forward)) {
            open.set(Some(next));
            // The button pressed may go at an end: the keys stay with the box.
            if let Some(el) = root.get_untracked() {
                let _ = el.focus();
            }
        }
    };
    // On the document, so the keys work wherever the focus is. A video's
    // player hears them first, and keeps the ones it takes.
    let keys = web_sys::window().and_then(|w| w.document()).map(|doc| {
        let watches = [
            dom::listen(&doc, "keydown", false, move |e| {
                let Some(k) = e.dyn_ref::<KeyboardEvent>() else {
                    return;
                };
                if k.default_prevented() || k.ctrl_key() || k.meta_key() || k.alt_key() {
                    return;
                }
                match k.key().as_str() {
                    "Escape" if full.try_get_untracked() == Some(true) => {
                        toggle_full.try_run(());
                    }
                    "Escape" => close(),
                    "ArrowLeft" => go(false),
                    "ArrowRight" => go(true),
                    _ => return,
                }
                e.prevent_default();
            }),
            dom::listen(&doc, "fullscreenchange", true, move |_| {
                let on = web_sys::window()
                    .and_then(|w| w.document())
                    .is_some_and(|d| d.fullscreen_element().is_some());
                full.try_set(on);
            }),
        ];
        (watches, SendWrapper::new(doc))
    });
    on_cleanup(move || {
        // Escape closes the lightbox from inside this very listener, which
        // can't be dropped while it runs.
        set_timeout(
            move || {
                if let Some((watches, doc)) = keys {
                    for watch in watches {
                        watch.stop();
                    }
                    if doc.fullscreen_element().is_some() {
                        dom::call_quietly(&doc, "exitFullscreen");
                    }
                }
            },
            Duration::ZERO,
        );
    });
    let media = move || {
        let index = open.get()?;
        let item = items.with_value(|all| all.get(index).cloned())?;
        Some(match item.kind {
            MediaKind::Video => view! {
                <Player
                    src=item.file
                    poster=item.sample.or(item.thumb)
                    style=fit_style(item.size)
                    full=full
                    on_full=toggle_full
                    keys=true
                    autoplay=!dom::reduced_motion() && !power::active_untracked()
                />
            }
            .into_any(),
            MediaKind::Image | MediaKind::Animation => view! {
                <img src=item.shown(false).to_owned() alt="" decoding="async" />
            }
            .into_any(),
        })
    };
    view! {
        <div
            class="lightbox"
            role="dialog"
            aria-modal="true"
            aria-label=move || t!("shared-open", n = at() + 1)
            tabindex="-1"
            node_ref=root
            on:click=move |_| close()
        >
            <div class="lightbox-media" on:click=|e| e.stop_propagation()>
                {media}
            </div>
            <button
                type="button"
                class="lightbox-btn close"
                title=move || t!("share-close")
                aria-label=move || t!("share-close")
                on:click=move |e| {
                    e.stop_propagation();
                    close();
                }
            >
                <Ico icon=Icon::Close />
            </button>
            <Show when=move || step(at(), count, false).is_some()>
                <button
                    type="button"
                    class="lightbox-btn prev"
                    title=move || t!("shared-previous")
                    aria-label=move || t!("shared-previous")
                    on:click=move |e| {
                        e.stop_propagation();
                        go(false);
                    }
                >
                    <Ico icon=Icon::ChevronLeft />
                </button>
            </Show>
            <Show when=move || step(at(), count, true).is_some()>
                <button
                    type="button"
                    class="lightbox-btn next"
                    title=move || t!("shared-next")
                    aria-label=move || t!("shared-next")
                    on:click=move |e| {
                        e.stop_propagation();
                        go(true);
                    }
                >
                    <Ico icon=Icon::ChevronRight />
                </button>
            </Show>
            <span class="lightbox-count" aria-hidden="true">
                {move || format!("{} / {count}", at() + 1)}
            </span>
        </div>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_lightbox_and_its_buttons_are_not_glass() {
        let source = include_str!("public.rs");
        assert!(source.contains(r#"class="lightbox""#));
        let css = include_str!("../../../style/pages/share.css");
        assert!(!css.contains("backdrop-filter"));
        let (_, button) = css.split_once("\n.lightbox-btn {").unwrap();
        assert!(
            button
                .split_once('}')
                .unwrap()
                .0
                .contains("background: var(--scrim)")
        );
        // The page under it stays where it is.
        assert!(css.contains("html:has(.lightbox) { overflow-y: hidden; }"));
    }
}
