//! Posts like the one in the viewer, in a row under its bar: the server finds
//! them by shared rare tags and by look. Nothing shows when there are none.

use std::sync::Arc;

use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::Post;

use crate::api;
use crate::components::snackbar::notify;
use crate::components::thumb::Thumb;
use crate::logic::layout::Place;
use crate::pages::post_menu::PostPress;
use crate::route;

/// Posts the viewer suggests.
const RELATED_SHOWN: u32 = 12;

/// The related posts of post `id`. The row of the post before stays until the
/// new one comes, so the page doesn't jump in between. `on_open` hears of one
/// opened here, before the viewer moves to it. While `held`, an editor has
/// changes not yet saved, and a tile opens nothing.
#[component]
pub fn Related(
    #[prop(into)] id: Signal<Option<i64>>,
    #[prop(into)] held: Signal<bool>,
    on_open: Callback<Arc<Post>>,
) -> impl IntoView {
    let found = RwSignal::new(None::<(i64, Vec<Arc<Post>>)>);
    Effect::watch(
        move || id.get(),
        move |wanted, _, _| {
            let Some(of) = *wanted else {
                return;
            };
            spawn_local(async move {
                let result =
                    api::call(move |c| async move { c.related(of, RELATED_SHOWN).await }).await;
                // The viewer may have moved on while it came.
                if id.try_get_untracked().flatten() != Some(of) {
                    return;
                }
                // A failure leaves the row out: the post is what matters here.
                let posts = result
                    .map(|r| r.posts.into_iter().map(Arc::new).collect())
                    .unwrap_or_default();
                found.try_set(Some((of, posts)));
            });
        },
        true,
    );
    let find = Callback::new(move |pid: i64| {
        found.with_untracked(|f| {
            f.as_ref()
                .and_then(|(_, posts)| posts.iter().find(|p| p.id == pid).cloned())
        })
    });
    // Before the tile's link and the router hear of it. A click for a new
    // tab leaves the page as it is, and still goes.
    let hold = move |e: MouseEvent| {
        let plain =
            e.button() == 0 && !(e.ctrl_key() || e.meta_key() || e.shift_key() || e.alt_key());
        if plain && held.get_untracked() {
            e.prevent_default();
            e.stop_propagation();
            notify(t!("post-unsaved"));
        }
    };
    let row = move || {
        let (_, posts) = found.get().filter(|(_, posts)| !posts.is_empty())?;
        let tiles = posts
            .into_iter()
            .enumerate()
            .map(|(index, p)| {
                let href = route::post(p.id, "", None);
                let place: Option<Place> = None;
                view! {
                    <Thumb
                        post=p
                        index=index
                        href=href
                        place=place
                        still=false
                        on_open=on_open
                    />
                }
            })
            .collect_view();
        Some(view! {
            <section class="related" aria-labelledby="related-title">
                <h2 class="card-title" id="related-title">
                    {move || t!("post-related")}
                </h2>
                <div class="related-grid" data-press="post" on:click:capture=hold>
                    {tiles}
                </div>
            </section>
        })
    };
    view! {
        {row}
        <PostPress find=find on_open=on_open />
    }
}
