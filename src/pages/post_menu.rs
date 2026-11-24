//! The menu a right-clicked tile opens, on any page that lists posts as
//! `Thumb`s under `data-press="post"`: open the post here or in a new tab,
//! like it, share it, look its picture up on other sites, download it and, in
//! the gallery, select it for a mass edit or move it to the recycle bin. A long press on a phone opens it too
//! where the browser reports one as a right click, and so do Shift+F10 and
//! the menu key on a focused tile. In the gallery a tile swiped aside does
//! what Settings › Gestures says, from the same actions. Under stars the like
//! gives five; a guest's menu has what reads, and a like asks to sign in.

use std::collections::HashMap;
use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use send_wrapper::SendWrapper;
use starbooru_core::Post;
use wasm_bindgen::JsCast;
use web_sys::MouseEvent;

use super::share::ShareSheet;
use super::{gallery, trash};
use crate::components::download;
use crate::components::icons::Icon;
use crate::components::menu::{self, ActionMenu, MenuAt, MenuItem};
use crate::components::reverse::ReverseMenu;
use crate::components::snackbar::{notify, notify_undo};
use crate::components::swipe::watch as watch_swipes;
use crate::dom;
use crate::logic::reverse::offered;
use crate::logic::stars::next_stars;
use crate::logic::swipe::{Side, action};
use crate::logic::viewer::download_name;
use crate::logic::votes::{Ballot, next_vote};
use crate::store::SwipeAction;
use crate::{api, instance};

/// The stars the menu's like gives, on a server that counts stars.
const LIKE_STARS: u8 = starbooru_core::MAX_STARS;

/// What a like from the menu sends: a like, or under stars all five, and
/// pressed again it takes the vote back.
fn like_ballot(post: &Post, stars: bool) -> Ballot {
    if stars {
        Ballot::Stars(next_stars(post.my_stars, LIKE_STARS))
    } else {
        Ballot::Like(next_vote(post.my_vote, 1))
    }
}

/// Whether the post has the menu's like from this account.
fn has_like(post: &Post, stars: bool) -> bool {
    if stars {
        post.my_stars == LIKE_STARS
    } else {
        post.my_vote > 0
    }
}

/// The tiles whose right click opens the menu.
const TILES: &str = r#"[data-press="post"] [data-post]"#;

/// The tile a right click was on: its post, the address it opens, and where.
fn pressed(e: &MouseEvent) -> Option<(i64, String, MenuAt)> {
    let (tile, at) = menu::pressed(e, TILES)?;
    let id = tile.get_attribute("data-post")?.parse().ok()?;
    let href = tile.get_attribute("href")?;
    Some((id, href, at))
}

/// A menu that's open: for which post, the address its tile opens, and where.
#[derive(Clone)]
struct Open {
    post: Arc<Post>,
    href: String,
    at: MenuAt,
}

