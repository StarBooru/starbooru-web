//! A post in the gallery's grid: a link to the post that waits in the
//! picture's main colour and fades the thumbnail in once it has come. A video
//! says how long it runs and plays, muted, under a mouse; a GIF says so.
//! Under `data-press="post"` a right click opens its menu (`pages/post_menu.rs`).
//! In the gallery's selection mode a click picks it instead, with a check mark.
//! In the gallery it can be swiped aside for what Settings › Gestures says
//! (`swipe.rs`).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use leptos::ev::{KeyboardEvent, MouseEvent, PointerEvent};
use leptos::html;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use starbooru_core::{MediaKind, Post};

use super::icons::{Ico, Icon};
use crate::logic::gallery::{PAGE_SIZE, tint};
use crate::logic::layout::Place;
use crate::logic::swipe::Side;
use crate::logic::viewer::format_duration;
use crate::store::SwipeAction;
use crate::{api, dom, power};

/// Where a tile goes and how big it is, for `.masonry .thumb` in
/// `style/pages/gallery.css`: so many tile widths and so many gaps.
pub(crate) fn place_style(p: Place) -> String {
    format!(
        "; --x: {:.5}; --x-gaps: {:.5}; --y: {:.5}; --y-gaps: {:.5}\
         ; --w: {:.5}; --w-gaps: {:.5}; --h: {:.5}; --h-gaps: {:.5}",
        p.left.tiles,
        p.left.gaps,
        p.top.tiles,
        p.top.gaps,
        p.width.tiles,
        p.width.gaps,
        p.height.tiles,
        p.height.gaps,
    )
}

/// The icon a swipe uncovers.
fn swipe_icon(action: SwipeAction) -> Icon {
    match action {
        SwipeAction::Off => Icon::Close,
        SwipeAction::Like => Icon::ThumbUp,
        SwipeAction::Select => Icon::Check,
        SwipeAction::Share => Icon::Share,
        SwipeAction::Download => Icon::Download,
        SwipeAction::Delete => Icon::Delete,
    }
}

