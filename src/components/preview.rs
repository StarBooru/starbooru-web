//! A search's first post as a small square picture of what the search finds,
//! for the Tags page and saved searches. It asks only once it comes near the
//! screen, with one `limit=1` search, and not at all when the gallery or an
//! earlier preview already knows the answer (`logic::previews`).

use std::cell::RefCell;
use std::collections::HashSet;

use chrono::Utc;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::logic::previews::{Preview, Previews};
use crate::pages::gallery;
use crate::{api, dom, store};

thread_local! {
    static KEPT: RefCell<Previews> = RefCell::new(Previews::default());
    /// Searches asked for and not answered yet, so two rows ask once.
    static ASKING: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
    /// Counts answers, so the previews waiting on one look again. Reference
    /// counted, as it outlives any one page.
    static ARRIVED: ArcRwSignal<u32> = ArcRwSignal::new(0);
}

/// Drops the kept previews, after a change that moves posts between searches.
pub fn forget_previews() {
    KEPT.with_borrow_mut(Previews::clear);
}

/// What search `q` shows, if known: `Some(None)` when it finds nothing.
fn known(q: &str) -> Option<Option<Preview>> {
    let owner = store::owner();
    KEPT.with_borrow(|k| k.get(&owner, q, Utc::now()))
        .or_else(|| gallery::kept_first(q).map(|p| p.as_deref().map(Preview::of)))
}

/// Asks for the first post of search `q`, unless that's being asked already.
fn ask(q: String) {
    if !ASKING.with_borrow_mut(|a| a.insert(q.clone())) {
        return;
    }
    let owner = store::owner();
    spawn_local(async move {
        let asked = q.clone();
        let found =
            api::call(move |c| async move { c.search_posts(&asked, None, 1, 1).await }).await;
        ASKING.with_borrow_mut(|a| a.remove(&q));
        // A search the server turns down shows nothing either.
        let first = found
            .ok()
            .and_then(|page| page.posts.first().map(Preview::of));
        if store::owner() == owner {
            KEPT.with_borrow_mut(|k| k.put(&owner, &q, first, Utc::now()));
        }
        ARRIVED.with(|a| a.update(|n| *n = n.wrapping_add(1)));
    });
}

/// The picture for search `q`: its first post's thumbnail over the post's
/// main colour, or an empty tile when it finds nothing. `nothing` says that
/// it's known to find nothing, as for a tag on no posts: it asks nothing.
#[component]
pub fn SearchPreview(
    #[prop(into)] q: Signal<String>,
    #[prop(optional)] nothing: bool,
) -> impl IntoView {
    let tile: NodeRef<html::Span> = NodeRef::new();
    let near = RwSignal::new(false);
    let watch = StoredValue::new(None::<dom::Watch>);
    tile.on_load(move |el| {
        let w = dom::on_near(&el, "0px 0px 300px 0px", move || {
            near.try_set(true);
        });
        watch.set_value(Some(w));
    });
    let stop = move || {
        if let Some(w) = watch.try_update_value(Option::take).flatten() {
            w.stop();
        }
    };
    on_cleanup(stop);
    let arrived = ARRIVED.with(Clone::clone);
    let shown = Memo::new(move |_| {
        arrived.track();
        if nothing {
            Some(None)
        } else if near.get() {
            known(&q.get())
        } else {
            None
        }
    });
    Effect::new(move |_| {
        if near.get() {
            stop();
            if shown.with(Option::is_none) {
                ask(q.get_untracked());
            }
        }
    });
    let loaded = RwSignal::new(false);
    let client = api::client();
    let src = move || {
        let thumb = shown.get().flatten()?.thumb?;
        client.as_ref().map(|c| c.absolute(&thumb))
    };
    let style = move || {
        shown
            .get()
            .flatten()
            .and_then(|p| p.tint)
            .map(|t| format!("--tint: {t}"))
    };
    view! {
        <span
            class="preview"
            class:empty=move || shown.get().is_some_and(|p| p.is_none())
            class:loaded=move || loaded.get()
            style=style
            node_ref=tile
            aria-hidden="true"
        >
            {move || {
                src()
                    .map(|src| {
                        view! {
                            <img
                                src=src
                                alt=""
                                decoding="async"
                                draggable="false"
                                on:load=move |_| loaded.set(true)
                            />
                        }
                    })
            }}
        </span>
    }
}
