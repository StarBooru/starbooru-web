//! The menu a right-clicked tile opens, on any page that lists posts as
//! `Thumb`s under `data-press="post"`: open the post here or in a new tab,
//! like it, download it and, in the gallery, move it to the recycle bin. A
//! long press on a phone opens it too where the browser reports one as a
//! right click, and so do Shift+F10 and the menu key on a focused tile.

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

use super::gallery;
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
/// the page can take a tile away, offers the recycle bin.
#[component]
pub fn PostPress(
    find: Callback<i64, Option<Arc<Post>>>,
    on_open: Callback<Arc<Post>>,
    #[prop(optional)] on_deleted: Option<Callback<i64>>,
) -> impl IntoView {
    let open = RwSignal::new(None::<Open>);
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
                    gallery::update_post(Arc::new(fresh));
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
                        deleted.run(id);
                    }
                    notify_undo(t!("press-deleted"), t!("press-undo"), move || {
                        gallery::undo_delete(id);
                    });
                }
                Err(message) => notify(t!("delete-failed", reason = message)),
            }
        });
    });

    move || {
        let Open { post, href, at } = open.get()?;
        let liked = post.my_vote > 0;
        let (here, there) = (Arc::clone(&post), href.clone());
        let voted = Arc::clone(&post);
        let file = absolute(&post.file_url);
        let name = download_name(&post.sha256, &post.file_url, &post.mime);
        let id = post.id;
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
        // Deleting needs a list to take the tile out of.
        let gallery = include_str!("gallery.rs");
        assert!(gallery.contains("on_deleted="));
        for page in [include_str!("explore.rs"), include_str!("post/related.rs")] {
            assert!(page.contains("<PostPress"));
            assert!(!page.contains("on_deleted="));
        }
    }
}
