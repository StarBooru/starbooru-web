//! The viewer at `/post/:id?q=…&seed=…`: the post on its stage, the bar of
//! what can be done with it, and its tags and details beside or below.
//! Previous and next walk the gallery's results for the same search. The
//! tags card turns into the tag editor in place.

mod edit;
mod glow;
mod side;
mod stage;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_params_map, use_query_map};
use send_wrapper::SendWrapper;
use starbooru_core::{EditKind, MediaKind, Neighbors, Post, edit_kind};

use self::edit::EditCard;
use self::glow::PostGlow;
use self::side::{DetailsCard, TagsCard};
use self::stage::Stage;
use super::gallery;
use crate::api::{self, Failure};
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::download::DownloadLink;
use crate::components::icons::{Ico, Icon};
use crate::components::snackbar::{notify, notify_undo};
use crate::components::tag_menu::TagPress;
use crate::logic::tags::may_edit_post;
use crate::logic::viewer::{
    back_through_history, download_name, needs_original, neighbour_href, queued, shown_url,
};
use crate::logic::zoom::Zoom;
use crate::route;
use crate::store;

/// Posts kept for the viewer to show at once: the tile just clicked and the
/// neighbours fetched ahead.
const KEPT: usize = 12;
/// Pictures fetched ahead, held so the browser keeps them.
const WARM: usize = 4;

thread_local! {
    static POSTS: RefCell<VecDeque<Arc<Post>>> = const { RefCell::new(VecDeque::new()) };
    static WARMED: RefCell<VecDeque<web_sys::HtmlImageElement>> = const { RefCell::new(VecDeque::new()) };
    /// The search of the gallery the viewer was opened from, for its Back.
    static OPENED_FROM: RefCell<Option<String>> = const { RefCell::new(None) };
}

fn keep(post: Arc<Post>) {
    POSTS.with_borrow_mut(|kept| {
        kept.retain(|p| p.id != post.id);
        kept.push_back(post);
        while kept.len() > KEPT {
            kept.pop_front();
        }
    });
}

fn kept(id: i64) -> Option<Arc<Post>> {
    POSTS.with_borrow(|kept| kept.iter().find(|p| p.id == id).cloned())
}

fn forget(id: i64) {
    POSTS.with_borrow_mut(|kept| kept.retain(|p| p.id != id));
}

/// The gallery is opening `post`, from its results for `q`: the viewer shows
/// it at once while it loads the rest.
pub fn opening(post: Arc<Post>, q: String) {
    keep(post);
    OPENED_FROM.with_borrow_mut(|from| *from = Some(q));
}

/// A server-relative address made absolute.
fn absolute(path: &str) -> String {
    api::client().map_or_else(|| path.to_owned(), |c| c.absolute(path))
}

/// Asks for `p`'s picture ahead, so stepping to it shows it at once.
fn warm(p: &Post) {
    let path = match p.kind {
        MediaKind::Video => p.thumb_url.as_deref(),
        MediaKind::Image | MediaKind::Animation => {
            Some(shown_url(p, store::prefs_untracked().originals))
        }
    };
    let Some(path) = path else {
        return;
    };
    let Ok(img) = web_sys::HtmlImageElement::new() else {
        return;
    };
    img.set_decoding("async");
    img.set_src(&absolute(path));
    WARMED.with_borrow_mut(|warm| {
        warm.push_back(img);
        while warm.len() > WARM {
            warm.pop_front();
        }
    });
}

/// Fetches the posts around one ahead, with their pictures.
fn fetch_ahead(n: Neighbors) {
    for id in [n.next, n.prev].into_iter().flatten() {
        if let Some(p) = kept(id).filter(|p| !p.tag_details.is_empty() || p.tags.is_empty()) {
            warm(&p);
            continue;
        }
        spawn_local(async move {
            if let Ok(p) = api::call(move |c| async move { c.post(id).await }).await {
                warm(&p);
                keep(Arc::new(p));
            }
        });
    }
}

/// What the page can show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Waiting,
    Failed,
    Ready,
}

