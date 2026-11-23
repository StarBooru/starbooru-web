//! The menu a right-clicked tile opens, on any page that lists posts as
//! `Thumb`s under `data-press="post"`: open the post here or in a new tab,
//! like it, share it, download it and, in the gallery, select it for a mass
//! edit or move it to the recycle bin. A long press on a phone opens it too
//! where the browser reports one as a right click, and so do Shift+F10 and
//! the menu key on a focused tile.

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
use crate::api;
use crate::components::download;
use crate::components::icons::Icon;
use crate::components::menu::{self, ActionMenu, MenuAt, MenuItem};
use crate::components::snackbar::{notify, notify_undo};
use crate::dom;
use crate::logic::viewer::download_name;
use crate::logic::votes::next_vote;

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
/// gets each one as it is after a vote.
#[component]
pub fn PostPress(
    find: Callback<i64, Option<Arc<Post>>>,
    on_open: Callback<Arc<Post>>,
    #[prop(optional)] on_deleted: Option<Callback<i64>>,
    #[prop(optional)] on_select: Option<Callback<i64>>,
    #[prop(optional)] on_voted: Option<Callback<Arc<Post>>>,
) -> impl IntoView {
    let open = RwSignal::new(None::<Open>);
    // The post the share sheet is open for.
    let sharing = RwSignal::new(None::<i64>);
    // Votes cast from here. The lists the tiles come from aren't asked for
    // again, and the menu should still say which way the next press goes.
    let votes = StoredValue::new(HashMap::<i64, i16>::new());
    let watch = web_sys::window().and_then(|w| w.document()).map(|doc| {
        dom::listen(&doc, "contextmenu", false, move |e| {
            let Some((id, href, at)) = e.dyn_ref::<MouseEvent>().and_then(pressed) else {
                return;
            };
            let Some(mut post) = find.run(id) else {
                return;
            };
            e.prevent_default();
            if let Some(vote) = votes.with_value(|v| v.get(&id).copied()) {
                Arc::make_mut(&mut post).my_vote = vote;
            }
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
        let id = post.id;
        let vote = next_vote(post.my_vote, 1);
        spawn_local(async move {
            match api::call(move |c| async move { c.vote(id, vote).await }).await {
                Ok(fresh) => {
                    let _ = votes.try_update_value(|v| v.insert(id, fresh.my_vote));
                    let fresh = Arc::new(fresh);
                    gallery::update_post(Arc::clone(&fresh));
                    // The page may have gone meanwhile.
                    if let Some(voted) = on_voted {
                        voted.try_run(fresh);
                    }
                    notify(if vote > 0 {
                        t!("press-liked")
                    } else {
                        t!("press-unliked")
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

    let menu = move || {
        let Open { post, href, at } = open.get()?;
        let liked = post.my_vote > 0;
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
                    icon=Icon::ThumbUp
                    label=move || if liked { t!("press-unlike") } else { t!("swipe-like") }
                    on_click=Callback::new(move |()| like.run(Arc::clone(&voted)))
                />
                {select}
                {share}
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
    view! {
        {menu}
        {sheet}
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
