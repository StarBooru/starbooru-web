//! The viewer at `/post/:id?q=…&seed=…`: the post on its stage, the bar of
//! what can be done with it, and its tags and details beside or below.
//! Previous and next walk the gallery's results for the same search. Each
//! post that loads goes first in the history Explore lists. The tags card
//! turns into the tag editor in place, and the people card (shown while
//! People are on in Settings) into theirs; the media editor opens over the
//! whole window.

mod edit;
mod editor;
pub(crate) mod glow;
mod people;
mod related;
mod side;
pub(crate) mod stage;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_params_map, use_query_map};
use send_wrapper::SendWrapper;
use starbooru_core::{EditKind, MediaKind, Neighbors, Post};
use wasm_bindgen::JsCast;

use self::edit::EditCard;
use self::editor::{EditorSheet, Saved, pause_stage_video};
use self::glow::PostGlow;
use self::people::PeopleCard;
use self::related::Related;
use self::side::{DetailsCard, TagsCard};
use self::stage::{Shown, Stage};
use super::share::ShareSheet;
use super::{gallery, trash};
use crate::api::{self, Failure};
use crate::components::banner::{ErrorBanner, Spinner};
use crate::components::download::DownloadLink;
use crate::components::icons::{Ico, Icon};
use crate::components::menu::{MenuAt, beside};
use crate::components::reverse::ReverseMenu;
use crate::components::snackbar::{notify, notify_undo};
use crate::components::tag_menu::TagPress;
use crate::logic::editor::offer;
use crate::logic::people::{PeopleShown, closes_when_off, people_shown};
use crate::logic::reverse::offered;
use crate::logic::tags::may_edit_post;
use crate::logic::viewer::{
    back_through_history, download_name, needs_original, neighbour_href, queued, shown_url,
};
use crate::logic::votes::{next_vote, rescored};
use crate::logic::zoom::Zoom;
use crate::{power, route, store};

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

/// `post` is back from the recycle bin: a copy kept to show at once says so.
pub fn restored(post: &Arc<Post>) {
    POSTS.with_borrow_mut(|kept| {
        for p in kept.iter_mut().filter(|p| p.id == post.id) {
            *p = Arc::clone(post);
        }
    });
}