#[component]
pub fn PostView() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let id =
        Memo::new(move |_| params.with(|p| p.get("id").and_then(|s| s.trim().parse::<i64>().ok())));
    let q = Memo::new(move |_| query.with(|m| m.get("q").unwrap_or_default()));
    let seed =
        Memo::new(move |_| query.with(|m| m.get("seed").and_then(|s| s.parse::<i64>().ok())));
    let opened_from = StoredValue::new(OPENED_FROM.with_borrow_mut(Option::take));

    // The post on the page: the one asked for, or a quicker copy of it, or the
    // one before while it comes.
    let post = RwSignal::new(None::<Arc<Post>>);
    let problem = RwSignal::new(None::<Failure>);
    let neighbours = RwSignal::new(None::<(i64, Neighbors)>);
    let zoom = RwSignal::new(Zoom::default());
    let full = RwSignal::new(false);
    let original = RwSignal::new(false);
    let enter = RwSignal::new(None::<bool>);
    let busy = RwSignal::new(false);
    let action_error = RwSignal::new(None::<String>);
    // The tags card is the tag editor.
    let editing = RwSignal::new(false);
    let steps = StoredValue::new(0_i32);
    let generation = StoredValue::new(0_u64);

    let navigate = SendWrapper::new(use_navigate());
    let go = Callback::new(move |(to, replace): (String, bool)| {
        let options = NavigateOptions {
            replace,
            scroll: !replace,
            ..NavigateOptions::default()
        };
        (*navigate)(&to, options);
    });

    // Back to the gallery: through history when it's the page before, so it
    // comes back as it was.
    let back = move || {
        let q = q.get_untracked();
        let history = web_sys::window().and_then(|w| w.history().ok());
        let through = back_through_history(opened_from.get_value().as_deref(), &q)
            && history
                .as_ref()
                .and_then(|h| h.length().ok())
                .is_some_and(|n| n > 1);
        match history.filter(|_| through) {
            Some(history) => {
                let _ = history.back();
            }
            None => go.run((route::gallery(&q), false)),
        }
    };

    // The address of the post before or after, in the same search. Stepping
    // replaces the address, so Back leaves the viewer.
    let href = move |n: Neighbors, forward: bool| {
        neighbour_href(n, forward, &q.get_untracked(), seed.get_untracked())
    };
    let step_to = move |to: String, forward: bool| {
        enter.set(Some(forward));
        go.run((to, true));
    };

    // Presses wait here until the post's neighbours are known.
    let drain = move || {
        let pending = steps.get_value();
        if pending == 0 {
            return;
        }
        let Some(current) = id.get_untracked() else {
            return;
        };
        let Some((_, n)) = neighbours.get_untracked().filter(|(of, _)| *of == current) else {
            return;
        };
        match href(n, pending > 0) {
            Some(to) => {
                steps.set_value(pending - pending.signum());
                step_to(to, pending > 0);
            }
            None => steps.set_value(0),
        }
    };
    let press = Callback::new(move |forward: bool| {
        steps.update_value(|s| *s = queued(*s, forward));
        drain();
    });

    let load = move |id: i64| {
        generation.update_value(|g| *g = g.wrapping_add(1));
        let at = generation.get_value();
        zoom.set(Zoom::default());
        original.set(false);
        problem.set(None);
        action_error.set(None);
        editing.set(false);
        gallery::viewed(&q.get_untracked(), id);
        if post.with_untracked(|p| p.as_ref().map(|p| p.id)) != Some(id)
            && let Some(quick) = kept(id)
        {
            post.set(Some(quick));
        }
        spawn_local(async move {
            let result = api::fetch(move |c| async move { c.post(id).await }).await;
            if generation.try_get_value() != Some(at) {
                return;
            }
            match result {
                Ok(p) => {
                    let p = Arc::new(p);
                    keep(Arc::clone(&p));
                    post.set(Some(p));
                }
                Err(failure) => problem.set(Some(failure)),
            }
        });
        let (search, shuffle) = (q.get_untracked(), seed.get_untracked());
        spawn_local(async move {
            let result =
                api::call(move |c| async move { c.neighbors(id, &search, shuffle).await }).await;
            if generation.try_get_value() != Some(at) {
                return;
            }
            // Without them, there's nowhere to step to.
            let n = result.unwrap_or_default();
            neighbours.set(Some((id, n)));
            drain();
            fetch_ahead(n);
        });
    };
    Effect::watch(
        move || id.get(),
        move |id, _, _| match id {
            Some(id) => load(*id),
            None => problem.set(Some(Failure {
                message: String::new(),
                missing: true,
            })),
        },
        true,
    );

    let mode = Memo::new(move |_| {
        let shown = post.with(|p| p.as_ref().map(|p| p.id));
        match (shown, problem.with(Option::is_some)) {
            (Some(shown), true) if Some(shown) != id.get() => Mode::Failed,
            (None, true) => Mode::Failed,
            (None, false) => Mode::Waiting,
            (Some(_), _) => Mode::Ready,
        }
    });

    // What the stage shows: the smaller copy unless the original is wanted.
    let originals = Memo::new(|_| store::prefs().originals);
    let sampled = Memo::new(move |_| {
        post.with(|p| {
            p.as_ref()
                .is_some_and(|p| p.kind != MediaKind::Video && p.sample_url.is_some())
        }) && !original.get()
            && !originals.get()
    });
    let src = Memo::new(move |_| {
        let wanted = !sampled.get();
        post.with(|p| {
            p.as_ref()
                .map(|p| absolute(shown_url(p, wanted)))
                .unwrap_or_default()
        })
    });
    let thumb = Memo::new(move |_| {
        post.with(|p| {
            p.as_ref()
                .and_then(|p| p.thumb_url.as_deref())
                .map(absolute)
        })
    });
    // Zooming in, or a screen with more pixels than the copy, brings the original.
    Effect::new(move |_| {
        if !sampled.get() {
            return;
        }
        let grown = zoom.with(Zoom::is_zoomed);
        let sharp_enough = !full.get() || {
            let window = web_sys::window();
            let size = window.as_ref().and_then(|w| {
                let width = w.inner_width().ok()?.as_f64()?;
                let height = w.inner_height().ok()?.as_f64()?;
                Some((width, height))
            });
            let dpr = window.map(|w| w.device_pixel_ratio());
            !post.with_untracked(|p| p.as_ref().is_some_and(|p| needs_original(p, size, dpr)))
        };
        if grown || !sharp_enough {
            original.set(true);
        }
    });

    let known = move || {
        let current = id.get();
        neighbours.with(|n| n.filter(|(of, _)| Some(*of) == current).map(|(_, n)| n))
    };
    // Until the neighbours are known, either way may be open.
    let first = Signal::derive(move || known().is_some_and(|n| n.prev.is_none()));
    let last = Signal::derive(move || known().is_some_and(|n| n.next.is_none()));

    let delete = move || {
        let Some(p) = post.get_untracked() else {
            return;
        };
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        action_error.set(None);
        let id = p.id;
        let next = neighbours
            .get_untracked()
            .filter(|(of, _)| *of == id)
            .and_then(|(_, n)| href(n, true));
        spawn_local(async move {
            let result = api::call(move |c| async move { c.delete_post(id).await }).await;
            busy.try_set(false);
            match result {
                Ok(()) => {
                    forget(id);
                    gallery::forget_post(id);
                    notify_undo(t!("press-deleted"), t!("press-undo"), move || undo(id));
                    // Only if the page is still on it.
                    if post.try_with_untracked(|p| p.as_ref().map(|p| p.id)) == Some(Some(id)) {
                        match next {
                            Some(next) => step_to(next, true),
                            None => back(),
                        }
                    }
                }
                Err(message) => {
                    action_error.try_set(Some(message));
                }
            }
        });
    };

    let restore = move || {
        let Some(p) = post.get_untracked() else {
            return;
        };
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        action_error.set(None);
        let id = p.id;
        spawn_local(async move {
            let result = api::call(move |c| async move { c.restore_post(id).await }).await;
            busy.try_set(false);
            match result {
                Ok(_) => {
                    gallery::forget_lists();
                    post.try_update(|p| {
                        if let Some(p) = p.as_mut().filter(|p| p.id == id) {
                            Arc::make_mut(p).deleted = false;
                            keep(Arc::clone(p));
                        }
                    });
                }
                Err(message) => {
                    action_error.try_set(Some(message));
                }
            }
        });
    };

    // Anyone signed in may edit a post, unless it's in the recycle bin.
    let editable = Memo::new(move |_| {
        let user = store::session().map(|s| s.user);
        post.with(|p| {
            p.as_ref()
                .is_some_and(|p| may_edit_post(user.as_ref(), p.deleted))
        })
    });
    let start_edit = Callback::new(move |()| {
        // Not while the page still shows the post before.
        let here = post.with_untracked(|p| p.as_ref().map(|p| p.id)) == id.get_untracked();
        if here && editable.get_untracked() {
            editing.set(true);
        }
    });
    let saved = Callback::new(move |p: Post| {
        let p = Arc::new(p);
        keep(Arc::clone(&p));
        post.set(Some(p));
        // The gallery's kept results and the suggestions show the old tags.
        gallery::forget_lists();
        api::forget_tags();
        editing.set(false);
    });

    let failed = move || {
        let failure = problem.get().unwrap_or(Failure {
            message: String::new(),
            missing: false,
        });
        let wanted = id.get();
        let gallery_href = move || route::gallery(&q.get());
        if failure.missing {
            let title = move || match wanted {
                Some(id) => t!("post-missing-title", id = id),
                None => t!("notfound-title"),
            };
            view! {
                <div class="center-page">
                    <div class="card empty">
                        <h1 class="break">{title}</h1>
                        <p class="muted">{move || t!("post-missing-body")}</p>
                        <a class="btn" href=gallery_href>
                            {move || t!("notfound-back")}
                        </a>
                    </div>
                </div>
            }
            .into_any()
        } else {
            let retry = Callback::new(move |()| {
                if let Some(id) = id.get_untracked() {
                    load(id);
                }
            });
            view! {
                <div class="page narrow">
                    <ErrorBanner message=failure.message on_retry=retry />
                    <a class="btn tonal" href=gallery_href>
                        <Ico icon=Icon::ArrowBack />
                        {move || t!("notfound-back")}
                    </a>
                </div>
            }
            .into_any()
        }
    };

    let ready = move || {
        let post = Signal::derive(move || post.get());
        view! {
            <div class="page post-layout" class:full-open=move || full.get()>
                <PostGlow thumb=thumb />
                <TagPress q=q />
                <div class="post-main">
                    <Stage
                        post=post
                        src=src
                        thumb=thumb
                        zoom=zoom
                        full=full
                        first=first
                        last=last
                        enter=enter
                        editing=editing
                        on_step=press
                        on_back=Callback::new(move |()| back())
                        on_edit=start_edit
                    />
                    <PostBar post=post busy=busy on_delete=Callback::new(move |()| delete()) />
                    {move || {
                        problem
                            .get()
                            .map(|f| {
                                let retry = Callback::new(move |()| {
                                    if let Some(id) = id.get_untracked() {
                                        load(id);
                                    }
                                });
                                view! { <ErrorBanner message=f.message on_retry=retry /> }
                            })
                    }}
                    {move || action_error.get().map(|message| view! { <ErrorBanner message=message /> })}
                    <Show when=move || post.with(|p| p.as_ref().is_some_and(|p| p.deleted))>
                        <div class="banner warn">
                            <span>{move || t!("post-in-trash")}</span>
                            <button
                                type="button"
                                class="btn tonal"
                                disabled=move || busy.get()
                                on:click=move |_| restore()
                            >
                                <Ico icon=Icon::Restore />
                                {move || t!("trash-restore")}
                            </button>
                        </div>
                    </Show>
                </div>
                <aside class="side">
                    {move || {
                        if editing.get() {
                            post.get_untracked()
                                .map(|p| {
                                    view! {
                                        <EditCard
                                            post=p
                                            on_cancel=Callback::new(move |()| editing.set(false))
                                            on_saved=saved
                                        />
                                    }
                                        .into_any()
                                })
                        } else {
                            Some(
                                view! { <TagsCard post=post editable=editable on_edit=start_edit /> }
                                    .into_any(),
                            )
                        }
                    }}
                    <DetailsCard
                        post=post
                        sampled=sampled
                        on_original=Callback::new(move |()| original.set(true))
                    />
                </aside>
            </div>
        }
    };

    view! {
        {move || match mode.get() {
            Mode::Waiting => {
                view! {
                    <div class="center-page">
                        <Spinner />
                    </div>
                }
                    .into_any()
            }
            Mode::Failed => failed().into_any(),
            Mode::Ready => ready().into_any(),
        }}
    }
}