/// `index` is the post's place in the list, for a staggered entrance within a
/// page of results; `still` skips the entrance, for a tile that isn't new to
/// the user. `place` is its spot in the layout. `on_open` hears of a click
/// that opens the post here, and not one for a new tab. While `selecting`,
/// a click or Space goes to `on_toggle` with the post and whether Shift was
/// down, and `selected` says whether it's picked. `swipe` is what swiping it
/// left and right does.
#[component]
pub fn Thumb(
    post: Arc<Post>,
    index: usize,
    #[prop(into)] href: Signal<String>,
    #[prop(into)] place: Signal<Option<Place>>,
    still: bool,
    #[prop(optional)] on_open: Option<Callback<Arc<Post>>>,
    #[prop(optional, into)] selecting: Signal<bool>,
    #[prop(optional, into)] selected: Signal<bool>,
    #[prop(optional)] on_toggle: Option<Callback<(i64, bool)>>,
    #[prop(optional, into)] swipe: Option<Signal<(SwipeAction, SwipeAction)>>,
) -> impl IntoView {
    let id = post.id;
    let opened = Arc::clone(&post);
    let click = move |e: MouseEvent| {
        if selecting.get_untracked() {
            e.prevent_default();
            if let Some(toggle) = on_toggle {
                toggle.run((id, e.shift_key()));
            }
            return;
        }
        let plain =
            e.button() == 0 && !(e.ctrl_key() || e.meta_key() || e.shift_key() || e.alt_key());
        if let Some(open) = on_open.filter(|_| plain) {
            open.run(Arc::clone(&opened));
        }
    };
    let key = move |e: KeyboardEvent| {
        if selecting.get_untracked() && e.key() == " " {
            e.prevent_default();
            if let Some(toggle) = on_toggle {
                toggle.run((id, e.shift_key()));
            }
        }
    };
    let client = api::client();
    let src = post
        .thumb_url
        .as_deref()
        .and_then(|u| client.as_ref().map(|c| c.absolute(u)));
    let video = matches!(post.kind, MediaKind::Video);
    let preview_src = client
        .as_ref()
        .filter(|_| video)
        .map(|c| c.absolute(&post.file_url));
    let loaded = RwSignal::new(false);
    // A browser goes on downloading a picture taken off the page, which after
    // a fast scroll keeps the ones on the screen waiting behind hundreds of
    // others, so a tile that goes before its picture has come cancels it.
    let img: NodeRef<html::Img> = NodeRef::new();
    let kept = Rc::new(RefCell::new(None::<web_sys::HtmlImageElement>));
    img.on_load({
        let kept = Rc::clone(&kept);
        move |el| *kept.borrow_mut() = Some(el)
    });
    let kept = SendWrapper::new(kept);
    on_cleanup(move || {
        if let Some(el) = kept.borrow_mut().take()
            && !el.complete()
        {
            let _ = el.remove_attribute("src");
        }
    });
    // A mouse only: a tap on a phone should open the post, not play it.
    let hovered = RwSignal::new(false);

    let delay = (index % PAGE_SIZE as usize) * 16;
    let mut fixed = format!("--delay: {delay}ms");
    if let Some(colour) = tint(&post.palette) {
        fixed.push_str(&format!("; --tint: {colour}"));
    }
    let style = move || {
        let mut style = fixed.clone();
        if let Some(p) = place.get() {
            style.push_str(&place_style(p));
        }
        style
    };
    let alt = post.tags.join(" ");
    let label = if alt.is_empty() {
        t!("post-alt", id = id)
    } else {
        alt.clone()
    };
    let picture = match src {
        Some(src) => view! {
            <img
                node_ref=img
                src=src
                alt=alt
                loading="lazy"
                decoding="async"
                on:load=move |_| loaded.set(true)
            />
        }
        .into_any(),
        None => view! {
            <div class="thumb-placeholder">
                <Ico icon=Icon::Play size=40 />
            </div>
        }
        .into_any(),
    };
    let preview = move || {
        let src = preview_src.clone().filter(|_| hovered.get())?;
        Some(view! {
            <div class="preview" aria-hidden="true">
                <video src=src muted=true prop:muted=true autoplay loop playsinline preload="auto"></video>
            </div>
        })
    };
    let badge = match post.kind {
        MediaKind::Video => {
            let length = post.duration_ms.and_then(format_duration);
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
    let check = move || {
        selecting.get().then(|| {
            view! {
                <span class="check-mark" aria-hidden="true">
                    <Ico icon=Icon::Check size=20 />
                </span>
            }
        })
    };
    // Under the picture, which uncovers them as it's swiped aside.
    let hints = move || {
        let (left, right) = swipe.map(|s| s.get()).unwrap_or_default();
        [(Side::Left, left), (Side::Right, right)]
            .into_iter()
            .filter(|(_, action)| *action != SwipeAction::Off)
            .map(|(side, action)| {
                view! {
                    <span
                        class="swipe-hint"
                        data-side=side.as_str()
                        data-action=action.as_str()
                        aria-hidden="true"
                    >
                        <Ico icon=swipe_icon(action) />
                    </span>
                }
            })
            .collect_view()
    };
    view! {
        <a
            class="thumb"
            class:still=still
            class:loaded=loaded
            class:selectable=move || selecting.get()
            class:on=move || selecting.get() && selected.get()
            style=style
            href=move || href.get()
            aria-label=label
            role=move || selecting.get().then_some("button")
            aria-pressed=move || {
                selecting.get().then(|| if selected.get() { "true" } else { "false" })
            }
            data-post=id
            on:click=click
            on:keydown=key
            // Shift+click picks a range, not the text in between.
            on:mousedown=move |e: MouseEvent| {
                if e.shift_key() && selecting.get_untracked() {
                    e.prevent_default();
                }
            }
        >
            {hints}
            <div
                class="thumb-media"
                on:pointerenter=move |e: PointerEvent| {
                    if video && e.pointer_type() == "mouse" && !dom::reduced_motion()
                        && !power::active_untracked()
                    {
                        hovered.set(true);
                    }
                }
                on:pointerleave=move |_| {
                    if hovered.get_untracked() {
                        hovered.set(false);
                    }
                }
            >
                {picture}
                {preview}
            </div>
            {badge}
            {check}
        </a>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::layout::Length;

    #[test]
    fn a_tile_says_where_it_goes_in_tiles_and_gaps() {
        let length = |tiles, gaps| Length { tiles, gaps };
        let style = place_style(Place {
            left: length(1.0, 1.0),
            top: length(2.5, 2.0),
            width: length(1.0, 0.0),
            height: length(1.0 / 3.0, 0.0),
        });
        assert_eq!(
            style,
            "; --x: 1.00000; --x-gaps: 1.00000; --y: 2.50000; --y-gaps: 2.00000\
             ; --w: 1.00000; --w-gaps: 0.00000; --h: 0.33333; --h-gaps: 0.00000"
        );
    }

    #[test]
    fn a_tile_waits_in_its_own_colour_and_fades_its_picture_in() {
        let css = include_str!("../../style/pages/gallery.css");
        assert!(css.contains("background: var(--tint, var(--surface-2));"));
        assert!(css.contains(".thumb.loaded img { opacity: 1; }"));
        // Under reduced motion it's there at once.
        let (_, calm) = css
            .split_once("@media (prefers-reduced-motion: reduce)")
            .unwrap();
        assert!(calm.contains(".thumb img { transition: none; }"));
    }
}