/// Drops the copies kept of posts changed elsewhere, by a mass edit or a
/// purge, so the viewer doesn't show them as they were.
pub fn forget_kept(ids: &[i64]) {
    POSTS.with_borrow_mut(|kept| kept.retain(|p| !ids.contains(&p.id)));
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

/// Fetches the posts around one ahead, with their pictures, unless the
/// battery saver is on.
fn fetch_ahead(n: Neighbors) {
    if power::active_untracked() {
        return;
    }
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
    // The tags card is the tag editor, and the people card theirs, each
    // with whether leaving now would lose what it holds.
    let editing = RwSignal::new(false);
    let people_editing = RwSignal::new(false);
    let tags_unsaved = RwSignal::new(false);
    let people_unsaved = RwSignal::new(false);
    // Settings in another tab can turn People off while this page is open:
    // an edit in progress stays until it's saved or cancelled.
    let people_on = Memo::new(|_| store::prefs().people);
    let people = Memo::new(move |_| people_shown(people_on.get(), people_editing.get()));
    Effect::watch(
        move || people_on.get(),
        move |on, _, _| {
            if !*on
                && closes_when_off(
                    people_editing.get_untracked(),
                    people_unsaved.get_untracked(),
                )
            {
                people_editing.set(false);
            }
        },
        false,
    );
    // The media editor is open, and whether it's fading out, when the stage
    // under it takes keys again.
    let media_editing = RwSignal::new(false);
    let editor_leaving = RwSignal::new(false);
    // The share sheet is open, for this post.
    let sharing = RwSignal::new(None::<i64>);
    // The menu of sites to look the picture up on is open, there.
    let reversing = RwSignal::new(None::<MenuAt>);
    // Trimming needs ffmpeg on the server.
    let info = LocalResource::new(api::server_info);
    let video_editing =
        Signal::derive(move || info.get().flatten().is_some_and(|i| i.video_editing));
    let steps = StoredValue::new(0_i32);
    let generation = StoredValue::new(0_u64);
    // The post whose neighbours are being asked for, so a press doesn't ask
    // twice.
    let asking = StoredValue::new(None::<i64>);
    // Votes show at once and go to the server one at a time, the latest
    // press winning. `vote_before` is the post's score and vote before the first
    // press the server hasn't answered, to put back if it fails.
    let vote_wanted = StoredValue::new(None::<(i64, i16)>);
    let vote_sending = StoredValue::new(false);
    let vote_before = StoredValue::new(None::<(i64, i32, i16)>);

    let navigate = SendWrapper::new(use_navigate());
    // For the notice's Undo, which outlives the page.
    let navigate_later = SendWrapper::new(use_navigate());
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
    // Asks for post `id`'s neighbours. Not knowing them leaves both ways
    // open, and the next press asks again; a press waiting on them hears why
    // they didn't come.
    let ask_neighbours = move |id: i64| {
        let at = generation.get_value();
        asking.set_value(Some(id));
        let (search, shuffle) = (q.get_untracked(), seed.get_untracked());
        spawn_local(async move {
            let result =
                api::call(move |c| async move { c.neighbors(id, &search, shuffle).await }).await;
            if generation.try_get_value() != Some(at) {
                return;
            }
            asking.set_value(None);
            match result {
                Ok(n) => {
                    neighbours.set(Some((id, n)));
                    drain();
                    fetch_ahead(n);
                }
                Err(message) => {
                    if steps.get_value() != 0 {
                        steps.set_value(0);
                        action_error.set(Some(t!("post-neighbours-failed", reason = message)));
                    }
                }
            }
        });
    };
    // An editor's changes would go with the post: they're saved or
    // cancelled first.
    let unsaved = move || {
        (editing.get_untracked() && tags_unsaved.get_untracked())
            || (people_editing.get_untracked() && people_unsaved.get_untracked())
    };
    let press = Callback::new(move |forward: bool| {
        if unsaved() {
            notify(t!("post-unsaved"));
            return;
        }
        steps.update_value(|s| *s = queued(*s, forward));
        if let Some(current) = id.get_untracked() {
            let known = neighbours.with_untracked(|n| n.is_some_and(|(of, _)| of == current));
            if !known && asking.get_value() != Some(current) {
                action_error.set(None);
                ask_neighbours(current);
            }
        }
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
        people_editing.set(false);
        media_editing.set(false);
        sharing.set(None);
        reversing.set(None);
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
                    // Explore lists what was opened, as long as it's live.
                    if p.deleted {
                        store::forget_opened(&[id]);
                    } else {
                        store::opened_post(id);
                    }
                    let p = Arc::new(p);
                    keep(Arc::clone(&p));
                    post.set(Some(p));
                }
                Err(failure) => {
                    if failure.missing {
                        store::forget_opened(&[id]);
                    }
                    problem.set(Some(failure));
                }
            }
        });
        ask_neighbours(id);
    };
    // The post again, for fresh addresses when its file didn't load, and
    // then `reload` loads the file. The editors stay as they are.
    let refetch = Callback::new(move |reload: Callback<()>| {
        let Some(id) = id.get_untracked() else {
            return;
        };
        let at = generation.get_value();
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
                    reload.try_run(());
                }
                Err(failure) => problem.set(Some(failure)),
            }
        });
    });
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
        // Not while the page still shows the post before.
        if busy.get_untracked() || Some(p.id) != id.get_untracked() {
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
                    notify_undo(t!("press-deleted"), t!("press-undo"), move || {
                        trash::undo_delete(vec![id]);
                    });
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
            // The way the bin's page restores, so the two agree.
            let trash::Restored { back, failed } = trash::restore_posts(vec![id]).await;
            busy.try_set(false);
            if let Some(fresh) = back.into_iter().next() {
                keep(Arc::clone(&fresh));
                post.try_update(|p| {
                    if let Some(p) = p.as_mut().filter(|p| p.id == id) {
                        *p = fresh;
                    }
                });
            } else if let Some(refused) = failed.into_iter().next() {
                action_error.try_set(Some(refused.message));
            }
        });
    };

    // Shows `score` and `vote` on post `of`, if the page still shows it.
    let show_vote = move |of: i64, score: i32, vote: i16| {
        post.try_update(|p| {
            if let Some(p) = p.as_mut().filter(|p| p.id == of) {
                let p = Arc::make_mut(p);
                p.score = score;
                p.my_vote = vote;
            }
        });
    };
    let send_votes = move || {
        vote_sending.set_value(true);
        spawn_local(async move {
            while let Some((of, vote)) = vote_wanted.try_get_value().flatten() {
                let result = api::call(move |c| async move { c.vote(of, vote).await }).await;
                let still = vote_wanted.try_get_value().flatten();
                match result {
                    Ok(fresh) => {
                        let fresh = Arc::new(fresh);
                        keep(Arc::clone(&fresh));
                        gallery::update_post(Arc::clone(&fresh));
                        if still != Some((of, vote)) {
                            // Pressed again meanwhile: that one goes next,
                            // and a failure puts back what the server has now.
                            if still.is_some_and(|(id, _)| id == of) {
                                vote_before.try_set_value(Some((of, fresh.score, fresh.my_vote)));
                            }
                            continue;
                        }
                        vote_wanted.try_set_value(None);
                        vote_before.try_set_value(None);
                        post.try_update(|p| {
                            if let Some(p) = p.as_mut().filter(|p| p.id == of) {
                                *p = fresh;
                            }
                        });
                    }
                    Err(message) => {
                        if still.is_some_and(|(id, _)| id != of) {
                            continue;
                        }
                        vote_wanted.try_set_value(None);
                        if let Some((_, score, mine)) = vote_before
                            .try_update_value(Option::take)
                            .flatten()
                            .filter(|(id, ..)| *id == of)
                        {
                            show_vote(of, score, mine);
                        }
                        action_error.try_set(Some(t!("vote-failed", reason = message)));
                    }
                }
                break;
            }
            vote_sending.try_set_value(false);
        });
    };
    // Up (`1`) or down (`-1`); pressing the vote already cast takes it back.
    let vote = Callback::new(move |pressed: i16| {
        let Some(p) = post.get_untracked() else {
            return;
        };
        // Not while the page still shows the post before, nor in the bin.
        if Some(p.id) != id.get_untracked() || p.deleted {
            return;
        }
        let new = next_vote(p.my_vote, pressed);
        if vote_before.get_value().map(|(of, ..)| of) != Some(p.id) {
            vote_before.set_value(Some((p.id, p.score, p.my_vote)));
        }
        show_vote(p.id, rescored(p.score, p.my_vote, new), new);
        action_error.set(None);
        vote_wanted.set_value(Some((p.id, new)));
        if !vote_sending.get_value() {
            send_votes();
        }
    });

    // Anyone signed in may edit a post, unless it's in the recycle bin.
    let editable = Memo::new(move |_| {
        let user = store::session().map(|s| s.user);
        post.with(|p| {
            p.as_ref()
                .is_some_and(|p| may_edit_post(user.as_ref(), p.deleted))
        })
    });
    // Not while the page still shows the post before.
    let here = move || post.with_untracked(|p| p.as_ref().map(|p| p.id)) == id.get_untracked();
    let start_edit = Callback::new(move |()| {
        if here() && editable.get_untracked() {
            editing.set(true);
        }
    });
    let start_people = Callback::new(move |()| {
        if here() && editable.get_untracked() && people_on.get_untracked() {
            people_editing.set(true);
        }
    });
    // The post as an editor saved it. Whether the page is still on it.
    let take = move |p: Post| {
        let p = Arc::new(p);
        keep(Arc::clone(&p));
        // The gallery's kept results and the suggestions show the old tags.
        gallery::forget_lists();
        api::forget_tags();
        let here = post.with_untracked(|shown| shown.as_ref().map(|s| s.id)) == Some(p.id);
        if here {
            post.set(Some(p));
        }
        here
    };
    let saved = Callback::new(move |p: Post| {
        if take(p) {
            editing.set(false);
        }
    });
    let people_saved = Callback::new(move |p: Post| {
        if take(p) {
            people_editing.set(false);
        }
    });

    // Not while the page still shows the post before, nor one in the bin.
    let share = Callback::new(move |()| {
        let shown = post.with_untracked(|p| p.as_ref().filter(|p| !p.deleted).map(|p| p.id));
        if shown.is_some() && shown == id.get_untracked() {
            sharing.set(shown);
        }
    });
    let reverse = Callback::new(move |at: MenuAt| {
        let shown = post.with_untracked(|p| p.as_deref().filter(|p| offered(p)).map(|p| p.id));
        if shown.is_some() && shown == id.get_untracked() {
            reversing.set(Some(at));
        }
    });

    let open_editor = Callback::new(move |()| {
        let offered = post.with_untracked(|p| {
            p.as_ref()
                .and_then(|p| offer(p, video_editing.get_untracked()))
        });
        if here() && offered.is_some() && !media_editing.get_untracked() {
            pause_stage_video();
            editor_leaving.set(false);
            media_editing.set(true);
        }
    });
    // The edit is a new post: the page goes to it, and the notice can take it
    // back.
    let media_saved = Callback::new(move |saved: Saved| {
        media_editing.set(false);
        if saved.binned {
            forget(saved.original);
            gallery::forget_post(saved.original);
        }
        gallery::forget_lists();
        let q = q.get_untracked();
        go.run((route::post(saved.id, &q, None), false));
        let message = if saved.binned {
            t!("editor-saved-binned")
        } else {
            t!("editor-saved")
        };
        let navigate = (*navigate_later).clone();
        notify_undo(message, t!("editor-undo-save"), move || {
            undo_edit(saved, q.clone(), navigate.clone());
        });
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

    let shown = Memo::new(move |_| post.with(|p| p.as_deref().map(Shown::of)));
    let ready = move || {
        let post = Signal::derive(move || post.get());
        view! {
            <div class="page post-layout" class:full-open=move || full.get()>
                <PostGlow thumb=thumb />
                <TagPress q=q />
                <div class="post-main">
                    <Stage
                        shown=shown
                        src=src
                        thumb=thumb
                        zoom=zoom
                        full=full
                        first=first
                        last=last
                        enter=enter
                        editing=Signal::derive(move || {
                            editing.get() || people_editing.get()
                                || media_editing.get() && !editor_leaving.get()
                                || sharing.with(Option::is_some)
                                || reversing.with(Option::is_some)
                        })
                        on_step=press
                        on_back=Callback::new(move |()| back())
                        on_edit=start_edit
                        on_retry=refetch
                    />
                    <PostBar
                        post=post
                        busy=busy
                        video_editing=video_editing
                        on_vote=vote
                        on_delete=Callback::new(move |()| delete())
                        on_edit_media=open_editor
                        on_share=share
                        on_reverse=reverse
                    />
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
                    <Related
                        id=id
                        held=Signal::derive(move || {
                            editing.get() && tags_unsaved.get()
                                || people_editing.get() && people_unsaved.get()
                        })
                        on_open=Callback::new(move |p: Arc<Post>| {
                            keep(p);
                            enter.set(None);
                        })
                    />
                </div>
                <aside class="side">
                    {move || {
                        if editing.get() {
                            post.get_untracked()
                                .map(|p| {
                                    view! {
                                        <EditCard
                                            post=p
                                            current=post
                                            unsaved=tags_unsaved
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
                    <Show when=move || people.get() != PeopleShown::Hidden>
                        <PeopleCard
                            post=post
                            editable=editable
                            editing=people_editing
                            unsaved=people_unsaved
                            kept=Signal::derive(move || people.get() == PeopleShown::Kept)
                            on_edit=start_people
                            on_cancel=Callback::new(move |()| people_editing.set(false))
                            on_saved=people_saved
                        />
                    </Show>
                    <DetailsCard
                        post=post
                        sampled=sampled
                        on_original=Callback::new(move |()| original.set(true))
                    />
                </aside>
                {move || {
                    if !media_editing.get() {
                        return None;
                    }
                    let p = post.get_untracked()?;
                    let kind = offer(&p, video_editing.get_untracked())?;
                    // The picture the stage already has; a video's own file.
                    let src = match kind {
                        EditKind::Image => src.get_untracked(),
                        EditKind::Video => absolute(&p.file_url),
                    };
                    Some(
                        view! {
                            <EditorSheet
                                post=p
                                kind=kind
                                src=src
                                on_leave=Callback::new(move |()| editor_leaving.set(true))
                                on_close=Callback::new(move |()| media_editing.set(false))
                                on_saved=media_saved
                            />
                        },
                    )
                }}
                {move || {
                    sharing
                        .get()
                        .map(|id| {
                            view! {
                                <ShareSheet
                                    post_ids=vec![id]
                                    on_close=Callback::new(move |()| sharing.set(None))
                                />
                            }
                        })
                }}
                {move || {
                    let at = reversing.get()?;
                    let p = post.get_untracked()?;
                    Some(
                        view! {
                            <ReverseMenu
                                post=p
                                at=at
                                on_close=Callback::new(move |()| reversing.set(None))
                            />
                        },
                    )
                }}
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
/// leaves with the file or a link to it, and, away from the rest, the one
/// that removes it.
#[component]
fn PostBar(
    post: Signal<Option<Arc<Post>>>,
    busy: RwSignal<bool>,
    video_editing: Signal<bool>,
    on_vote: Callback<i16>,
    on_delete: Callback<()>,
    on_edit_media: Callback<()>,
    on_share: Callback<()>,
    on_reverse: Callback<MenuAt>,
) -> impl IntoView {
    let bar = move || {
        let p = post.get()?;
        let deleted = p.deleted;
        let edit = offer(&p, video_editing.get()).map(|kind| {
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
                    title=move || t!("editor-open")
                    aria-label=move || t!("editor-open")
                    aria-haspopup="dialog"
                    on:click=move |_| on_edit_media.run(())
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
                    title=move || t!("share-button")
                    aria-label=move || t!("share-button")
                    aria-haspopup="dialog"
                    on:click=move |_| on_share.run(())
                >
                    <Ico icon=Icon::Share />
                    <span class="bar-label">{move || t!("share-button")}</span>
                </button>
            }
        });
        let reverse = offered(&p).then(|| {
            view! {
                <button
                    type="button"
                    class="icon-btn"
                    title=move || t!("reverse-title")
                    aria-label=move || t!("reverse-title")
                    aria-haspopup="menu"
                    on:click=move |e| {
                        if let Some(button) = e
                            .current_target()
                            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                        {
                            on_reverse.run(beside(&button));
                        }
                    }
                >
                    <Ico icon=Icon::Search />
                    <span class="bar-label">{move || t!("reverse-button")}</span>
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
            <div class="vote" role="group" aria-label=move || t!("post-votes")>
                <button
                    type="button"
                    class="icon-btn"
                    class:on=up
                    disabled=deleted
                    aria-pressed=up.to_string()
                    title=move || t!("post-upvote")
                    aria-label=move || t!("post-upvote")
                    on:click=move |_| on_vote.run(1)
                >
                    <Ico icon=Icon::ThumbUp />
                </button>
                <span class="score" title=move || t!("post-score")>
                    {p.score}
                </span>
                <button
                    type="button"
                    class="icon-btn"
                    class:on=down
                    disabled=deleted
                    aria-pressed=down.to_string()
                    title=move || t!("post-downvote")
                    aria-label=move || t!("post-downvote")
                    on:click=move |_| on_vote.run(-1)
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
                {reverse}
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

/// Takes an edit back from the notice after saving: the original comes out of
/// the recycle bin, the edited copy is deleted for good, and the original
/// shows again if the copy is still on the screen.
fn undo_edit(saved: Saved, q: String, navigate: impl Fn(&str, NavigateOptions) + 'static) {
    spawn_local(async move {
        if saved.binned {
            let trash::Restored { failed, .. } = trash::restore_posts(vec![saved.original]).await;
            if let Some(refused) = failed.into_iter().next() {
                notify(refused.message);
                return;
            }
        }
        // For good: left in the bin, the same edit would be a duplicate the
        // next time. If the purge fails, it just stays in the bin.
        let copy = saved.id;
        match api::call(move |c| async move { c.delete_post(copy).await }).await {
            Ok(()) => {
                let _ = api::call(move |c| async move { c.purge_post(copy).await }).await;
            }
            // The copy is still there, and still shown.
            Err(message) => {
                notify(message);
                return;
            }
        }
        forget(copy);
        gallery::forget_post(copy);
        gallery::forget_lists();
        let on_copy = web_sys::window()
            .and_then(|w| w.location().pathname().ok())
            .is_some_and(|path| path == format!("/post/{copy}"));
        if on_copy {
            let options = NavigateOptions {
                replace: true,
                ..NavigateOptions::default()
            };
            navigate(&route::post(saved.original, &q, None), options);
        }
    });
}