/// Listens for right clicks on tiles while it's on the page, and shows their
/// menu. `find` gives the post a tile stands for, `on_open` hears of one
/// opened from the menu as a click on the tile would, and `on_deleted`, where
/// the page can take a tile away, offers the recycle bin. Where the page has
/// a selection, `on_select` offers it; where it keeps its posts, `on_voted`
/// gets each one as it is after a vote. With `swipe`, a tile swiped left or
/// right does that action.
#[component]
pub fn PostPress(
    find: Callback<i64, Option<Arc<Post>>>,
    on_open: Callback<Arc<Post>>,
    #[prop(optional)] on_deleted: Option<Callback<i64>>,
    #[prop(optional)] on_select: Option<Callback<i64>>,
    #[prop(optional)] on_voted: Option<Callback<Arc<Post>>>,
    #[prop(optional, into)] swipe: Option<Signal<(SwipeAction, SwipeAction)>>,
) -> impl IntoView {
    let open = RwSignal::new(None::<Open>);
    // The post the share sheet is open for.
    let sharing = RwSignal::new(None::<i64>);
    // The post whose picture is being looked up, and where its menu was.
    let reversing = RwSignal::new(None::<(Arc<Post>, MenuAt)>);
    // Votes cast from here, a like and stars. The lists the tiles come from
    // aren't asked for again, and the menu should still say which way the
    // next press goes.
    let votes = StoredValue::new(HashMap::<i64, (i16, u8)>::new());
    // A guest reads: no selection, no recycle bin, and a like asks to sign in.
    let guest = untrack(instance::guest);
    let on_select = on_select.filter(|_| !guest);
    let on_deleted = on_deleted.filter(|_| !guest);
    let mine = move |post: &mut Arc<Post>| {
        if let Some((vote, stars)) = votes.with_value(|v| v.get(&post.id).copied()) {
            let p = Arc::make_mut(post);
            p.my_vote = vote;
            p.my_stars = stars;
        }
    };
    let watch = web_sys::window().and_then(|w| w.document()).map(|doc| {
        dom::listen(&doc, "contextmenu", false, move |e| {
            let Some((id, href, at)) = e.dyn_ref::<MouseEvent>().and_then(pressed) else {
                return;
            };
            let Some(mut post) = find.run(id) else {
                return;
            };
            e.prevent_default();
            mine(&mut post);
            open.set(Some(Open { post, href, at }));
        })
    });
    on_cleanup(move || {
        if let Some(watch) = watch {
            watch.stop();
        }
    });

    let navigate = SendWrapper::new(use_navigate());
    let close = Callback::new(move |()| open.set(None));
    let go = Callback::new(move |(post, href): (Arc<Post>, String)| {
        open.set(None);
        on_open.run(post);
        (*navigate)(&href, NavigateOptions::default());
    });
    let like = Callback::new(move |post: Arc<Post>| {
        open.set(None);
        if guest {
            instance::ask_sign_in(Some(t!("guest-vote-why")));
            return;
        }
        let id = post.id;
        let ballot = like_ballot(&post, instance::stars());
        spawn_local(async move {
            let sent = api::call(move |c| async move {
                match ballot {
                    Ballot::Like(value) => c.vote(id, value).await,
                    Ballot::Stars(stars) => c.vote_stars(id, stars).await,
                }
            })
            .await;
            match sent {
                Ok(fresh) => {
                    let _ =
                        votes.try_update_value(|v| v.insert(id, (fresh.my_vote, fresh.my_stars)));
                    let fresh = Arc::new(fresh);
                    gallery::update_post(Arc::clone(&fresh));
                    // The page may have gone meanwhile.
                    if let Some(voted) = on_voted {
                        voted.try_run(fresh);
                    }
                    notify(match ballot {
                        Ballot::Like(1..) => t!("press-liked"),
                        Ballot::Stars(1..) => t!("press-starred"),
                        Ballot::Like(_) | Ballot::Stars(_) => t!("press-unliked"),
                    });
                }
                Err(message) => notify(t!("vote-failed", reason = message)),
            }
        });
    });
    let remove = Callback::new(move |id: i64| {
        open.set(None);
        spawn_local(async move {
            match api::call(move |c| async move { c.delete_post(id).await }).await {
                Ok(()) => {
                    gallery::forget_post(id);
                    if let Some(deleted) = on_deleted {
                        deleted.try_run(id);
                    }
                    notify_undo(t!("press-deleted"), t!("press-undo"), move || {
                        trash::undo_delete(vec![id]);
                    });
                }
                Err(message) => notify(t!("delete-failed", reason = message)),
            }
        });
    });

    // A swipe does at once what the menu would.
    let swipes = swipe.map(|swipe| {
        let run = move |id: i64, side: Side| {
            let Some(mut post) = find.run(id) else {
                return;
            };
            mine(&mut post);
            match action(side, swipe.get_untracked()) {
                SwipeAction::Off => {}
                SwipeAction::Like => like.run(post),
                SwipeAction::Select => {
                    if let Some(select) = on_select {
                        select.run(id);
                    }
                }
                SwipeAction::Share => {
                    if !post.deleted {
                        sharing.set(Some(id));
                    }
                }
                SwipeAction::Download => download::start(
                    absolute(&post.file_url),
                    download_name(&post.sha256, &post.file_url, &post.mime),
                ),
                SwipeAction::Delete => {
                    if on_deleted.is_some() {
                        remove.run(id);
                    }
                }
            }
        };
        watch_swipes(run)
    });
    on_cleanup(move || {
        for watch in swipes.into_iter().flatten() {
            watch.stop();
        }
    });

    let menu = move || {
        let Open { post, href, at } = open.get()?;
        let stars = instance::stars();
        let liked = has_like(&post, stars);
        let like_label = move || match (stars, liked) {
            (false, false) => t!("swipe-like"),
            (false, true) => t!("press-unlike"),
            (true, false) => t!("press-star"),
            (true, true) => t!("press-unstar"),
        };
        let reverse = offered(&post).then(|| {
            let post = Arc::clone(&post);
            view! {
                <MenuItem
                    icon=Icon::Search
                    label=move || t!("reverse-button")
                    on_click=Callback::new(move |()| {
                        open.set(None);
                        reversing.set(Some((Arc::clone(&post), at)));
                    })
                />
            }
        });
        let (here, there) = (Arc::clone(&post), href.clone());
        let voted = Arc::clone(&post);
        let file = absolute(&post.file_url);
        let name = download_name(&post.sha256, &post.file_url, &post.mime);
        let id = post.id;
        let select = on_select.map(|select| {
            view! {
                <MenuItem
                    icon=Icon::Check
                    label=move || t!("swipe-select")
                    on_click=Callback::new(move |()| {
                        open.set(None);
                        select.run(id);
                    })
                />
            }
        });
        let delete = on_deleted.map(|_| {
            view! {
                <MenuItem
                    icon=Icon::Delete
                    label=move || t!("post-delete-title")
                    danger=true
                    on_click=Callback::new(move |()| remove.run(id))
                />
            }
        });
        // The server shares nothing from the recycle bin.
        let share = (!post.deleted).then(|| {
            view! {
                <MenuItem
                    icon=Icon::Share
                    label=move || t!("share-button")
                    on_click=Callback::new(move |()| {
                        open.set(None);
                        sharing.set(Some(id));
                    })
                />
            }
        });
        Some(view! {
            <ActionMenu at=at title=t!("post-alt", id = id) on_close=close>
                <MenuItem
                    icon=Icon::Fullscreen
                    label=move || t!("press-open")
                    on_click=Callback::new(move |()| go.run((Arc::clone(&here), there.clone())))
                />
                <MenuItem
                    icon=Icon::OpenInNew
                    label=move || t!("press-open-tab")
                    on_click=Callback::new(move |()| {
                        open.set(None);
                        open_tab(&href);
                    })
                />
                <MenuItem
                    icon=if stars { Icon::Star } else { Icon::ThumbUp }
                    label=like_label
                    on_click=Callback::new(move |()| like.run(Arc::clone(&voted)))
                />
                {select}
                {share}
                {reverse}
                <MenuItem
                    icon=Icon::Download
                    label=move || t!("post-download")
                    on_click=Callback::new(move |()| {
                        open.set(None);
                        download::start(file.clone(), name.clone());
                    })
                />
                {delete}
            </ActionMenu>
        })
    };
    let sheet = move || {
        sharing.get().map(|id| {
            view! {
                <ShareSheet post_ids=vec![id] on_close=Callback::new(move |()| sharing.set(None)) />
            }
        })
    };
    // In the tile menu's place.
    let sites = move || {
        reversing.get().map(|(post, at)| {
            view! {
                <ReverseMenu post=post at=at on_close=Callback::new(move |()| reversing.set(None)) />
            }
        })
    };
    view! {
        {menu}
        {sheet}
        {sites}
    }
}