/// What can be done with the post, apart from looking at it: an opinion, what
/// leaves with the file, and, away from the rest, the one that removes it.
/// Votes, sharing and the editor come later; their buttons hold their places.
#[component]
fn PostBar(
    post: Signal<Option<Arc<Post>>>,
    busy: RwSignal<bool>,
    on_delete: Callback<()>,
) -> impl IntoView {
    let bar = move || {
        let p = post.get()?;
        let deleted = p.deleted;
        let edit = edit_kind(p.kind, &p.mime)
            .filter(|k| *k == EditKind::Video || p.width.is_some() && p.height.is_some())
            .filter(|_| !deleted)
            .map(|kind| {
                let icon = match kind {
                    EditKind::Video => Icon::Cut,
                    EditKind::Image => Icon::Crop,
                };
                let label = move || match kind {
                    EditKind::Video => t!("editor-video-short"),
                    EditKind::Image => t!("editor-image-short"),
                };
                view! {
                    <button
                        type="button"
                        class="icon-btn"
                        disabled=true
                        title=move || t!("editor-open")
                        aria-label=move || t!("editor-open")
                    >
                        <Ico icon=icon />
                        <span class="bar-label">{label}</span>
                    </button>
                }
            });
        let share = (!deleted).then(|| {
            view! {
                <button
                    type="button"
                    class="icon-btn"
                    disabled=true
                    title=move || t!("share-button")
                    aria-label=move || t!("share-button")
                >
                    <Ico icon=Icon::Share />
                    <span class="bar-label">{move || t!("share-button")}</span>
                </button>
            }
        });
        let remove = (!deleted).then(|| {
            view! {
                <button
                    type="button"
                    class="icon-btn danger bar-delete"
                    title=move || t!("post-delete-title")
                    aria-label=move || t!("post-delete")
                    disabled=move || busy.get()
                    on:click=move |_| on_delete.run(())
                >
                    <Ico icon=Icon::Delete />
                </button>
            }
        });
        let (up, down) = (p.my_vote > 0, p.my_vote < 0);
        Some(view! {
            <div class="vote">
                <button
                    type="button"
                    class="icon-btn"
                    class:on=up
                    disabled=true
                    aria-pressed=up.to_string()
                    aria-label=move || t!("post-upvote")
                >
                    <Ico icon=Icon::ThumbUp />
                </button>
                <span class="score">{p.score}</span>
                <button
                    type="button"
                    class="icon-btn"
                    class:on=down
                    disabled=true
                    aria-pressed=down.to_string()
                    aria-label=move || t!("post-downvote")
                >
                    <Ico icon=Icon::ThumbDown />
                </button>
            </div>
            <div class="bar-acts">
                <DownloadLink
                    href=absolute(&p.file_url)
                    name=download_name(&p.sha256, &p.file_url, &p.mime)
                />
                {share}
                {edit}
            </div>
            {remove}
        })
    };
    view! {
        <div class="post-bar glass float" role="toolbar" aria-label=move || t!("post-actions")>
            {bar}
        </div>
    }
}

/// From the notice after a delete, which outlives the page.
fn undo(id: i64) {
    spawn_local(async move {
        match api::call(move |c| async move { c.restore_post(id).await }).await {
            Ok(_) => gallery::forget_lists(),
            Err(message) => notify(message),
        }
    });
}
