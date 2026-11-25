//! A post of the booru named in a forum post (`>>123`, `post #123`): a link
//! to it, and while the pointer rests on it or the focus is on it, a small
//! card with its picture, so a reader sees which one without leaving.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use leptos::ev::{FocusEvent, KeyboardEvent, MouseEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use starbooru_core::Post;

use crate::components::banner::Spinner;
use crate::components::blacklist::entry_label;
use crate::components::icons::{Ico, Icon};
use crate::{api, route};

thread_local! {
    /// The posts asked for, so each is asked once a page: `None` for one
    /// the server doesn't have.
    static SEEN: RefCell<HashMap<i64, Option<Arc<Post>>>> = RefCell::new(HashMap::new());
    /// For each card's own id.
    static NEXT: Cell<u32> = const { Cell::new(0) };
}

/// Forgets the posts asked for, after a change to what they say, such as
/// the blacklist's.
pub fn forget_seen() {
    SEEN.with_borrow_mut(HashMap::clear);
}

/// How long the pointer rests on the link before the card opens, so passing
/// over it doesn't flash one.
const REST: Duration = Duration::from_millis(250);
/// The card's width, and how far it keeps from the window's edges.
const CARD_W: f64 = 176.0;
const EDGE: f64 = 8.0;

/// What the card shows.
#[derive(Clone, Debug, PartialEq)]
enum Found {
    Asking,
    Post(Arc<Post>),
    Missing,
}

/// Where the card goes: above the link when there's room, below it
/// otherwise, and never past the window's sides.
fn place(left: f64, top: f64, bottom: f64, width: f64) -> String {
    let x = left.min(width - CARD_W - EDGE).max(EDGE);
    if top > 220.0 {
        format!("left: {x:.0}px; bottom: calc(100vh - {top:.0}px + 6px)")
    } else {
        format!("left: {x:.0}px; top: {:.0}px", bottom + 6.0)
    }
}

/// The link to post `id`, shown as `text` was written.
#[component]
pub fn Mention(id: i64, text: String) -> impl IntoView {
    let found = RwSignal::new(None::<Found>);
    let at = RwSignal::new(None::<String>);
    let timer = StoredValue::new(None::<TimeoutHandle>);
    let link: NodeRef<html::A> = NodeRef::new();
    let card_id = format!(
        "mention-{}",
        NEXT.with(|n| {
            n.set(n.get().wrapping_add(1));
            n.get()
        })
    );

    let ask = move || {
        if found.with_untracked(Option::is_some) {
            return;
        }
        if let Some(known) = SEEN.with_borrow(|s| s.get(&id).cloned()) {
            found.set(Some(known.map_or(Found::Missing, Found::Post)));
            return;
        }
        found.set(Some(Found::Asking));
        spawn_local(async move {
            let answer = api::fetch(move |c| async move { c.post(id).await }).await;
            let post = match answer {
                Ok(p) if !p.deleted => Some(Arc::new(p)),
                Ok(_) => None,
                Err(f) if f.missing => None,
                // Asked again next time.
                Err(_) => {
                    found.try_set(None);
                    return;
                }
            };
            SEEN.with_borrow_mut(|s| s.insert(id, post.clone()));
            found.try_set(Some(post.map_or(Found::Missing, Found::Post)));
        });
    };
    let open = move || {
        let Some(el) = link.get_untracked() else {
            return;
        };
        let r = el.get_bounding_client_rect();
        let (width, _) = crate::dom::window_size();
        at.set(Some(place(r.left(), r.top(), r.bottom(), width)));
        ask();
    };
    let cancel = move || {
        if let Some(t) = timer.try_update_value(Option::take).flatten() {
            t.clear();
        }
    };
    let close = move || {
        cancel();
        at.try_set(None);
    };
    on_cleanup(cancel);
    let rest = move |_: MouseEvent| {
        cancel();
        timer.set_value(set_timeout_with_handle(open, REST).ok());
    };

    let described = card_id.clone();
    let card = move || {
        let style = at.get()?;
        let inside = match found.get() {
            None | Some(Found::Asking) => view! {
                <span class="mention-wait">
                    <Spinner />
                </span>
            }
            .into_any(),
            Some(Found::Missing) => {
                view! { <span class="mention-missing">{move || t!("mention-missing")}</span> }
                    .into_any()
            }
            // Left out: the card says what hid it instead.
            Some(Found::Post(p)) if !p.blacklisted.is_empty() => {
                let entries = p
                    .blacklisted
                    .iter()
                    .map(|e| entry_label(e))
                    .collect::<Vec<_>>()
                    .join(", ");
                view! {
                    <span class="mention-missing mention-hidden">
                        <Ico icon=Icon::EyeOff size=22 />
                        <span>{move || t!("mention-hidden", entries = entries.clone())}</span>
                    </span>
                }
                .into_any()
            }
            Some(Found::Post(p)) => match p.thumb_url.clone() {
                Some(thumb) => view! { <img src=api::media(&thumb) alt="" decoding="async" /> }
                    .into_any(),
                None => view! { <span class="mention-missing">{move || t!("mention-no-picture")}</span> }
                    .into_any(),
            },
        };
        Some(view! {
            <span class="mention-card" role="tooltip" id=card_id.clone() style=style>
                {inside}
                <span class="mention-id">{move || t!("mention-post", id = id)}</span>
            </span>
        })
    };
    view! {
        <a
            node_ref=link
            class="mention"
            href=route::post(id, "", None)
            aria-describedby=move || at.with(Option::is_some).then(|| described.clone())
            on:mouseenter=rest
            on:mouseleave=move |_| close()
            on:focus=move |_: FocusEvent| open()
            on:blur=move |_: FocusEvent| close()
            on:keydown=move |e: KeyboardEvent| {
                if e.key() == "Escape" && at.with_untracked(Option::is_some) {
                    e.stop_propagation();
                    close();
                }
            }
        >
            {text}
        </a>
        {card}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_card_opens_above_with_room_and_inside_the_window() {
        let above = place(100.0, 400.0, 420.0, 390.0);
        assert!(
            above.contains("bottom: calc(100vh - 400px + 6px)"),
            "{above}"
        );
        let below = place(100.0, 100.0, 120.0, 390.0);
        assert!(below.contains("top: 126px"), "{below}");
        // Near the right edge it moves left; off the left edge, right.
        assert!(place(380.0, 400.0, 420.0, 390.0).starts_with("left: 206px"));
        assert!(place(-50.0, 400.0, 420.0, 390.0).starts_with("left: 8px"));
    }
}