/// A server-relative address made absolute.
fn absolute(path: &str) -> String {
    api::client().map_or_else(|| path.to_owned(), |c| c.absolute(path))
}

/// Opens `href`, an address in the app, in a tab of its own.
fn open_tab(href: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.open_with_url_and_target_and_features(href, "_blank", "noopener");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn post(my_vote: i16, my_stars: u8) -> Post {
        serde_json::from_value(serde_json::json!({
            "id": 1, "kind": "image", "mime": "image/png", "file_size": 1, "width": 1,
            "height": 1, "sha256": "00", "rating": "safe", "score": 0, "my_vote": my_vote,
            "my_stars": my_stars, "source": null, "tags": [],
            "created_at": "2026-11-24T00:00:00Z", "deleted": false, "file_url": "/media/x",
            "thumb_url": null
        }))
        .unwrap()
    }

    #[test]
    fn the_menus_like_is_five_stars_under_stars() {
        assert_eq!(like_ballot(&post(0, 0), false), Ballot::Like(1));
        assert_eq!(like_ballot(&post(1, 0), false), Ballot::Like(0));
        assert_eq!(like_ballot(&post(0, 0), true), Ballot::Stars(5));
        assert_eq!(like_ballot(&post(1, 4), true), Ballot::Stars(5));
        assert_eq!(like_ballot(&post(1, 5), true), Ballot::Stars(0));
        assert!(has_like(&post(1, 0), false));
        assert!(!has_like(&post(1, 4), true));
        assert!(has_like(&post(1, 5), true));
    }

    #[test]
    fn the_menu_opens_for_tiles_under_a_press_area() {
        let source = include_str!("post_menu.rs");
        assert!(source.contains(r#"[data-press="post"] [data-post]"#));
        let thumb = include_str!("../components/thumb.rs");
        assert!(thumb.contains("data-post=id"));
        // Deleting needs a list to take the tile out of, and selecting a
        // selection to add it to: the gallery's.
        let gallery = include_str!("gallery.rs");
        for prop in ["on_deleted=", "on_select=", "on_voted="] {
            assert!(gallery.contains(prop), "{prop}");
        }
        for page in [include_str!("explore.rs"), include_str!("post/related.rs")] {
            assert!(page.contains("<PostPress"));
            assert!(!page.contains("on_deleted=") && !page.contains("on_select="));
        }
    }
}
